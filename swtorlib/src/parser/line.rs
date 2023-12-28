use super::{Action, Actor, Metadata, Value};

use chrono::NaiveTime;

#[derive(Debug, Clone, Default)]
pub struct Line {
	pub ts: NaiveTime,
	pub source: Option<Actor>,
	pub target: Option<Actor>,
	pub ability: Metadata,
	pub action: Action,
	pub value: Value,
}
impl Line {
	pub fn new<'a>(l: &'a str) -> Option<Self> {
		let mut parts = l.splitn(6, ']').map(|s| s.trim().trim_start_matches('['));
		let ts = NaiveTime::parse_from_str(parts.next().unwrap(), "%H:%M:%S.%3f").expect(l);

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
