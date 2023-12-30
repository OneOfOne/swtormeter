use core::fmt;
use std::{
	collections::HashMap,
	ops::{AddAssign, Div, Mul},
};

pub fn extract_num(p: &str, l: char, r: char, right: bool) -> u64 {
	if right {
		extract_rpart(p, l, r).parse().unwrap_or(0)
	} else {
		extract_lpart(p, l, r).parse().unwrap_or(0)
	}
}

pub fn extract_rpart(p: &str, l: char, r: char) -> &str {
	if let Some(start) = p.rfind(l) {
		if let Some(end) = p.rfind(r) {
			return &p[start + 1..end];
		}
	}

	""
}

pub fn extract_lpart(p: &str, l: char, r: char) -> &str {
	if let Some(start) = p.find(l) {
		if let Some(end) = p.find(r) {
			return &p[start + 1..end];
		}
	}

	""
}

pub fn extract_until(p: &str, r: char) -> &str {
	if let Some(idx) = p.find(r) {
		&p[..idx]
	} else {
		p
	}
}

fn num_with_unit(n: f64) -> String {
	if n > 1_000_000. {
		format!("{:.02}M", n / 1_000_000.)
	} else if n > 1_000. {
		format!("{:.02}K", n / 1_000.)
	} else {
		format!("{:.02}", n)
	}
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NumWithUnit(pub f64);
impl NumWithUnit {
	pub fn to_string(&self) -> String {
		let n = num_with_unit(self.0).trim_end_matches(".00").to_owned();
		n
	}
}
impl fmt::Display for NumWithUnit {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let n = num_with_unit(self.0).trim_end_matches(".00").to_owned();
		if let Some(w) = f.width() {
			write!(f, "{:w$}", n)
		} else {
			write!(f, "{:.3}", n)
		}
	}
}

impl Mul for NumWithUnit {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		NumWithUnit(self.0 * rhs.0)
	}
}

impl Div for NumWithUnit {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		NumWithUnit(self.0 / rhs.0)
	}
}

impl AddAssign for NumWithUnit {
	fn add_assign(&mut self, rhs: Self) {
		self.0 += rhs.0
	}
}

impl Into<f64> for NumWithUnit {
	fn into(self) -> f64 {
		self.0
	}
}

// impl Deref for NumWithUnit {
// 	type Target = f64;
//
// 	fn deref(&self) -> &Self::Target {
// 		&self.0
// 	}
// }
