use std::collections::HashMap;

use chrono::NaiveTime;

use super::*;
use std::ops::Sub;

enum MeterType {
	Damage,
	Heal,
}
#[derive(Debug, Clone, Default)]
pub struct Meter {
	pub scount: i32,
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
	pub fn new() -> Self {
		Self::default()
	}

	pub fn append(&mut self, l: Line) -> Option<bool> {
		self.lines.push(l.clone());
		match l.action.effect.id {
			EffectIDs::ENTER_COMBAT => {
				self.start = l.ts;
				self.heal.clear();
			}

			EffectIDs::EXIT_COMBAT => {
				self.end = l.ts;
			}

			_ => {
				if !l.source.player {
					return None;
				}
				let src = &l.source;
				match l.value {
					Value::Heal {
						total: t,
						effective: e,
						critical: c,
					} => {
						let mut m = self.heal.entry(src.name.clone()).or_insert(Meter::new());
						m.scount += 1;
						m.total += if e > 0 { e } else { t };
						m.crits += if c { 1 } else { 0 };
						m.xps = m.total as f64 / l.ts.sub(self.start).num_seconds() as f64;
						return Some(true);
					}
					_ => {}
				}
			}
		}
		None
	}
}
