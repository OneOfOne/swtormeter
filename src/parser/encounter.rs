use std::collections::HashMap;

use chrono::NaiveTime;

use super::Line;

#[derive(Debug, Clone, Default)]
pub struct Encounter<'a> {
	pub start: NaiveTime,
	pub end: NaiveTime,
	pub lines: Vec<Line<'a>>,
	pub names: HashMap<u64, &'a str>,
}

pub fn parse() {}
