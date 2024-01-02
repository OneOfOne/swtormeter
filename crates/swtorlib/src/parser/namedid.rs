use std::{cell::OnceCell, collections::HashMap, rc::Rc, sync::Mutex};

use crate::parser::utils::*;

// fn id_cache() -> &'static Mutex<HashMap<&str, Rc<NamedID>>> {
// 	static INSTANCE: OnceCell<Mutex<HashMap<&str, Rc<NamedID>>>> = OnceCell::new();
// 	INSTANCE.get_or_init(|| {
// 		let m = HashMap::new();
// 		Mutex::new(m)
// 	})
// }

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
