use super::{
	utils::{extract_num, extract_until},
	*,
};

pub type Direction = f64;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Position {
	pub x: f64,
	pub y: f64,
	pub z: f64,
	pub dir: Direction,
}

impl Position {
	pub fn new(p: &str) -> Self {
		//dbg!(p);
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

#[derive(Debug, Clone, Default, Eq, Hash, PartialEq)]
pub enum ActorType {
	#[default]
	Player,
	NPC,
	Companion(NamedID),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct Actor {
	pub id: NamedID,
	pub typ: ActorType,
	//pub local_player: bool,
	pub health: i32,
	pub max_health: i32,

	pos: String,
}

impl Actor {
	pub fn new(p: &str) -> Option<Self> {
		if p.is_empty() || p == "=" {
			return None;
		}

		let mut parts = p.split('|').map(|s| s.trim());
		let mut name = parts.next().unwrap().trim();
		let id: u64;
		let mut typ = ActorType::Player;
		if let Some(idx) = name.find('#') {
			id = if let Some(sidx) = name.rfind('/') {
				typ = ActorType::Companion(NamedID::new(&name[sidx + 1..]));
				name[idx + 1..sidx].parse().unwrap()
			} else {
				name[idx + 1..].parse().unwrap()
			};
			name = &name[1..idx];
		} else {
			typ = ActorType::NPC;
			id = extract_num(p, '{', '}', false);
			name = extract_until(p, '{').trim();
		};

		// let pos = Position::new(parts.next().unwrap());
		let pos = parts.next().unwrap().to_owned();

		let mut health = parts
			.next()
			.unwrap()
			.trim_matches(|c| c == '(' || c == ')')
			.splitn(2, '/')
			.map(|v| v.parse::<i32>().unwrap_or(0));

		Some(Actor {
			id: NamedID {
				id,
				name: name.into(),
			},
			typ,
			// local_player: false,
			health: health.next().unwrap(),
			max_health: health.next().unwrap(),
			pos,
		})
	}

	pub fn is_full_health(&self) -> bool {
		self.health == self.max_health
	}

	pub fn position(&self) -> Position {
		Position::new(self.pos.as_str())
	}
}
