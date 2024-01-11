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

impl From<NamedID> for u64 {
	fn from(val: NamedID) -> Self {
		val.id
	}
}

impl From<NamedID> for String {
	fn from(val: NamedID) -> Self {
		val.name
	}
}
