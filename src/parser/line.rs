use super::{Action, Actor, Metadata, Value};

use chrono::NaiveTime;

#[derive(Debug, Clone, Copy, Default)]
pub struct Line<'a> {
	pub ts: NaiveTime,
	pub source: Actor<'a>,
	pub target: Actor<'a>,
	pub ability: Metadata<'a>,
	pub action: Action<'a>,
	pub value: Value,
}
impl<'a> Line<'a> {
	pub fn new(l: &'a str) -> Option<Self> {
		let mut parts = l.split(']').map(|s| s.trim().trim_start_matches('['));
		let ts = NaiveTime::parse_from_str(parts.next().unwrap(), "%H:%M:%S.%3f").unwrap();

		let source = Actor::new(parts.next().unwrap());
		let target = Actor::new(parts.next().unwrap());
		let ability = Metadata::new(parts.next().unwrap());
		let action = Action::new(parts.next().unwrap(), ':');
		let value = Value::new(parts.next().unwrap(), &action);

		// dbg!(l, action);
		// dbg!(&parts[5]);
		Some(Line {
			ts,
			source,
			target,
			ability,
			action,
			value,
		})
	}
}
