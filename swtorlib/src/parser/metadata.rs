use crate::parser::utils::*;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct NamedID<'a> {
	pub id: u64,
	pub name: &'a str,
}

impl<'a> NamedID<'a> {
	pub fn new(p: &str) -> Self {
		let id = extract_num(p, '{', '}', false);
		let name = extract_until(p, '{').trim();
		//println!("{} {} {}", id, name, m.get(id));
		Self {
			id,
			name: name.into(),
		}
	}
}

impl<'a> Into<u64> for NamedID<'a> {
	fn into(self) -> u64 {
		self.id
	}
}

impl<'a> Into<String> for NamedID<'a> {
	fn into(self) -> String {
		self.name.into()
	}
}

impl<'a> Into<&'a str> for NamedID<'a> {
	fn into(self) -> &'a str {
		self.name
	}
}
