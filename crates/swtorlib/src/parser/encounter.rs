use chrono::{Duration, NaiveTime};
use std::collections::HashMap;

use std::ops::Sub;

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
	pub fn new(area: String) -> Self {
		Self {
			area,
			..Default::default()
		}
	}

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
				if self.end != NaiveTime::MIN {
					return false;
				}
				self.ts = l.ts;
			}
		};

		if let Some(ref src) = l.source {
			let id = src.get_id();
			let astats = if src.is_npc() {
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

		if let Some(ref dst) = l.target {
			let id = dst.get_id();

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
	) -> Vec<(Vec<String>, f64)> {
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

		hm.sort_by(|(_, a), (_, b)| b.total_cmp(a));

		hm
	}

	pub fn heals_out(&self) -> Vec<(Vec<String>, f64)> {
		let elapsed = self.elapsed().num_seconds();
		Self::get_vec_for(&self.players, elapsed, ActorStats::all_heal_out)
	}

	pub fn dmg_out(&self) -> Vec<(Vec<String>, f64)> {
		let elapsed = self.elapsed().num_seconds();
		Self::get_vec_for(&self.players, elapsed, ActorStats::all_dmg_out)
	}

	pub fn heals_in(&self) -> Vec<(Vec<String>, f64)> {
		let elapsed = self.elapsed().num_seconds();
		Self::get_vec_for(&self.players, elapsed, ActorStats::all_heal_in)
	}

	pub fn dmg_in(&self) -> Vec<(Vec<String>, f64)> {
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

#[derive(Debug, Default, Clone)]
pub struct Encounters {
	all: Vec<Encounter>,
	curr: Option<Encounter>,
	last_area: String,
}

impl Encounters {
	pub fn new() -> Self {
		Self::default()
	}

	pub async fn process<F: Fn(&Encounter, &Line)>(&mut self, rx: &mut Receiver<Line>, process: F) {
		while let Some(l) = rx.recv().await {
			match l.action {
				Action::AreaEntered(n) => self.last_area = n.name,

				Action::EnterCombat => {
					let mut e = Encounter::new(self.last_area.clone());
					e.append(&l);
					if let Some(oe) = self.curr.replace(e) {
						self.all.push(oe);
					}
				}

				Action::ExitCombat => {
					if let Some(e) = &mut self.curr.take() {
						e.append(&l);
						process(e, &l);
						self.all.push(e.clone());
					}
				}

				_ => {
					if let Some(e) = &mut self.curr {
						if e.end != NaiveTime::MIN {
							continue;
						}

						if e.append(&l) {
							process(e, &l);
						}
					}
				}
			}
		}
	}
}
