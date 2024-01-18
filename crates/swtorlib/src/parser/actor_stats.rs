use std::ops::AddAssign;

use super::{action::Action, actor::Actor, namedid::NamedID, sorted_vec::SortedVec};

#[derive(Debug, Clone, Default, Hash, PartialEq)]
pub struct Meter {
	pub id: NamedID,

	pub casts: i32,
	pub total: i32,
	pub crit_total: i32,
	pub crits: i32,
}

impl Meter {
	pub fn new(id: NamedID) -> Self {
		Self {
			id,
			..Self::default()
		}
	}

	pub fn update(&mut self, value: i32, crit: bool) {
		self.casts += 1;
		self.total += value;
		if crit {
			self.crit_total += value;
			self.crits += 1;
		}
	}

	pub fn xps(&self, seconds: i64) -> f64 {
		self.total as f64 / seconds as f64
	}

	pub fn apm(&self, seconds: i64) -> f64 {
		(self.casts as f64 / (seconds as f64 / 60.0)) as i32 as f64
	}

	//
	// pub fn to_vec(&self) -> Vec<String> {
	// 	let crit = i32(100.) * (self.crits / self.casts);
	// 	vec![
	// 		format!("{} ({}/{})", self.name.clone(), self.class, self.spec),
	// 		self.casts.to_string(),
	// 		self.total.to_string(),
	// 		format!("{}%", crit.to_string()),
	// 		self.xps.to_string(),
	// 		self.spells().join(", "),
	// 	]
	// }
}

impl AddAssign<&Meter> for Meter {
	fn add_assign(&mut self, other: &Self) {
		self.casts += other.casts;
		self.total += other.total;
		self.crit_total += other.crit_total;
		self.crits += other.crits;
	}
}

// impl fmt::Display for Meter {
// 	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// 		let crit = i32(100.) * (self.crits / self.casts);
// 		write!(
// 			f,
// 			"{:15} | casts: {:4} | total: {:8} | crit: {:5} | xps: {:8.} | max: {:?}",
// 			"", //&self.name,
// 			self.casts,
// 			self.total,
// 			format!("{}%", crit),
// 			self.xps,
// 			"" //self.max_cast().unwrap_or(("n/a".to_string(), 0)),
// 		)
// 	}
// }

fn new_sorted_by_total() -> SortedVec<Meter> {
	SortedVec::<Meter>::new(|a, b| a.total.cmp(&b.total))
}

#[derive(Default, Debug, Clone)]
pub struct ActorStats {
	pub id: NamedID,
	pub spec: NamedID,
	pub class: NamedID,

	pub health: i32,

	pub dmg_out: SortedVec<Meter>,
	pub dmg_in: SortedVec<Meter>,
	pub dmg_total: Meter,

	pub heal_out: SortedVec<Meter>,
	pub heal_in: SortedVec<Meter>,
	pub heal_total: Meter,

	pub spells_out: SortedVec<Meter>,
	pub spells_in: SortedVec<Meter>,

	pub interrupted: i32,
	pub absorbed: i32,
	pub deaths: i32,
	pub revives: i32,
}

impl ActorStats {
	pub fn new(id: NamedID) -> Self {
		Self {
			id,
			dmg_out: new_sorted_by_total(),
			dmg_in: new_sorted_by_total(),
			heal_out: new_sorted_by_total(),
			heal_in: new_sorted_by_total(),
			spells_out: new_sorted_by_total(),
			spells_in: new_sorted_by_total(),
			..Self::default()
		}
	}

	pub fn update(&mut self, src: &Option<Actor>, dst: &Option<Actor>, act: &Action) {
		match act {
			Action::DisciplineChanged { class, spec } => {
				self.class = class.clone();
				self.spec = spec.clone();
				self.health = src.clone().unwrap().max_health;
			}

			Action::Damage {
				ability,
				value,
				critical,
				..
			} => {
				if value == &0 {
					return;
				}

				let m = &mut self.dmg_total;
				m.update(*value, *critical);
				let (dm, sm) = if src.clone().is_some_and(|src| src.get_id() == self.id) {
					self.health = src.clone().unwrap().health;
					(&mut self.dmg_out, &mut self.spells_out)
				} else {
					(&mut self.dmg_in, &mut self.spells_in)
				};
				let id = if let Some(dst) = dst {
					dst.get_id()
				} else if let Some(src) = src {
					src.get_id()
				} else {
					NamedID::default()
				};
				if id.id > 0 {
					Self::update_meter(dm, id, |m| m.update(*value, *critical));
				}
				Self::update_meter(sm, ability.clone(), |m| m.update(*value, *critical));
			}
			Action::Heal {
				ability,
				value,
				//effective,
				critical,
				..
			} => {
				if value == &0 {
					return;
				}
				let m = &mut self.heal_total;
				m.update(*value, *critical);

				let (dm, sm) = if src.clone().is_some_and(|src| src.get_id() == self.id) {
					self.health = src.clone().unwrap().health;
					(&mut self.heal_out, &mut self.spells_out)
				} else {
					(&mut self.heal_in, &mut self.spells_in)
				};

				if let Some(dst) = dst {
					Self::update_meter(dm, dst.get_id(), |m| m.update(*value, *critical));
				}
				Self::update_meter(sm, ability.clone(), |m| m.update(*value, *critical));
			}

			Action::Death => self.deaths += 1,
			Action::Revived => self.revives += 1,
			Action::Interrupted(_) => self.interrupted += 1,

			_ => {}
		}
	}

	pub fn is_dead(&self) -> bool {
		self.deaths > self.revives
	}

	pub fn all_heal_out(&self) -> Meter {
		Self::all_x(&self.heal_out, &self.id)
	}

	pub fn all_heal_in(&self) -> Meter {
		Self::all_x(&self.heal_in, &self.id)
	}
	pub fn all_dmg_out(&self) -> Meter {
		Self::all_x(&self.dmg_out, &self.id)
	}

	pub fn all_dmg_in(&self) -> Meter {
		Self::all_x(&self.dmg_in, &self.id)
	}

	fn all_x(v: &SortedVec<Meter>, id: &NamedID) -> Meter {
		let mut m = Meter::new(id.clone());
		for mm in v.iter() {
			m += mm;
		}
		m
	}

	fn update_meter<F: Fn(&mut Meter)>(v: &mut SortedVec<Meter>, id: NamedID, process: F) {
		v.update(|| Meter::new(id.clone()), |m| m.id == id, process)
	}
}
