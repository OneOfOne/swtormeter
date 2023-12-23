use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ValueType {
	Charges(i32),

	Damage {
		typ: Damage,
		value: i32,
		absorbed: i32,
		shielded: bool,
	},
	Avoidance(Avoidance),

	Threat(i32),

	Heal(i32),

	#[default]
	None,
}

impl ValueType {
	pub fn new(id: &str, v: i32, absorbed: i32, shielded: bool) -> Self {
		match id {
			ValueIDs::CHARGES => Self::Charges(v),
			//
			ValueIDs::ENERGY => Self::Damage {
				typ: Damage::Energy,
				value: v,
				absorbed,
				shielded,
			},
			ValueIDs::KINETIC => Self::Damage {
				typ: Damage::Kinetic,
				value: v,
				absorbed,
				shielded,
			},
			ValueIDs::ELEMENTAL => Self::Damage {
				typ: Damage::Elemental,
				value: v,
				absorbed,
				shielded,
			},
			ValueIDs::INTERNAL => Self::Damage {
				typ: Damage::Internal,
				value: v,
				absorbed,
				shielded,
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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Hit {
	Critical,
	#[default]
	Normal,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Value {
	pub typ: ValueType,
	pub critical: bool,
	pub tilde: i32,
}

impl Value {
	pub fn new<'a>(p: &'a str, eff: &Action<'a>) -> Self {
		let parts = p
			.split(' ')
			.map(|p| p.trim_matches(|c| c == '(' || c == ')'));

		// dbg!(p, eff);
		let mut value = 0;
		let mut got_value = false;
		let mut absorbed = 0;
		let mut tilde = 0;
		let mut critical = false;
		let mut shielded = false;
		let mut value_id: &'a str = "";
		let mut threat: i32 = 0;

		for part in parts {
			// if part.starts_with("he") {
			// 	return None;
			// }
			// dbg!(part);
			if !got_value && !part.starts_with('<') {
				got_value = true;
				value = if part.ends_with('*') {
					critical = true;
					part.strip_suffix('*').unwrap_or("0").parse().unwrap_or(0)
				} else {
					part.parse().unwrap_or(0)
				};
			} else if let Some(v) = part.strip_prefix('~') {
				tilde = v.parse().unwrap();
			} else if part.starts_with('{') && value_id.is_empty() {
				value_id = part;
			} else if part == ValueIDs::SHIELD {
				shielded = true
			} else if got_value && absorbed == 0 && part.starts_with(|c: char| c.is_ascii_digit()) {
				absorbed = part.parse().unwrap()
			} else if part.starts_with('<') {
				threat = part[1..part.len() - 1].parse().unwrap_or(0);
			}
		}

		let id = format!("{{{}}}", eff.effect.id);
		let typ = match id.as_str() {
			ActionIDs::MODIFY_THREAT => ValueType::Threat(threat),
			ActionIDs::HEAL => {
				if threat > 0 {
					//dbg!(p);
				}
				// ValueType::Heal(value - tilde - threat)
				ValueType::Heal(if tilde > 0 { tilde } else { value })
			}
			ActionIDs::DAMAGE => ValueType::new(value_id, value, absorbed, shielded),
			_ => ValueType::None,
		};

		// dbg!(&id, id == ActionIDs::HEAL, typ);
		Self {
			typ,
			critical,
			tilde,
		}
	}
}
