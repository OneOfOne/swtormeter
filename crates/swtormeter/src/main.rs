use std::{
	cmp::Ordering,
	error::Error,
	io,
	sync::{Arc, Mutex},
	time::Duration,
};

use chrono::NaiveTime;
use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use swtorlib::{
	parse,
	parser::{encounter::Encounter, logs_path, utils::fmt_num},
};

static XPS_HEADER: [&str; 6] = ["name", "# casts", "total", "crit %", "apm", "xps"];

static XPS_COLUMN_WIDTHS: [Constraint; 6] = [
	Constraint::Percentage(45),
	Constraint::Percentage(10),
	Constraint::Percentage(10),
	Constraint::Percentage(10),
	Constraint::Percentage(7),
	Constraint::Percentage(10),
];

#[derive(Default)]
struct App {
	states: Vec<TableState>,
	selected: usize,
	npcs: Arc<Mutex<String>>,
	curr: Arc<Mutex<Encounter>>,
}

impl App {
	fn new() -> Self {
		let states = vec![
			TableState::default(),
			TableState::default(),
			TableState::default(),
			TableState::default(),
		];
		Self {
			states,
			selected: 0,
			..Self::default()
		}
	}
	pub fn next(&mut self) {
		let st = &mut self.states[self.selected];
		let i = match st.selected() {
			Some(i) => {
				let its = self.curr.clone().lock().unwrap().players.len();
				if i >= its - 1 {
					0
				} else {
					i + 1
				}
			}
			None => 0,
		};
		st.select(Some(i));
	}

