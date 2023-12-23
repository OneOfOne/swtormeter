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

pub mod utils;

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
		let mut heals = 0.;
		for line in String::from_utf8_lossy(&f).lines() {
			if let Some(l) = Line::new(line) {
				// dbg!(l.value);
				if l.action.effect.name == "EnterCombat" {
					cmpt = true;
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
					println!("{} {} {}", heals, end.sub(start), heals / secs);
					heals = 0.;
					continue;
				}
				if !cmpt {
					continue;
				}
				let value = l.value;
				let (typ, src, val) = (value.typ, l.source.name, value);
				if val.tilde != 0 {
					// println!("{:?}", l.clone());
				}
				if !src.contains("Locu") {
					continue;
				}
				match typ {
					ValueType::Heal(v) => {
						heals += v as f64;
					}
					_ => {}
				}
				//println!("{}", line);
			}
		}
		println!("{}", end.sub(start).num_seconds());
		// dbg!("x", "3503452067987731".parse::<u64>().ok());
		// let result = Line::new("[19:46:01.686] [@Nyx'ayuna#686862584797878|(-109.03,-630.11,-63.40,108.12)|(263258/380355)] [=] [Berserk {4056205769048064}] [ApplyEffect {836045448945477}: Heal {836045448945500}] (3955) <1977>") ;
		// dbg!(result);
		// let result = Line::new("[19:52:17.535] [Toth {2857549116211200}:52422000225810|(-103.70,-609.83,-63.40,-155.69)|(5430688/13451706)] [@Recency#689265319653227|(-101.50,-604.66,-62.68,21.52)|(309708/457173)] [Backhand Smash {2857321482944512}] [ApplyEffect {836045448945477}: Damage {836045448945501}] (5289 kinetic {836045448940873} -shield {836045448945509} (74525 absorbed {836045448945511})) <5289>") ;
		// dbg!(result);
		// let result = Line::new("[19:52:17.948] [@Locutus'of#690037831467479|(-97.30,-622.67,-63.39,165.31)|(140782/428340)] [@Nyx'ayuna#686862584797878|(-102.09,-616.61,-63.40,166.80)|(225043/380355)] [Salvation {812990064492544}] [ApplyEffect {836045448945477}: Heal {836045448945500}] (3075*) <1384>") ;
		// dbg!(result);
	}
}
