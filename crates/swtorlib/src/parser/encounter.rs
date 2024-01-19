use chrono::{Duration, NaiveTime};

use std::ops::Sub;

use super::actor_stats::{ActorStats, Meter};
use super::sorted_vec::SortedVec;
use super::utils::fmt_num;
use super::*;

fn new_sorted_by_health() -> SortedVec<ActorStats> {
	SortedVec::<ActorStats>::new(|a, b| b.max_health.cmp(&a.max_health))
}

fn new_sorted_by_dmg() -> SortedVec<ActorStats> {
	SortedVec::<ActorStats>::new(|a, b| a.dmg_total.total.cmp(&b.dmg_total.total))
}

#[derive(Debug, Clone, Default)]
pub struct Encounter {
	pub area: String,
	pub start: NaiveTime,
	pub ts: NaiveTime,
	pub end: NaiveTime,

	//pub lines: Vec<Line>,
	pub players: SortedVec<ActorStats>,
	pub npcs: SortedVec<ActorStats>,
}

impl Encounter {
	pub fn new(area: String) -> Self {
		Self {
			area,
			players: new_sorted_by_dmg(),
			npcs: new_sorted_by_health(),
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
			let v = if src.is_npc() {
				&mut self.npcs
			} else {
				&mut self.players
			};

			v.update(
				|| ActorStats::new(id.clone()),
				|a| a.id == id,
				|a| a.update(&l.source, &l.target, &l.action),
			)
		}

		if let Some(ref dst) = l.target {
			let id = dst.get_id();
			let v = if dst.is_npc() {
				&mut self.npcs
			} else {
				&mut self.players
			};

			v.update(
				|| ActorStats::new(id.clone()),
				|a| a.id == id,
				|a| a.update(&l.source, &l.target, &l.action),
			)
		}

		true
	}

	pub fn get_vec_for<F: Fn(&ActorStats) -> Meter>(
		m: &SortedVec<ActorStats>,
		elapsed: i64,
		fn_: F,
	) -> Vec<(Vec<String>, f64)> {
		let mut hm = m
			.iter()
			.map(|v| {
				let m = fn_(v);
				let xps = m.xps(elapsed);
				let apm = m.apm(elapsed);
				let o = vec![
					if v.spec.id != 0 {
						format!("{} ({})", v.id.name, v.spec.name)
					} else {
						v.id.name.clone()
					},
					fmt_num(m.casts as f64),
					fmt_num(m.total as f64),
					fmt_num((m.crits as f64 / m.casts as f64) * 100.) + "%",
					fmt_num(apm),
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

	pub fn spells_out(&self) -> Vec<(Vec<String>, f64)> {
		let elapsed = self.elapsed().num_seconds();
		Self::get_vec_for(&self.players, elapsed, ActorStats::all_spells_out)
	}

	pub fn heals_in(&self) -> Vec<(Vec<String>, f64)> {
		let elapsed = self.elapsed().num_seconds();
		Self::get_vec_for(&self.players, elapsed, ActorStats::all_heal_in)
	}

	pub fn dmg_in(&self) -> Vec<(Vec<String>, f64)> {
		let elapsed = self.elapsed().num_seconds();
		Self::get_vec_for(&self.players, elapsed, ActorStats::all_dmg_in)
	}

	pub fn npc_by_health(&self, filter_bosses: bool) -> Vec<(String, i32)> {
		let it = self.npcs.iter();
		if filter_bosses {
			let players_health = self.players.iter().fold(0, |v, a| v + a.max_health);
			let is_boss = self
				.npcs
				.v
				.first()
				.map_or_else(|| false, |v| v.max_health > players_health);

			if is_boss {
				it.filter(|n| n.max_health > players_health / 2)
					.map(|v| {
						let n = v.id.name.clone();
						let h = v.max_health;
						(n, h)
					})
					.collect::<Vec<_>>()
			} else {
				it.map(|v| {
					let n = v.id.name.clone();
					let h = v.max_health;
					(n, h)
				})
				.collect::<Vec<_>>()
			}
		} else {
			it.map(|v| {
				let n = v.id.name.clone();
				let h = v.max_health;
				(n, h)
			})
			.collect::<Vec<_>>()
		}
	}

	pub fn is_boss(&self) -> bool {
		let players_health = self.players.iter().fold(0, |v, a| v + a.max_health);
		if let Some(npc) = self.npcs.v.first() {
			npc.max_health > players_health
		} else {
			false
		}
	}

	pub fn player_by_name(&self, name: &str) -> Option<&ActorStats> {
		let name = if let Some(idx) = name.find('(') {
			name[..idx - 1].to_owned()
		} else {
			return None;
		};
		if let Some(p) = self.players.v.iter().find(|p| p.id.name == name) {
			Some(p)
		} else {
			None
		}
	}

	pub fn is_boss_dead(&self) -> bool {
		self.npcs.iter().all(|v| v.is_dead())
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
