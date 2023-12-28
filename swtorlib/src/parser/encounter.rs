use chrono::NaiveTime;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::{Index, Sub};

use crate::parser::utils::num_with_unit;

use super::*;

#[derive(Debug, Clone, Default)]
pub struct Meter {
	pub name: String,

	pub casts: i32,
	pub total: i32,
	pub crits: i32,
	pub xps: f64,

	pub max: HashMap<String, i32>,
}

impl Meter {
	pub fn new(name: String) -> Self {
		Self {
			name,
			..Self::default()
		}
	}

	pub fn update(&mut self, spell: &String, value: i32, crit: bool, seconds: i64) {
		self.casts += 1;
		self.total += value;
		if crit {
			self.crits += 1;
		}
		self.xps = self.total as f64 / seconds as f64;

		let max = self.max.entry(spell.clone()).or_insert(0);
		if value > *max {
			*max = value;
		}
	}

	pub fn max_cast(&self) -> Option<(String, i32)> {
		let mut vec = self.max.iter().collect::<Vec<_>>();
		vec.sort_by(|(_, a), (_, b)| a.cmp(b));
		if vec.len() > 0 {
			let it = vec.pop().unwrap();
			Some((it.0.clone(), *it.1))
		} else {
			None
		}
	}
}

impl fmt::Display for Meter {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"name: {:15} | casts: {:6} | total: {:8} | crit: {:5.02}% | xps: {:8.} | max: {:?}",
			&self.name,
			num_with_unit(self.casts as f64),
			num_with_unit(self.total as f64),
			100. * (self.crits as f64 / self.casts as f64),
			num_with_unit(self.xps),
			self.max_cast().unwrap_or(("n/a".to_string(), 0)),
		)
	}
}

pub struct Meters {
	taken: HashMap<String, Meter>,
	given: HashMap<String, Meter>,
	avoided: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Encounter {
	pub area: String,
	pub start: NaiveTime,
	pub end: NaiveTime,

	//pub lines: Vec<Line>,
	pub npcs: HashSet<String>,
	pub heal: Vec<Meter>,
	pub dmg: Vec<Meter>,
}

impl Encounter {
	pub fn append(&mut self, l: Line) -> bool {
		//self.lines.push(l.clone());
		match l.action.effect.id {
			EffectIDs::ENTER_COMBAT => {
				self.start = l.ts;
				return false;
			}

			EffectIDs::EXIT_COMBAT => {
				self.end = l.ts;
				return false;
			}

			_ => {}
		}

		if let Some(ref src) = l.source {
			let name = match src.typ {
				ActorType::Player => src.name.clone(),
				ActorType::Companion(ref m) => format!("{} ({})", m.name, src.name),
				ActorType::NPC => {
					self.npcs.insert(src.name.clone());
					return false;
				}
			};

			match l.value {
				Value::Heal {
					total: t,
					effective: e,
					critical: c,
				} => {
					Self::update(&mut self.heal, name, |m| {
						let val = if e > 0 { e } else { t };
						m.update(&l.ability.name, val, c, l.ts.sub(self.start).num_seconds());
					});
					return true;
				}

				Value::Damage {
					total: t,
					critical: c,
					typ: _tt,
				} => {
					Self::update(&mut self.dmg, name, |m| {
						m.update(&l.ability.name, t, c, l.ts.sub(self.start).num_seconds());
					});
					return true;
				}

				_ => {}
			}
		}

		false
	}

	fn update<F: FnMut(&mut Meter) -> ()>(v: &mut Vec<Meter>, name: String, mut process: F) {
		let m = if let Some(i) = v.iter().position(|m| m.name == name) {
			&mut v[i]
		} else {
			v.push(Meter::new(name.clone()));
			v.last_mut().unwrap()
		};
		process(m);
		v.sort_by(|a, b| b.xps.total_cmp(&a.xps))
	}
}

#[derive(Debug, Clone)]
pub enum Event {
	EnterCombat,
	ExitCombat,
	Damage(Box<Encounter>),
	Heal(Box<Encounter>),
	Other(Box<Encounter>, Line),
}

#[derive(Debug, Clone)]
pub struct Encounters {
	start: NaiveDateTime,
	all: Vec<Encounter>,
	last_area: String,
}

impl Encounters {
	pub fn new(name: &str) -> Self {
		let name = name.trim_start_matches("combat_").trim_end_matches(".txt");
		let (start, _) = NaiveDateTime::parse_and_remainder(name, "%Y-%m-%d_%H_%M").unwrap();

		Self {
			start,
			all: vec![],
			last_area: "n/a".to_string(),
		}
	}

	pub fn append(&mut self, l: Line) -> Option<Encounter> {
		if l.action.event.id == EventIDs::AREA_ENTERED {
			self.last_area = l.action.effect.name;
			return None;
		}

		let ln = self.all.len();
		match l.action.effect.id {
			EffectIDs::ENTER_COMBAT => {
				let mut e = Encounter::default();
				e.area = self.last_area.clone();
				e.append(l);
				self.all.push(e);
			}

			EffectIDs::EXIT_COMBAT => {
				if ln > 0 {
					let e = self.all.get_mut(ln - 1).unwrap();
					e.append(l);
				}
			}

			_ => {
				if ln > 0 {
					let e = self.all.get_mut(ln - 1).unwrap();
					if e.end != NaiveTime::MIN {
						return None;
					}
					if e.append(l) {
						return Some(e.clone());
					}
				}
			}
		};
		None
	}

	pub async fn process<F: Fn(&Encounter)>(&mut self, rx: &mut Receiver<Line>, process: F) {
		while let Some(l) = rx.recv().await {
			if l.action.event.id == EventIDs::AREA_ENTERED {
				self.last_area = l.action.effect.name;
				continue;
			}

			let ln = self.all.len();

			match l.action.effect.id {
				EffectIDs::ENTER_COMBAT => {
					let mut e = Encounter::default();
					e.area = self.last_area.clone();
					e.append(l);
					self.all.push(e);
				}

				EffectIDs::EXIT_COMBAT => {
					if ln > 0 {
						let e = self.all.get_mut(ln - 1).unwrap();
						e.append(l);
						process(e);
					}
				}

				_ => {
					if ln > 0 {
						let e = self.all.get_mut(ln - 1).unwrap();
						if e.end != NaiveTime::MIN {
							continue;
						}
						if e.append(l) {
							process(e);
						}
					}
				}
			}
		}
	}
}
