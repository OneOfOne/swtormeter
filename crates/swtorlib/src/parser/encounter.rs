use chrono::{Duration, NaiveDate, NaiveTime};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::Sub;
use std::sync::Arc;

use super::actor_stats::{ActorStats, Meter};
use super::utils::fmt_num;
use super::*;

#[derive(Debug, Clone, Default)]
pub struct Encounter {
	pub area: String,
	pub start: NaiveTime,
	pub ts: NaiveTime,
	pub end: NaiveTime,

	//pub lines: Vec<Line>,
	pub players: HashMap<NamedID, ActorStats>,
	pub npcs: HashMap<NamedID, ActorStats>,
}

impl Encounter {
	pub fn append(&mut self, l: &Line) -> bool {
		//self.lines.push(l.clone());
		match &l.action {
			Action::EnterCombat => {
				self.start = l.ts;
				return false;
			}

			Action::ExitCombat => {
				self.end = l.ts;
				return false;
			}

			_ => {
				if self.start == NaiveTime::MIN {
					return false;
				}
				self.ts = l.ts;
			}
		};

		if let Some(ref src) = l.source {
			let id = match src.typ {
				ActorType::Player | ActorType::NPC => src.id.clone(),
				ActorType::Companion(ref n) => NamedID {
					id: n.id,
					name: format!("{} ({})", n.name, src.id.name),
				},
			};
			let astats = if src.is_npc() {
				self.npcs
					.entry(id.clone())
					.or_insert_with(|| ActorStats::new(id))
			} else {
				self.players
					.entry(id.clone())
					.or_insert_with(|| ActorStats::new(id))
			};
			//dbg!(&astats);
			astats.update(&l.source, &l.target, &l.action)
		}

		if let Some(ref dst) = l.target {
			let id = match dst.typ {
				ActorType::Player | ActorType::NPC => dst.id.clone(),
				ActorType::Companion(ref n) => NamedID {
					id: n.id,
					name: format!("{} ({})", n.name, dst.id.name),
				},
			};
			let astats = if dst.is_npc() {
				self.npcs
					.entry(id.clone())
					.or_insert_with(|| ActorStats::new(id))
			} else {
				self.players
					.entry(id.clone())
					.or_insert_with(|| ActorStats::new(id))
			};

			astats.update(&l.source, &l.target, &l.action)
		}

		true
	}

	pub fn get_vec_for<F: Fn(&ActorStats) -> Meter>(
		m: &HashMap<NamedID, ActorStats>,
		elapsed: i64,
		fn_: F,
	) -> Vec<((Vec<String>, f64))> {
		let mut hm = m
			.iter()
			.map(|(_, v)| {
				let m = fn_(v);
				let xps = m.xps(elapsed);
				let o = vec![
					if v.spec.id != 0 {
						format!("{} ({})", v.id.name, v.spec.name)
					} else {
						v.id.name.clone()
					},
					fmt_num(m.casts as f64),
					fmt_num(m.total as f64),
					fmt_num((m.crits as f64 / m.casts as f64) * 100.) + "%",
					fmt_num(xps),
				];
				(o, xps)
			})
			.collect::<Vec<_>>();
		hm.sort_by(|(_, a), (_, b)| b.total_cmp(&a));
		hm
	}

	pub fn heals_out(&self) -> Vec<((Vec<String>, f64))> {
		let elapsed = self.elapsed().num_seconds();
		Self::get_vec_for(&self.players, elapsed, ActorStats::all_heal_out)
	}

	pub fn dmg_out(&self) -> Vec<((Vec<String>, f64))> {
		let elapsed = self.elapsed().num_seconds();
		Self::get_vec_for(&self.players, elapsed, ActorStats::all_dmg_out)
	}

	pub fn heals_in(&self) -> Vec<((Vec<String>, f64))> {
		let elapsed = self.elapsed().num_seconds();
		Self::get_vec_for(&self.players, elapsed, ActorStats::all_heal_in)
	}

	pub fn dmg_in(&self) -> Vec<((Vec<String>, f64))> {
		let elapsed = self.elapsed().num_seconds();
		Self::get_vec_for(&self.players, elapsed, ActorStats::all_dmg_in)
	}

	pub fn elapsed(&self) -> Duration {
		if self.end != NaiveTime::MIN {
			self.end.sub(self.start)
		} else {
			self.ts.sub(self.start)
		}
	}
}

#[derive(Debug, Clone)]
pub struct Encounters {
	all: Vec<Encounter>,
	last_area: String,
}

impl Encounters {
	pub fn new() -> Self {
		Self {
			//		start,
			all: vec![],
			last_area: "n/a".to_string(),
		}
	}

	pub async fn process<F: Fn(&Encounter)>(&mut self, rx: &mut Receiver<Line>, process: F) {
		while let Some(l) = rx.recv().await {
			let ln = self.all.len();
			match l.action {
				Action::AreaEntered(n) => self.last_area = n.name.into(),
				Action::EnterCombat => {
					let mut e = Encounter::default();
					e.area = self.last_area.clone();
					e.append(&l);
					self.all.push(e);
				}

				Action::ExitCombat => {
					if ln > 0 {
						let mut e = self.all.get_mut(ln - 1).unwrap();
						e.append(&l);
						process(e);
					}
				}

				_ => {
					if ln > 0 {
						let e = self.all.get_mut(ln - 1).unwrap();
						if e.end != NaiveTime::MIN {
							continue;
						}
						if e.append(&l) {
							process(e);
						}
					}
				}
			}
		}
	}
}
