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

#[cfg(test)]
mod tests {

	use chrono::NaiveTime;
	use std::ops::Sub;

	use super::*;
	#[tokio::test]
	async fn metadata_test() {
		// let f = fs::read("/home/oneofone/code/rust/swtormeter/log.txt").unwrap();
		let mut cmpt = false;
		let mut start: NaiveTime = NaiveTime::MIN;
		let mut end: NaiveTime = NaiveTime::MIN;
		let mut heals = 0i32;
		let mut count = 0;
		let _tx = reader::tail_file("/games/Steam/steamapps/compatdata/1286830/pfx/drive_c/users/steamuser/Documents/Star Wars - The Old Republic/CombatLogs/combat_2023-12-25_13_33_43_126195.txt")
		//let mut tx = reader::tail_file("/home/oneofone/code/rust/swtormeter/log.txt")
			.await
			.unwrap();

		// for l in tx.recv().await {
		//dbg!(&line);
		// dbg!(l.value);
		// if l.action.effect.name == "EnterCombat" {
		// 	cmpt = true;
		// 	heals = 0;
		// 	count = 0;
		// 	start = l.ts;
		//
		// 	continue;
		// }
		// if l.action.effect.name == "ExitCombat" {
		// 	cmpt = false;
		// 	end = l.ts;
		// 	// dbg!(l.action, heals);
		// 	let secs = end.sub(start).num_seconds() as f64;
		//
		// 	if secs < 50. {
		// 		continue;
		// 	}
		// 	println!(
		// 		"{} {} {} {}",
		// 		heals,
		// 		count,
		// 		end.sub(start),
		// 		heals as f64 / secs
		// 	);
		// 	continue;
		// }
		// if !cmpt {
		// 	continue;
		// }
		// let (src, val) = (l.source.name, l.value);
		// if !src.contains("Locu") {
		// 	continue;
		// }
		// match val {
		// 	Value::Heal {
		// 		total: t,
		// 		effective: e,
		// 		..
		// 	} => {
		// 		count += 1;
		// 		heals += if e > 0 { e } else { t };
		// 	}
		// 	_ => {}
		// }
		// }
	}
}
