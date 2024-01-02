use std::{
	error::Error,
	io,
	sync::{Arc, Mutex, MutexGuard},
	time::Duration,
};

use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use swtorlib::{parse, parser::logs_path};

#[derive(Default)]
struct App {
	states: Vec<TableState>,
	selected: i32,
	hps: Arc<Mutex<Vec<(Vec<String>, f64)>>>,
	hps_in: Arc<Mutex<Vec<(Vec<String>, f64)>>>,
	dps: Arc<Mutex<Vec<(Vec<String>, f64)>>>,
	dps_in: Arc<Mutex<Vec<(Vec<String>, f64)>>>,
	area: Arc<Mutex<String>>,
	elapsed: Arc<Mutex<String>>,
	npcs: Arc<Mutex<String>>,
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
			selected: 1,
			..Self::default()
		}
	}
	pub fn next(&mut self) {
		dbg!(&self.states[0]);
		let i = match self.states[0].selected() {
			Some(i) => {
				let its = self.hps.clone().lock().unwrap().len();
				if i >= its - 1 {
					0
				} else {
					i + 1
				}
			}
			None => 0,
		};
		self.states[0].select(Some(i));
	}

	pub fn previous(&mut self) {
		// let i = match self.state.selected() {
		// 	Some(i) => {
		// 		if i == 0 {
		// 			self.items.len() - 1
		// 		} else {
		// 			i - 1
		// 		}
		// 	}
		// 	None => 0,
		// };
		// self.state.select(Some(i));
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
	let mut hps = app.hps.clone();
	let mut hps_in = app.hps_in.clone();
	let mut dps = app.dps.clone();
	let mut dps_in = app.dps_in.clone();
	let mut area = app.area.clone();
	let mut npcs = app.npcs.clone();
	let mut elapsed = app.elapsed.clone();
	tokio::spawn(async move {
		let dir = logs_path().unwrap();
		//dbg!(logs_path());
		parse(dir.as_str(), |enc| {
			{
				let mut it = hps_in.lock().unwrap();
				*it = enc.heals_in();
			}
			{
				let mut it = dps_in.lock().unwrap();
				*it = enc.dmg_in();
			}
			{
				let mut it = hps.lock().unwrap();
				*it = enc.heals_out();
			}
			{
				let mut it = dps.lock().unwrap();
				*it = enc.dmg_out();
			}
			{
				let mut area = area.lock().unwrap();
				*area = enc.area.clone();
			}
			{
				let mut it = elapsed.lock().unwrap();
				let el = enc.elapsed();
				*it = format!(
					"{:02}:{:02}m",
					el.num_minutes(),
					el.num_seconds() - (el.num_minutes() * 60)
				);
			}

			{
				let mut npcs = npcs.lock().unwrap();
				*npcs = enc
					.npcs
					.keys()
					.into_iter()
					.map(|id| id.name.to_owned())
					.collect::<Vec<String>>()
					.join(", ");
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

		if event::poll(Duration::from_millis(100))? {
			if let Event::Key(key) = event::read()? {
				if key.kind == KeyEventKind::Press {
					match key.code {
						KeyCode::Char('q') => return Ok(()),
						KeyCode::Char('1') => app.selected = 1,
						KeyCode::Char('2') => app.selected = 2,
						KeyCode::Char('3') => app.selected = 3,
						KeyCode::Char('4') => app.selected = 4,
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

	let text = vec![Line::from(app.npcs.lock().unwrap().clone())];

	let paragraph = Paragraph::new(text.clone())
		.style(Style::default().fg(Color::Gray))
		.block(create_block(app.area.lock().unwrap().clone()))
		.wrap(Wrap { trim: true });
	f.render_widget(paragraph, header);

	let text = vec![Line::from(format!(
		"Elapsed: {}",
		app.elapsed.lock().unwrap().clone()
	))];

	let paragraph = Paragraph::new(text.clone())
		.style(Style::default().fg(Color::Gray))
		.block(create_block("Other".into()))
		.wrap(Wrap { trim: true });
	f.render_widget(paragraph, footer);

	let t = {
		let vec = app.hps.lock().unwrap();
		make_table("Healing (1)".to_owned(), vec, app.selected == 1)
	};
	f.render_stateful_widget(t, areas[0][0], &mut app.states[0]);

	let t = {
		let vec = app.dps.lock().unwrap();
		make_table("Damage (2)".to_owned(), vec, app.selected == 2)
	};
	f.render_stateful_widget(t, areas[0][1], &mut app.states[1]);

	let t = {
		let vec = app.hps_in.lock().unwrap();
		make_table("Healing Taken (3)".to_owned(), vec, app.selected == 3)
	};
	f.render_stateful_widget(t, areas[1][0], &mut app.states[0]);

	let t = {
		let vec = app.dps_in.lock().unwrap();
		make_table("Damage Taken (4)".to_owned(), vec, app.selected == 4)
	};
	f.render_stateful_widget(t, areas[1][1], &mut app.states[1]);
}

fn calculate_layout(area: Rect) -> (Rect, Vec<Vec<Rect>>, Rect) {
	let layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Length(4),
			Constraint::Min(0),
			Constraint::Length(6),
		])
		.split(area);
	let main_areas = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Percentage(50); 6])
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

fn make_table(name: String, vec: MutexGuard<Vec<(Vec<String>, f64)>>, selected: bool) -> Table {
	let selected_style = Style::default().add_modifier(Modifier::REVERSED);
	let normal_style = Style::default().bg(Color::Blue);
	let header_cells = ["name", "# casts", "total", "crit %", "xps"]
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
	let t = Table::new(
		rows,
		[
			Constraint::Percentage(55),
			Constraint::Percentage(10),
			Constraint::Percentage(10),
			Constraint::Percentage(10),
			Constraint::Percentage(10),
		],
	)
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
	.highlight_style(selected_style)
	.highlight_symbol(">> ");
	t
}
