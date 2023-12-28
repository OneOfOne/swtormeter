use std::collections::HashMap;

pub fn extract_num(p: &str, l: char, r: char, right: bool) -> u64 {
	if right {
		extract_rpart(p, l, r).parse().unwrap_or(0)
	} else {
		extract_lpart(p, l, r).parse().unwrap_or(0)
	}
}

pub fn extract_rpart(p: &str, l: char, r: char) -> &str {
	if let Some(start) = p.rfind(l) {
		if let Some(end) = p.rfind(r) {
			return &p[start + 1..end];
		}
	}

	""
}

pub fn extract_lpart(p: &str, l: char, r: char) -> &str {
	if let Some(start) = p.find(l) {
		if let Some(end) = p.find(r) {
			return &p[start + 1..end];
		}
	}

	""
}

pub fn extract_until(p: &str, r: char) -> &str {
	if let Some(idx) = p.find(r) {
		&p[..idx]
	} else {
		p
	}
}

#[derive(Debug, Clone, Default)]
pub struct IdMap<'a> {
	m: HashMap<u64, &'a str>,
}

impl<'a> IdMap<'a> {
	pub fn get(&self, v: u64) -> &'a str {
		self.m.get(&v).unwrap_or(&"<unknown>")
	}
	pub fn get_or_set<F: Fn() -> &'a str>(&mut self, v: u64, name: F) -> &'a str {
		self.m.entry(v).or_insert(name())
	}
}