	pub fn previous(&mut self) {
		let st = &mut self.states[self.selected];
		let i = match st.selected() {
			Some(i) => {
				let its = self.curr.clone().lock().unwrap().players.len();
				if i == 0 {
					its - 1
				} else {
					i - 1
				}
			}
			None => 0,
		};
		st.select(Some(i));
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	// setup terminal
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// create app and run it
	let app = App::new();
	let npcs = app.npcs.clone();
	let curr = app.curr.clone();
	tokio::spawn(async move {
		let dir = logs_path().unwrap();
		//dbg!(logs_path());i
		let last = Arc::new(Mutex::new(NaiveTime::default()));
		parse(dir.as_str(), |enc, _| {
			{
				let mut last = last.lock().unwrap();
				if enc.start.cmp(&last) != Ordering::Equal {
					*last = enc.start;
				}
			}
			{
				let mut curr = curr.lock().unwrap();
				*curr = enc.clone();
			}

			{
				let mut npcs = npcs.lock().unwrap();
				*npcs = enc
					.npc_by_health(true)
					.into_iter()
					.map(|(id, v)| format!("{} ({})", id, fmt_num(v as f64)))
					.collect::<Vec<String>>()
					.join(", ");
				if enc.is_boss() {
					*npcs = format!("** {}", *npcs);
				}
			}
		})
		.await
		.unwrap();
	});
	let res = run_app(&mut terminal, app);

	// restore terminal
	disable_raw_mode()?;
	execute!(
		terminal.backend_mut(),
		LeaveAlternateScreen,
		DisableMouseCapture
	)?;
	terminal.show_cursor()?;

	if let Err(err) = res {
		println!("{err:?}");
	}

	Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
	loop {
		terminal.draw(|f| ui(f, &mut app))?;
		let mut prev = 0;
		let mut set_sel = |idx| {
			app.selected = idx;
			app.states[prev].select(None);
			prev = idx;
		};

		if event::poll(Duration::from_millis(250))? {
			if let Event::Key(key) = event::read()? {
				if key.kind == KeyEventKind::Press {
					match key.code {
						KeyCode::Char('q') => return Ok(()),
						KeyCode::Char('1') => set_sel(0),
						KeyCode::Char('2') => set_sel(1),
						KeyCode::Char('3') => set_sel(2),
						KeyCode::Char('4') => set_sel(3),
						KeyCode::Esc => app.states[app.selected].select(None),
						KeyCode::Down | KeyCode::Char('j') => app.next(),
						KeyCode::Up | KeyCode::Char('k') => app.previous(),
						_ => {}
					}
				}
			}
		}
	}
}

fn ui(f: &mut Frame, app: &mut App) {
	let (header, areas, footer) = calculate_layout(f.size());

	let create_block = |title| {
		Block::default()
			.borders(Borders::ALL)
			.style(Style::default().fg(Color::Gray))
			.title(Span::styled(
				title,
				Style::default().add_modifier(Modifier::BOLD),
			))
	};

	let enc = app.curr.lock().unwrap();

	let elapsed = enc.elapsed();

	let text = vec![Line::from(app.npcs.lock().unwrap().clone())];
	let paragraph = Paragraph::new(text.clone())
		.style(Style::default().fg(Color::Gray))
		.block(create_block(format!(
			" {} (elapsed: {:02}:{:02}) ",
			enc.area,
			elapsed.num_minutes(),
			elapsed.num_seconds() - (elapsed.num_minutes() * 60)
		)))
		.wrap(Wrap { trim: true });
	f.render_widget(paragraph, header);

	let (t, states) = {
		let vec = match app.selected {
			0 => enc.heals_out(),
			1 => enc.dmg_out(),
			_ => Vec::new(),
		};
		let states = if let Some(idx) = app.states[app.selected].selected() {
			if idx >= vec.len() {
				app.states[app.selected].select(None);
				None
			} else if let Some(p) = enc.player_by_name(&vec[idx].0[0]) {
				let elps = enc.elapsed().num_seconds();
				let x_out = match app.selected {
					0 => p.heal_out_to_vec(elps),
					1 => p.dmg_out_to_vec(elps),
					_ => vec![],
				};
				let mut spells_out = p.spells_out_to_vec(elps);
				spells_out.push((vec![], 0.));
				spells_out.push((
					vec![
						"Other".to_owned(),
						"value".to_owned(),
						"value".to_owned(),
						"-".to_owned(),
						"-".to_owned(),
						"-".to_owned(),
					],
					0.,
				));
				spells_out.push((vec![], 0.));
				spells_out.push((
					vec![
						"Health / Max Health".to_owned(),
						fmt_num(p.health as f64),
						fmt_num(p.max_health as f64),
					],
					0.,
				));
				spells_out.push((vec!["# Deaths".to_owned(), fmt_num(p.deaths as f64)], 0.));
				spells_out.push((vec!["# Revived".to_owned(), fmt_num(p.revives as f64)], 0.));
				spells_out.push((
					vec!["# Interrupted".to_owned(), fmt_num(p.interrupted as f64)],
					0.,
				));

				Some((
					make_table(
						format!(" Spells for {} ", p.id.name),
						XPS_HEADER.as_slice(),
						XPS_COLUMN_WIDTHS.as_slice(),
						&spells_out,
						false,
					),
					make_table(
						format!(" Targets for {} ", p.id.name),
						XPS_HEADER.as_slice(),
						XPS_COLUMN_WIDTHS.as_slice(),
						&x_out,
						false,
					),
				))
			} else {
				None
			}
		} else {
			None
		};
		let title = if app.selected == 0 {
			" * Healing | Damage (2) "
		} else {
			" Healing (1) | * Damage "
		};
		(
			make_table(
				title.to_owned(),
				XPS_HEADER.as_slice(),
				XPS_COLUMN_WIDTHS.as_slice(),
				&vec,
				true,
			),
			states,
		)
	};
	f.render_stateful_widget(t, areas[0][0], &mut app.states[app.selected]);
	if let Some((spells, x_out)) = states {
		let mut dummy = TableState::default();
		f.render_stateful_widget(spells, areas[0][1], &mut dummy);
		f.render_stateful_widget(x_out, footer, &mut dummy);
	}
}

fn calculate_layout(area: Rect) -> (Rect, Vec<Vec<Rect>>, Rect) {
	let layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Length(4),
			Constraint::Min(0),
			Constraint::Length(15),
		])
		.split(area);
	let main_areas = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Percentage(100)])
		.split(layout[1])
		.iter()
		.map(|&area| {
			Layout::default()
				.direction(Direction::Horizontal)
				.constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
				.split(area)
				.to_vec()
		})
		.collect();
	(layout[0], main_areas, layout[2])
}

fn make_table<'a>(
	name: String,
	header: &[&'static str],
	widths: &[Constraint],
	vec: &[(Vec<String>, f64)],
	selected: bool,
) -> Table<'a> {
	let selected_style = Style::default().add_modifier(Modifier::REVERSED);
	let normal_style = Style::default().bg(Color::Blue);
	let header_cells = header
		.iter()
		.map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
	let header = Row::new(header_cells).style(normal_style).height(1);

	let rows = vec.iter().map(|item| {
		let height =
			item.0
				.iter()
				.map(|content| content.chars().filter(|c| *c == '\n').count())
				.max()
				.unwrap_or(0) + 1;
		let cells = item.0.iter().map(|c| Cell::from(c.clone()));
		Row::new(cells).height(height as u16)
	});
	let t = Table::new(rows, widths)
		.header(header)
		.block(
			Block::default()
				.borders(Borders::ALL)
				.border_style(if selected {
					Style::default().fg(Color::Green)
				} else {
					Style::default()
				})
				.title(name),
		)
		.highlight_style(selected_style);
	//.highlight_symbol(">");
	t
}
