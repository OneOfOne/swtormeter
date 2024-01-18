use std::{cmp::Ordering, fmt::Debug, rc::Rc, slice::Iter};

type CmpFn<T> = Rc<dyn Fn(&T, &T) -> Ordering>;

pub struct SortedVec<T: Debug> {
	pub v: Vec<T>,
	cmp: CmpFn<T>,
}

impl<T: Debug> SortedVec<T> {
	pub fn new<Cmp: Fn(&T, &T) -> Ordering + 'static>(cmp: Cmp) -> Self {
		Self {
			v: Vec::new(),
			cmp: Rc::new(cmp),
		}
	}

	pub fn update<New: Fn() -> T, Find: Fn(&T) -> bool, Process: Fn(&mut T)>(
		&mut self,
		v: New,
		find: Find,
		proc: Process,
	) {
		let v = if let Some(i) = self.v.iter().position(find) {
			&mut self.v[i]
		} else {
			self.v.push(v());
			self.v.last_mut().unwrap()
		};
		proc(v);
		self.v.sort_by(self.cmp.as_ref());
	}

	pub fn iter(&self) -> Iter<T> {
		self.v.iter()
	}

	pub fn is_empty(&self) -> bool {
		self.v.is_empty()
	}

	pub fn len(&self) -> usize {
		self.v.len()
	}
}

unsafe impl<T: Debug> Send for SortedVec<T> {}

impl<T: Debug> Default for SortedVec<T> {
	fn default() -> Self {
		Self::new(|_, _| Ordering::Equal)
	}
}

impl<T: Debug> Debug for SortedVec<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.v)
	}
}

impl<T: Debug + Clone> Clone for SortedVec<T> {
	fn clone(&self) -> Self {
		Self {
			v: self.v.clone(),
			cmp: self.cmp.clone(),
		}
	}
}
