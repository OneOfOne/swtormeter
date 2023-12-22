pub mod parser;

#[derive(Debug, Clone)]
pub struct Heal {
	pub name: String,
	pub ts: String,
	pub amount: u64,
	pub absorb: u64,
}

#[derive(Debug, Clone)]
pub struct Encounter {
	pub id: u64,
	pub name: String,
	pub start: u64,
	pub end: u64,
	pub healing: Vec<Heal>,
}

pub fn add(left: usize, right: usize) -> usize {
	left + right
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let result = add(2, 2);
		assert_eq!(result, 4);
	}
}
