pub type Direction = f64;

#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
	pub x: f64,
	pub y: f64,
	pub z: f64,
	pub dir: Direction,
}
impl Position {
	pub fn new(p: &str) -> Self {
		let mut pos = p[1..p.len() - 1]
			.splitn(4, ',')
			.map(|n| n.parse::<f64>().unwrap_or(0.));

		Self {
			x: pos.next().unwrap(),
			y: pos.next().unwrap(),
			z: pos.next().unwrap(),
			dir: pos.next().unwrap(),
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct Actor {
	pub id: u64,
	pub name: String,
	pub player: bool,
	pub companion: bool,
	pub npc: bool,
	//pub local_player: bool,
	pub health: i32,
	pub max_health: i32,
	pub pos: Position,
}

impl Actor {
	pub fn new(p: &str) -> Self {
		if p.is_empty() || p == "=" {
			return Actor::default();
		}

		let mut parts = p.split('|').map(|s| s.trim());
		let mut name = parts.next().unwrap().trim();
		let mut id: u64 = 0;
		if let Some(idx) = name.find('#') {
			id = if let Some(sidx) = name.rfind('/') {
				name[idx + 1..sidx].parse().unwrap()
			} else {
				name[idx + 1..].parse().unwrap()
			};
			name = &name[..idx];
		}

		let mut companion = false;

		let player = if name.starts_with('@') {
			companion = name.contains('/');
			!companion
		} else {
			false
		};

		let pos = Position::new(parts.next().unwrap());

		let mut health = parts
			.next()
			.unwrap()
			.trim_matches(|c| c == '(' || c == ')')
			.splitn(2, '/')
			.map(|v| v.parse::<i32>().unwrap_or(0));

		Actor {
			id,
			name: name.into(),
			player,
			companion,
			npc: !player && !companion,
			// local_player: false,
			health: health.next().unwrap(),
			max_health: health.next().unwrap(),
			pos,
		}
	}
}
