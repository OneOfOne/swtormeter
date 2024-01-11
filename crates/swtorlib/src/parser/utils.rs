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
	if !n.is_finite() {
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

pub struct Packet<'a> {
	id: [u8; 4],
	code: u16,
	req: u16,
	sample: u32,
	size: u32,
	size_uncompressed: u32,
	body: Option<&'a [u8]>,
}

impl<'a> Packet<'a> {
	pub const SIZE: usize = 4 + 2 + 2 + 4 + 4 + 4;

	pub fn to_bytes(&self) -> Vec<u8> {
		let mut buf = Vec::with_capacity(24 + self.size as usize);
		buf.extend_from_slice(&self.id);
		buf.extend_from_slice(&self.code.to_be_bytes());
		buf.extend_from_slice(&self.req.to_be_bytes());
		buf.extend_from_slice(&self.sample.to_be_bytes());
		buf.extend_from_slice(&self.size.to_be_bytes());
		buf.extend_from_slice(&self.size_uncompressed.to_be_bytes());
		if let Some(b) = self.body {
			buf.extend_from_slice(b);
		}
		buf
	}
}

#[cfg(test)]
mod tests {

	use super::*;
	#[test]
	fn pkg_to_bytes() {
		let p = Packet {
			id: [69, 69, 4, 20],
			code: 69,
			req: 420,
			sample: 42069,
			size: 69,
			size_uncompressed: 420,
			body: None,
		};
		println!("{:?} {}", p.to_bytes(), Packet::SIZE);
	}
}
