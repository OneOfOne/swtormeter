use crate::parser::utils::*;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct NamedID {
	pub id: u64,
	pub name: String,
}

impl NamedID {
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

impl Into<u64> for NamedID {
	fn into(self) -> u64 {
		self.id
	}
}

impl Into<String> for NamedID {
	fn into(self) -> String {
		self.name.into()
	}
}

