use crate::parser::{consts::*, metadata::NamedID};

use super::actor::Actor;

#[derive(Debug, Clone, Default)]
pub enum DamageKind {
	Energy,
	Kinetic,
	Elemental,
	Internal,

	Absorbed,
	Parry,
	Deflect,
	Dodge,
	Miss,

	#[default]
	Unknown,
}

impl DamageKind {
	pub fn new(id: u64) -> Self {
		match id {
			ENERGY => Self::Energy,
			KINETIC => Self::Kinetic,
			ELEMENTAL => Self::Elemental,
			INTERNAL => Self::Internal,

			PARRY => Self::Parry,
			DEFLECT => Self::Deflect,
			DODGE => Self::Dodge,
			MISS => Self::Miss,

			_ => Self::Unknown,
		}
	}
}

#[derive(Debug, Clone)]
pub struct Heal<'a> {
	pub ability: NamedID<'a>,
	pub value: i32,
	pub effective: i32,
	pub critical: bool,
}

#[derive(Debug, Clone)]
pub struct EffectAbility<'a> {
	effect: NamedID<'a>,
	ability: NamedID<'a>,
}

#[derive(Debug, Clone)]
pub struct Other<'a> {
	event: NamedID<'a>,
	effect: NamedID<'a>,
	ability: NamedID<'a>,
}

#[derive(Debug, Clone)]
pub struct Discipline<'a> {
	class: NamedID<'a>,
	spec: NamedID<'a>,
}

#[derive(Debug, Clone, Default)]
pub enum Action<'a> {
	AreaEntered(NamedID<'a>),
	EnterCombat,
	ExitCombat,

	DisciplineChanged(Discipline<'a>),

	TargetSet(NamedID<'a>),
	TargetCleared,

	AbilityActivate(NamedID<'a>),
	AbilityDeactivate(NamedID<'a>),

	ModifyThreat(NamedID<'a>, i32),
	ModifyCharges(NamedID<'a>),

	Spend,
	Restore,

	Stunned(NamedID<'a>),

	ApplyEffect(EffectAbility<'a>),
	RemoveEffect(EffectAbility<'a>),

	Damage {
		kind: DamageKind,
		ability: NamedID<'a>,
		value: i32,
		absorbed: i32,
		shielded: bool,
		reflected: bool,
		critical: bool,
	},
	Heal {
		ability: NamedID<'a>,
		value: i32,
		effective: i32,
		critical: bool,
	},

	Other(Other<'a>),

	#[default]
	None,
}

impl<'a> Action<'a> {
	pub fn new(act: &'a str, val: &'a str, ability: NamedID, dst: Option<Actor>) -> Self {
		let mut parts = act.splitn(2, ':');
		let event = NamedID::new(parts.next().unwrap());
		let neffect = parts.next().unwrap();
		let effect = NamedID::new(neffect);
		let val = value::new(val);

		dbg!(val);

		match event.id {
			SPEND => Self::Spend,
			RESTORE => Self::Restore,

			DISCIPLINE_CHANGED => {
				// DisciplineChanged
				let mut parts = neffect.split('/');
				let class = NamedID::new(parts.next().unwrap());
				let spec = NamedID::new(parts.next().unwrap());
				Self::DisciplineChanged(Discipline { class, spec })
			}

			AREA_ENTERED => Self::AreaEntered(effect), // AreaEntered

			EVENT => match effect.id {
				// event
				ENTER_COMBAT => Self::EnterCombat,
				EXIT_COMBAT => Self::ExitCombat,

				TARGET_SET => Self::TargetSet(dst.unwrap().id),
				TARGET_CLEARED => Self::TargetCleared,

				ABILITY_ACTIVATE => Self::AbilityActivate(ability),
				ABILITY_DEACTIVATE => Self::AbilityDeactivate(ability),

				MODIFY_THREAT => Self::ModifyThreat(ability, val.threat),

				_ => Self::Other(Other {
					ability,
					event,
					effect,
				}),
			},

			MODIFY_CHARGES => Self::ModifyCharges(effect),

			APPLY_EFFECT => match effect.id {
				// ApplyEffect
				HEAL => Self::Heal {
					ability,
					value: val.total,
					effective: if val.tilde != 0 {
						val.tilde
					} else if dst.unwrap().is_full_health() {
						0
					} else {
						val.total
					},
					critical: val.critical,
				},

				DAMAGE => Self::Damage {
					kind: DamageKind::new(val.value_id),

					ability,
					value: val.total,
					absorbed: val.absorbed,
					shielded: val.shielded,
					reflected: val.reflected,
					critical: val.critical,
				},

				STUNNED_01 | STUNNED_02 | STUNNED_03 | STUNNED_FORCE | STUNNED_TECH => {
					Self::Stunned(ability)
				}

				_ => Self::ApplyEffect(EffectAbility { effect, ability }),
			},

			REMOVE_EFFECT => Self::RemoveEffect(EffectAbility { effect, ability }),

			_ => {
				match effect.id {
					_ => {}
				}
				Self::Other(Other {
					ability,
					effect,
					event,
				})
			}
		}
	}
}

#[derive(Debug, Clone)]
struct value {
	value_id: u64,
	total: i32,
	absorbed: i32,
	tilde: i32,
	threat: i32,
	critical: bool,
	shielded: bool,
	reflected: bool,
}

impl value {
	fn new(p: &str) -> Self {
		dbg!(p);
		let parts = p
			.split(' ')
			.map(|p| p.trim_matches(|c| c == '(' || c == ')'));

		// dbg!(p, eff);
		let mut total = 0;
		let mut got_value = false;
		let mut absorbed = 0;
		let mut tilde = 0;
		let mut critical = false;
		let mut shielded = false;
		let mut reflected = false;
		let mut value_id = 0;
		let mut threat: i32 = 0;

		for part in parts {
			// if part.starts_with("he") {
			// 	return None;
			// }
			// dbg!(part);
			if !got_value && !part.starts_with('<') {
				got_value = true;
				total = if part.ends_with('*') {
					critical = true;
					part.strip_suffix('*').unwrap_or("0").parse().unwrap_or(0)
				} else {
					part.parse().unwrap_or(0)
				};
			} else if let Some(v) = part.strip_prefix('~') {
				tilde = v.parse().unwrap();
			} else if part.starts_with('{') && value_id == 0 {
				value_id = part[1..part.rfind('}').unwrap()].parse().unwrap();
			} else if part == SHIELD_STR {
				shielded = true;
			} else if part == REFLECTED_STR {
				reflected = true;
			} else if got_value && absorbed == 0 && part.starts_with(|c: char| c.is_ascii_digit()) {
				absorbed = part.parse().unwrap()
			} else if part.starts_with('<') {
				threat = part[1..part.len() - 1].parse().unwrap_or(0);
			}
		}

		Self {
			value_id,
			total,
			absorbed,
			tilde,
			threat,
			critical,
			shielded,
			reflected,
		}
	}
}
