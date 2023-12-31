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
	hps: Arc<Mutex<Vec<Vec<String>>>>,
	dps: Arc<Mutex<Vec<Vec<String>>>>,
	area: Arc<Mutex<String>>,
	npcs: Arc<Mutex<String>>,
}

impl App {
	fn new() -> Self {
		let states = vec![TableState::default(), TableState::default()];
		Self {
			states,
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
	let mut dps = app.dps.clone();
	let mut area = app.area.clone();
	let mut npcs = app.npcs.clone();
	tokio::spawn(async move {
		let dir = logs_path().unwrap();
		//dbg!(logs_path());
		parse(dir.as_str(), |enc| {
			{
				let mut it = hps.lock().unwrap();
				*it = enc.heal_to_vec();
			}
			{
				let mut it = dps.lock().unwrap();
				*it = enc.dmg_to_vec();
			}
			{
				let mut area = area.lock().unwrap();
				*area = enc.area.clone();
			}
			{
				let mut npcs = npcs.lock().unwrap();
				*npcs = enc
					.npcs
					.clone()
					.into_iter()
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
	let (main_area, areas) = calculate_layout(f.size());

	let rects = Layout::default()
		.constraints([
			Constraint::Percentage(10),
			Constraint::Percentage(40),
			Constraint::Percentage(40),
		])
		.margin(1)
		.split(f.size());

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
	f.render_widget(paragraph, main_area);

	let t = {
		let vec = app.hps.lock().unwrap();
		make_table("Healing".to_owned(), vec)
	};
	f.render_stateful_widget(t, rects[1], &mut app.states[0]);

	let t = {
		let vec = app.dps.lock().unwrap();
		make_table("Damage".to_owned(), vec)
	};
	f.render_stateful_widget(t, rects[2], &mut app.states[1]);
}

fn calculate_layout(area: Rect) -> (Rect, Vec<Vec<Rect>>) {
	let layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Length(4), Constraint::Min(0)])
		.split(area);
	let title_area = layout[0];
	let main_areas = Layout::default()
		.direction(Direction::Vertical)
		.constraints([Constraint::Percentage(50); 6])
		.split(layout[1])
		.iter()
		.map(|&area| {
			Layout::default()
				.direction(Direction::Horizontal)
				.constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
				.split(area)
				.to_vec()
		})
		.collect();
	(title_area, main_areas)
}

fn make_table(name: String, vec: MutexGuard<Vec<Vec<String>>>) -> Table {
	let selected_style = Style::default().add_modifier(Modifier::REVERSED);
	let normal_style = Style::default().bg(Color::Blue);
	let header_cells = ["name", "# casts", "total", "crit %", "xps", "top spells"]
		.iter()
		.map(|h| Cell::from(*h).style(Style::default().fg(Color::Black)));
	let header = Row::new(header_cells).style(normal_style).height(1);

	let rows = vec.iter().map(|item| {
		let height =
			item.iter()
				.map(|content| content.chars().filter(|c| *c == '\n').count())
				.max()
				.unwrap_or(0) + 1;
		let cells = item.iter().map(|c| Cell::from(c.clone()));
		Row::new(cells).height(height as u16)
	});
	let t = Table::new(
		rows,
		[
			Constraint::Percentage(15),
			Constraint::Percentage(6),
			Constraint::Percentage(6),
			Constraint::Percentage(6),
			Constraint::Percentage(6),
			Constraint::Percentage(60),
		],
	)
	.header(header)
	.block(Block::default().borders(Borders::ALL).title(name))
	.highlight_style(selected_style)
	.highlight_symbol(">> ");
	t
}
