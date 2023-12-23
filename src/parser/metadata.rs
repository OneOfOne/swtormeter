use crate::parser::utils::*;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Metadata<'a> {
	pub id: u64,
	pub name: &'a str,
}

impl<'a> Metadata<'a> {
	pub fn new(p: &'a str) -> Self {
		let id = extract_num(p, '{', '}', false);
		let name = extract_until(p, '{').trim();
		//println!("{} {} {}", id, name, m.get(id));
		Self { id, name }
	}
}
