use chrono::NaiveTime;

pub fn extract_id(p: &str) -> u64 {
	if let Some(start) = p.rfind('{') {
		if let Some(end) = p.rfind('}') {
			return p[start + 1..end].parse().unwrap_or(0);
		}
	}

	0
}

#[derive(Debug, Clone, Default)]
pub struct Metadata {
	// pub id: u64,
	pub name: String,
}

impl Metadata {
	pub fn new(p: &str) -> Self {
		let parts: Vec<_> = p
			.splitn(2, '{')
			.map(|v| v.trim().trim_matches(|c| c == '}'))
			.collect();
		// dbg!(&parts);
		Self {
			//id: parts[1].parse().unwrap_or(0),
			name: parts[0].to_string(),
		}
	}
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ValueType {
	Charges,

	Energy,
	Kinetic,
	Elemental,
	Internal,

	Absorbed,
	Parry,
	Deflect,
	Dodge,
	Miss,

	Threat,
	Heal,

	#[default]
	Empty,
}

impl ValueType {
	pub fn new(p: &str) -> Self {
		match p {
			"charges" => Self::Charges,

			"energy" => Self::Energy,
			"kinetic" => Self::Kinetic,
			"elemental" => Self::Elemental,
			"internal" => Self::Internal,

			"absorbed" => Self::Absorbed,
			"~parry" => Self::Parry,
			"deflect" => Self::Deflect,
			"~dodge" => Self::Dodge,
			"~miss" => Self::Miss,

			_ => Self::Empty,
		}
	}
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Value {
	pub typ: ValueType,
	pub critical: bool,
	pub shield: bool,
	pub value: i32,
	pub tilde: i32,
	pub absorbed: i32,
}

impl Value {
	pub fn new(p: &str, eff: &String) -> Self {
		let parts: Vec<_> = p
			.split(' ')
			.map(|p| p.trim_matches(|c| c == '(' || c == ')' || c == '<' || c == '>' || c == ' '))
			.collect();

		let mut typ = ValueType::Empty;
		if eff == "ModifyThreat" {
			typ = ValueType::Threat;
		} else if eff == "Heal" {
			typ = ValueType::Heal;
		} else if let Some(idx) = p.find(|c: char| c.is_lowercase()) {
			let p = &p[idx..];
			if let Some(idx) = p.find(|c: char| !c.is_lowercase()) {
				typ = ValueType::new(&p[..idx])
			}
		}

		let it = parts.iter();

		Self {
			typ,
			critical: parts[0].contains('*'),
			shield: p.contains("-shield"),
			value: parts[0].trim_end_matches('*').parse().unwrap_or(0),

			tilde: if let Some(idx) = it.clone().position(|&v| v != "" && &v[..1] == "~") {
				parts[idx][1..].parse().unwrap_or(0)
			} else {
				0
			},

			absorbed: if let Some(idx) = it.clone().position(|&v| v == "absorbed") {
				parts[idx - 1].parse().unwrap_or(0)
			} else {
				0
			},
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct Action {
	pub event: String,
	pub effect: String,
}
impl Action {
	pub fn new(p: &str) -> Self {
		let parts: Vec<_> = p.splitn(2, ':').collect();
		Self {
			event: Metadata::new(parts[0]).name,
			effect: Metadata::new(parts[1]).name,
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct Position {
	pub x: f64,
	pub y: f64,
	pub z: f64,
	pub dir: f64,
}
impl Position {
	pub fn new(p: &str) -> Self {
		let pos: Vec<_> = p
			.trim_matches(|c| c == '(' || c == ')')
			.splitn(4, ',')
			.map(|n| n.parse::<f64>().unwrap_or(0.0))
			.collect();

		Self {
			x: pos[0],
			y: pos[1],
			z: pos[2],
			dir: pos[3],
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct Actor {
	pub id: u64,
	pub name: String,
	pub player: bool,
	pub npc: bool,
	pub companion: bool,
	pub local_player: bool,
	pub health: i32,
	pub max_health: i32,
	pub pos: Position,
}
impl Actor {
	pub fn new(p: &str) -> Self {
		let parts: Vec<_> = p.split('|').map(|s| s.trim()).collect();

		if parts.is_empty() || parts[0].is_empty() || parts[0] == "=" {
			return Actor::default();
		}

		let mut id: u64 = 0;
		let mut name: &str = "";
		if let Some(idx) = parts[0].find('#') {
			id = parts[0][idx + 1..].parse().unwrap_or(0);
			name = &parts[0][..idx]
		}
		// local_player

		let p0 = parts[0].as_bytes();
		let companion = p0[0] == b'@' && p0.contains(&b'/');
		let p2: Vec<_> = parts[2]
			.trim_matches(|c| c == '(' || c == ')')
			.splitn(2, '/')
			.map(|v| v.parse::<i32>().unwrap_or(0))
			.collect();

		Actor {
			id,
			name: name.to_string(),
			player: p0[0] == b'@' && !companion,
			npc: p0[0] != b'@',
			companion,
			local_player: false,
			health: p2[0],
			max_health: p2[1],
			pos: Position::new(parts[1]),
		}
	}

	pub fn empty(&self) -> bool {
		self.id == 0
	}
}

#[derive(Debug, Clone)]
pub struct Line {
	pub ts: NaiveTime,
	pub source: Actor,
	pub target: Actor,
	pub ability: Metadata,
	pub action: Action,
	pub value: Value,
}
impl Line {
	pub fn new(l: &str) -> Self {
		let parts: Vec<_> = l
			.split(']')
			.map(|s| s.trim().trim_start_matches('['))
			.collect();

		let ts = NaiveTime::parse_from_str(parts[0], "%H:%M:%S.%3f").unwrap_or(NaiveTime::MIN);

		let action = Action::new(parts[4]);
		// dbg!(&parts[5]);
		Line {
			ts,
			source: Actor::new(parts[1]),
			target: Actor::new(parts[2]),
			ability: Metadata::new(parts[3]),
			action: action.clone(),
			value: Value::new(parts[5], &action.effect),
		}
	}

	pub fn is_heal(&self) -> bool {
		self.action.effect == "Heal"
	}

	pub fn get_type(&self) -> ValueType {
		self.value.typ
	}
}

#[cfg(test)]
mod tests {

	use std::{fs, ops::Sub};

	use super::*;
	#[test]
	fn metadata_test() {
		let f = fs::read("/home/oneofone/code/rust/swtormeter/log.txt").unwrap();
		let mut cmpt = false;
		let mut start: NaiveTime = NaiveTime::MIN;
		let mut end: NaiveTime = NaiveTime::MIN;
		for line in String::from_utf8_lossy(&f).lines() {
			let l = Line::new(line);
			if l.action.effect == "EnterCombat" {
				cmpt = true;
				start = l.ts;
				continue;
			}
			if l.action.effect == "ExitCombat" {
				cmpt = false;
				end = l.ts;
				continue;
			}
			if !cmpt {
				continue;
			}
			let (typ, src, dst, abt, act, value) = (
				l.get_type(),
				l.source.name,
				l.target.name,
				l.ability.name,
				l.action.effect,
				l.value.value,
			);
			println!("{}", line);
			println!("{:?} {} {} {} {} {}", typ, src, dst, abt, act, value)
		}
		println!("{}", end.sub(start).num_seconds());
		// dbg!("x", "3503452067987731".parse::<u64>().ok());
		// let result = Line::new("[19:46:01.686] [@Nyx'ayuna#686862584797878|(-109.03,-630.11,-63.40,108.12)|(263258/380355)] [=] [Berserk {4056205769048064}] [ApplyEffect {836045448945477}: Heal {836045448945500}] (3955) <1977>") ;
		// dbg!(result);
		// let result = Line::new("[19:52:17.535] [Toth {2857549116211200}:52422000225810|(-103.70,-609.83,-63.40,-155.69)|(5430688/13451706)] [@Recency#689265319653227|(-101.50,-604.66,-62.68,21.52)|(309708/457173)] [Backhand Smash {2857321482944512}] [ApplyEffect {836045448945477}: Damage {836045448945501}] (5289 kinetic {836045448940873} -shield {836045448945509} (74525 absorbed {836045448945511})) <5289>") ;
		// dbg!(result);
		// let result = Line::new("[19:52:17.948] [@Locutus'of#690037831467479|(-97.30,-622.67,-63.39,165.31)|(140782/428340)] [@Nyx'ayuna#686862584797878|(-102.09,-616.61,-63.40,166.80)|(225043/380355)] [Salvation {812990064492544}] [ApplyEffect {836045448945477}: Heal {836045448945500}] (3075*) <1384>") ;
		// dbg!(result);
	}
}
