use super::{Action, Actor, NamedID};

use chrono::NaiveTime;

#[derive(Debug, Clone, Default)]
pub struct Line {
	pub ts: NaiveTime,
	pub source: Option<Actor>,
	pub target: Option<Actor>,
	pub action: Action,
	//pub value: Value,
}

impl Line {
	pub fn new(l: &str) -> Option<Self> {
		let l = l.replace("[HIDDEN]", "");
		let mut parts = l.splitn(6, ']').map(|s| s.trim().trim_start_matches('['));
		let ts =
			NaiveTime::parse_from_str(parts.next().unwrap(), "%H:%M:%S.%3f").expect(l.as_str());

		let source = Actor::new(parts.next().unwrap());
		let target = Actor::new(parts.next().unwrap());
		let ability = NamedID::new(parts.next().unwrap());
		let act = parts.next().unwrap();
		let val = parts.next().unwrap();
		let action = Action::new(act, val, ability, &target);
		// let value = Value::new(parts.next().unwrap(), &action);

		// dbg!(l, action);
		// dbg!(&parts[5]);
		Some(Line {
			ts,
			source,
			target,
			action,
			//value,
		})
	}
}
