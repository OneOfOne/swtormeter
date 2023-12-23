use crate::parser::metadata::Metadata;

#[derive(Debug, Clone, Copy, Default)]
pub struct Action<'a> {
	pub event: Metadata<'a>,
	pub effect: Metadata<'a>,
}
impl<'a> Action<'a> {
	pub fn new(p: &'a str, delm: char) -> Self {
		let mut parts = p.splitn(2, delm);
		Self {
			event: Metadata::new(parts.next().unwrap().trim()),
			effect: Metadata::new(parts.next().unwrap().trim()),
		}
	}
}
