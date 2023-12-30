use chrono::NaiveTime;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::Sub;

use super::utils::NumWithUnit;
use super::*;

#[derive(Debug, Clone, Default)]
pub struct Meter {
	pub name: String,

	pub casts: NumWithUnit,
	pub total: NumWithUnit,
	pub crits: NumWithUnit,
	pub xps: NumWithUnit,

	pub max: HashMap<String, i32>,
}

impl Meter {
	pub fn new(name: String) -> Self {
		Self {
			name,
			..Self::default()
		}
	}

	pub fn update(&mut self, spell: &str, value: i32, crit: bool, seconds: i64) {
		self.casts += NumWithUnit(1.);
		self.total += NumWithUnit(value as f64);
		if crit {
			self.crits += NumWithUnit(1.);
		}
		self.xps = self.total / NumWithUnit(seconds as f64);

		let max = self.max.entry(spell.into()).or_insert(0);
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

	pub fn to_vec(&self) -> Vec<String> {
		let crit = NumWithUnit(100.) * (self.crits / self.casts);
		vec![
			self.name.clone(),
			self.casts.to_string(),
			self.total.to_string(),
			format!("{}%", crit.to_string()),
			self.xps.to_string(),
		]
	}
}

impl fmt::Display for Meter {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let crit = NumWithUnit(100.) * (self.crits / self.casts);
		write!(
			f,
			"{:15} | casts: {:4} | total: {:8} | crit: {:5} | xps: {:8.} | max: {:?}",
			&self.name,
			self.casts,
			self.total,
			format!("{}%", crit),
			self.xps,
			self.max_cast().unwrap_or(("n/a".to_string(), 0)),
		)
	}
}

fn trim_to_n(s: &String, n: usize) -> String {
	if s.len() <= n {
		s.into()
	} else {
		format!("{}...", &s[..n - 3])
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
		match l.action {
			Action::EnterCombat => {
				self.start = l.ts;
				return false;
			}

			Action::ExitCombat => {
				self.end = l.ts;
				return false;
			}

			_ => {}
		}

		if let Some(ref src) = l.source {
			let name = match src.typ {
				ActorType::Player => src.id.name.clone(),
				ActorType::Companion(ref m) => format!("{} ({})", m.name, src.id.name).as_str(),
				ActorType::NPC => {
					self.npcs.insert(src.id.name.into());
					return false;
				}
			};

			match l.action {
				Action::Heal {
					ability,
					value: v,
					effective: e,
					critical: c,
					..
				} => {
					Self::update(&mut self.heal, name.to_string(), |m| {
						let val = if e > 0 { e } else { v };
						m.update(
							ability.name.into(),
							val,
							c,
							l.ts.sub(self.start).num_seconds(),
						);
					});
					return true;
				}
				Action::Damage {
					ability,
					value: v,
					critical: c,
					..
				} => {
					Self::update(&mut self.dmg, name.into(), |m| {
						m.update(
							ability.name.into(),
							v,
							c,
							l.ts.sub(self.start).num_seconds(),
						);
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
			v.push(Meter::new(name));
			v.last_mut().unwrap()
		};
		process(m);
		v.sort_by(|a, b| b.xps.0.total_cmp(&a.xps.0))
	}

	pub fn dmg_to_vec(&self) -> Vec<Vec<String>> {
		let mut all = Vec::new();
		for m in &self.dmg {
			all.push(m.to_vec());
		}
		all
	}
	pub fn heal_to_vec(&self) -> Vec<Vec<String>> {
		let mut all = Vec::new();
		for m in &self.heal {
			all.push(m.to_vec());
		}
		all
	}
}

#[derive(Debug, Clone)]
pub struct Encounters<'a> {
	start: NaiveDateTime,
	all: Vec<Encounter>,
	last_area: String,
}

impl<'a> Encounters<'a> {
	pub fn new(name: &'a str) -> Self {
		let name = name.trim_start_matches("combat_").trim_end_matches(".txt");
		let (start, _) = NaiveDateTime::parse_and_remainder(name, "%Y-%m-%d_%H_%M").unwrap();

		Self {
			start,
			all: vec![],
			last_area: "n/a".to_string(),
		}
	}

	pub async fn process<F: Fn(&Encounter)>(&mut self, rx: &mut Receiver<Line<'a>>, process: F) {
		while let Some(l) = rx.recv().await {
			let ln = self.all.len();
			match l.action {
				Action::AreaEntered(n) => self.last_area = n.name.into(),
				Action::EnterCombat => {
					let mut e = Encounter::default();
					e.area = self.last_area.clone();
					e.append(l.clone());
					self.all.push(e);
				}

				Action::ExitCombat => {
					if ln > 0 {
						let e = self.all.get_mut(ln - 1).unwrap();
						e.append(l.clone());
						process(e);
					}
				}

				_ => {
					if ln > 0 {
						let e = self.all.get_mut(ln - 1).unwrap();
						if e.end != NaiveTime::MIN {
							continue;
						}
						if e.append(l.clone()) {
							process(e);
						}
					}
				}
			}
		}
	}
}
