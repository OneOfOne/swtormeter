use std::{collections::HashMap, sync::OnceLock};

pub struct ActionIDs {}
impl ActionIDs {
	pub const MODIFY_THREAT: &'static str = "{836045448945483}";
	pub const DAMAGE: &'static str = "{836045448945501}";
	pub const HEAL: &'static str = "{836045448945500}";
}

pub struct ValueIDs {}
impl ValueIDs {
	pub const CHARGES: &'static str = "{836045448953667}";
	pub const SHIELD: &'static str = "{836045448945509}";

	pub const ENERGY: &'static str = "{836045448940874}";
	pub const KINETIC: &'static str = "{836045448940873}";
	pub const ELEMENTAL: &'static str = "{836045448940875}";
	pub const INTERNAL: &'static str = "{836045448940876}";

	pub const ABSORBED: &'static str = "{836045448945511}";

	pub const PARRY: &'static str = "{836045448945503}";
	pub const DEFLECT: &'static str = "{836045448945508}";
	pub const DODGE: &'static str = "{836045448945505}";
	pub const MISS: &'static str = "{836045448945502}"; //
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Damage {
	Energy,
	Kinetic,
	Elemental,
	Internal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Avoidance {
	Absorbed,
	Parry,
	Deflect,
	Dodge,
	Miss,
}

//#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ID(u64);

impl ID {
	pub fn name<'a>() -> &'a str {
		""
	}
}
