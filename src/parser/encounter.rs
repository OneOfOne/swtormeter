use std::{borrow::BorrowMut, collections::HashMap};

use chrono::NaiveTime;

use super::*;
use std::ops::Sub;

#[derive(Debug, Clone, Default)]
pub struct Meter {
	pub casts: i32,
	pub total: i32,
	pub crits: i32,
	pub xps: f64,
}
impl Meter {
	pub fn new() -> Self {
		Self { ..Self::default() }
	}
}

#[derive(Debug, Clone, Default)]
pub struct Encounter {
	pub npcs: HashMap<u64, String>,
	pub start: NaiveTime,
	pub end: NaiveTime,
	pub lines: Vec<Line>,

	pub heal: HashMap<String, Meter>,
	pub dmg: HashMap<String, Meter>,
}

impl Encounter {
	pub fn append(&mut self, l: Line) -> bool {
		self.lines.push(l.clone());
		match l.action.effect.id {
			EffectIDs::ENTER_COMBAT => {
				self.start = l.ts;
				self.heal.clear();
				self.dmg.clear();
			}

			EffectIDs::EXIT_COMBAT => {
				self.end = l.ts;
			}

			_ => {
				println!("{:?}", &l);
				if let Some(src) = l.source {
					if src.typ == ActorType::NPC {
						return false;
					}

					let name = match src.typ {
						ActorType::Player => src.name,
						ActorType::Companion(m) => format!("{} ({})", m.name, src.name),
						_ => "".to_string(),
					};

					match l.value {
						Value::Heal {
							total: t,
							effective: e,
							critical: c,
						} => {
							let m = self.heal.entry(name).or_insert(Meter::new());
							m.casts += 1;
							m.total += if e > 0 { e } else { t };
							m.crits += if c { 1 } else { 0 };
							m.xps = m.total as f64 / l.ts.sub(self.start).num_seconds() as f64;
							return true;
						}
						Value::Damage {
							total: t,
							critical: c,
							..
						} => {
							let m = self.dmg.entry(name).or_insert(Meter::new());
							m.casts += 1;
							m.total += t;
							m.crits += if c { 1 } else { 0 };
							m.xps = m.total as f64 / l.ts.sub(self.start).num_seconds() as f64;
							return true;
						}

						_ => {}
					}
				}
			}
		}
		false
	}
}

#[derive(Debug, Clone, Default)]
pub struct Encounters {
	all: Vec<Box<Encounter>>,
}

impl Encounters {
	pub fn append(&mut self, l: Line) -> Option<Box<Encounter>> {
		let ln = self.all.len();
		match l.action.effect.id {
			EffectIDs::ENTER_COMBAT => {
				let mut e = Box::new(Encounter::default());
				e.append(l);
				self.all.push(e);
				None
			}

			EffectIDs::EXIT_COMBAT => {
				if ln > 0 {
					let mut e = self.all.get_mut(ln - 1).unwrap();
					e.append(l);
				}
				None
			}
			EffectIDs::HEAL | EffectIDs::DAMAGE => {
				if ln > 0 {
					let mut e = self.all.get_mut(ln - 1).unwrap();
					if e.end != NaiveTime::MIN {
						return None;
					}
					e.append(l);
					return Some(e.clone());
				}
				None
			}
			_ => None,
		}
	}
}
