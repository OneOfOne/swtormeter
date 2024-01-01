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

pub fn fmt_num(n: f64) -> String {
	if n != n {
		// nan check
		return "0".to_owned();
	}
	if n > 1_000_000. {
		format!("{:.02}M", n / 1_000_000.)
	} else if n > 1_000. {
		format!("{:.02}K", n / 1_000.)
	} else {
		format!("{:.02}", n).replace(".00", "")
	}
}
