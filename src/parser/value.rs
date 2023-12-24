use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Value {
	Charges(i32),

	Damage {
		typ: Damage,
		total: i32,
		critical: bool,
	},

	Absorbed {
		total: i32,
		absorbed: i32,
		shielded: bool,
	},

	Avoidance(Avoidance),

	Threat(i32),
	Reflected(i32),

	Heal {
		total: i32,
		effective: i32,
		critical: bool,
	},

	#[default]
	None,
}

impl Value {
	pub fn new<'a>(p: &'a str, act: &Action<'a>) -> Self {
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
			} else if part == ValueIDs::SHIELD {
				shielded = true;
			} else if part == ValueIDs::REFLECTED {
				reflected = true;
			} else if got_value && absorbed == 0 && part.starts_with(|c: char| c.is_ascii_digit()) {
				absorbed = part.parse().unwrap()
			} else if part.starts_with('<') {
				threat = part[1..part.len() - 1].parse().unwrap_or(0);
			}
		}

		match act.effect.id {
			EffectIDs::MODIFY_THREAT => Value::Threat(threat),
			EffectIDs::HEAL => Value::Heal {
				total,
				effective: tilde,
				critical,
			},
			EffectIDs::DAMAGE => {
				if shielded || absorbed > 0 {
					Value::Absorbed {
						total,
						absorbed,
						shielded,
					}
				} else if reflected {
					Value::Reflected(total)
				} else {
					Value::dmg_or_avoid(value_id, total, critical)
				}
			}
			_ => Value::None,
		}
	}

	fn dmg_or_avoid(id: u64, total: i32, critical: bool) -> Self {
		match id {
			ValueIDs::CHARGES => Self::Charges(total),
			//
			ValueIDs::ENERGY => Self::Damage {
				typ: Damage::Energy,
				total,
				critical,
			},
			ValueIDs::KINETIC => Self::Damage {
				typ: Damage::Kinetic,
				total,
				critical,
			},
			ValueIDs::ELEMENTAL => Self::Damage {
				typ: Damage::Elemental,
				total,
				critical,
			},
			ValueIDs::INTERNAL => Self::Damage {
				typ: Damage::Internal,
				total,
				critical,
			},
			//
			//ValueIDs::ABSORBED => Self::Avoidance(Avoidance::Absorbed, absorb),
			ValueIDs::PARRY => Self::Avoidance(Avoidance::Parry),
			ValueIDs::DEFLECT => Self::Avoidance(Avoidance::Deflect),
			ValueIDs::DODGE => Self::Avoidance(Avoidance::Dodge),
			ValueIDs::MISS => Self::Avoidance(Avoidance::Miss),
			_ => Self::None,
		}
	}
}
