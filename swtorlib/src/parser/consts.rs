pub struct EventIDs;
impl EventIDs {
	pub const AREA_ENTERED: u64 = 836045448953664;
}

pub struct EffectIDs;
impl EffectIDs {
	pub const ENTER_COMBAT: u64 = 836045448945489;
	pub const EXIT_COMBAT: u64 = 836045448945490;

	pub const MODIFY_THREAT: u64 = 836045448945483;
	pub const DAMAGE: u64 = 836045448945501;
	pub const HEAL: u64 = 836045448945500;
}

pub struct ValueIDs;
impl ValueIDs {
	pub const CHARGES: u64 = 836045448953667;

	pub const SHIELD: &'static str = "{836045448945509}";
	pub const REFLECTED: &'static str = "{836045448953649}";

	pub const ENERGY: u64 = 836045448940874;
	pub const KINETIC: u64 = 836045448940873;
	pub const ELEMENTAL: u64 = 836045448940875;
	pub const INTERNAL: u64 = 836045448940876;

	pub const ABSORBED: u64 = 836045448945511;

	pub const PARRY: u64 = 836045448945503;
	pub const DEFLECT: u64 = 836045448945508;
	pub const DODGE: u64 = 836045448945505;
	pub const MISS: u64 = 836045448945502; //
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Damage {
	Energy,
	Kinetic,
	Elemental,
	Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Avoidance {
	Absorbed,
	Parry,
	Deflect,
	Dodge,
	Miss,
}
