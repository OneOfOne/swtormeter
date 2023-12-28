use crate::parser::metadata::Metadata;

#[derive(Debug, Clone, Default)]
pub struct Action {
	pub event: Metadata,
	pub effect: Metadata,
}
impl Action {
	pub fn new(p: &str, delm: char) -> Self {
		let mut parts = p.splitn(2, delm);
		Self {
			event: Metadata::new(parts.next().unwrap().trim()),
			effect: Metadata::new(parts.next().unwrap().trim()),
		}
	}
}
