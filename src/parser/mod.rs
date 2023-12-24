mod consts;
pub use consts::*;

mod metadata;
pub use metadata::*;

mod action;
pub use action::*;

mod actor;
pub use actor::*;

mod value;
pub use value::*;

mod line;
pub use line::*;

pub mod encounter;
pub use encounter::*;

pub mod reader;
pub mod utils;

pub fn parse<'a, It>(it: It)
where
	It: IntoIterator<Item = &'a str>,
{
	let mut it = it.into_iter();
	dbg!(it.next().unwrap());
}

#[cfg(test)]
mod tests {

	use chrono::NaiveTime;
	use std::{fs, ops::Sub};

	use super::*;
	#[test]
	fn metadata_test() {
		let f = fs::read("/home/oneofone/code/rust/swtormeter/log.txt").unwrap();
		let mut cmpt = false;
		let mut start: NaiveTime = NaiveTime::MIN;
		let mut end: NaiveTime = NaiveTime::MIN;
		let mut heals = 0i32;
		let mut count = 0;
		let tx = reader::tail_file("/home/oneofone/code/rust/swtormeter/log.txt").unwrap();

		for line in tx {
			//dbg!(&line);
			if let Some(l) = Line::new(line.as_str()) {
				// dbg!(l.value);
				if l.action.effect.name == "EnterCombat" {
					cmpt = true;
					heals = 0;
					count = 0;
					start = l.ts;
					continue;
				}
				if l.action.effect.name == "ExitCombat" {
					cmpt = false;
					end = l.ts;
					// dbg!(l.action, heals);
					let secs = end.sub(start).num_seconds() as f64;

					if secs < 50. {
						continue;
					}
					println!(
						"{} {} {} {}",
						heals,
						count,
						end.sub(start),
						heals as f64 / secs
					);
					continue;
				}
				if !cmpt {
					continue;
				}
				let (src, val) = (l.source.name, l.value);
				if !src.contains("Locu") {
					continue;
				}
				match val {
					Value::Heal {
						total: t,
						effective: e,
						..
					} => {
						count += 1;
						heals += if e > 0 { e } else { t };
					}
					_ => {}
				}
			}
		}
	}
}
