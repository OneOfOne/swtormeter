use crate::parser::utils::*;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Metadata {
	pub id: u64,
	pub name: String,
}

impl Metadata {
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
