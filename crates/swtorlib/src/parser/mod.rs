use tokio::sync::mpsc::Receiver;

pub mod action;
use action::*;

pub mod actor;
use actor::*;

pub mod encounter;
use encounter::*;

pub mod line;
use line::*;

pub mod namedid;
use namedid::*;

pub mod actor_stats;

pub mod consts;
pub mod reader;
pub mod utils;

pub async fn parse<F: Fn(&Encounter, &Line)>(dir: &str, process: F) -> std::io::Result<()> {
	let mut rx = reader::Reader::process_dir(dir).await.unwrap();
	let mut enc = Encounters::new();
	let h = enc.process(&mut rx, process).await;
	Ok(h)
}

pub static BASE_COMBATLOGS_DIR: &str = "/Documents/Star Wars - The Old Republic/CombatLogs/";

pub fn logs_path() -> Option<String> {
	let home = dirs_next::home_dir().unwrap().display().to_string();
	let dir = if let Ok(dir) = std::env::var("LOGS_PATH") {
		dir
	} else if home.starts_with('/') {
		home + "/.local/share/Steam/steamapps/compatdata/1286830/pfx/drive_c/users/steamuser"
			+ BASE_COMBATLOGS_DIR
	} else {
		home + BASE_COMBATLOGS_DIR
	};

	if let Ok(m) = std::fs::metadata(&dir) {
		if m.is_dir() {
			return Some(dir);
		}
	}

	None
}

#[cfg(test)]
mod tests {

	use super::*;
	#[tokio::test]
	async fn parse_test() {
		dbg!(logs_path());
		parse(logs_path().unwrap().as_str(), |enc, l| {
			//print!("{esc}c{esc}c", esc = 27 as char);
			println!("area: {}", enc.area);
			println!("line: {:?}", l);
			//println!("{:?}", enc.heals_in());
			// println!(
			// 	"npcs: {}\n",
			// 	enc.npcs.clone().drain().collect::<Vec<String>>().join(", ")
			// );
			// println!("----- hps -----\n");
			//
			// for v in enc.heal.clone() {
			// 	println!("{}", &v);
			// }
			//
			// println!("\n----- dps -----\n");
			//
			// for v in enc.dmg.clone() {
			// 	println!("{}", &v);
			// }
		})
		.await
		.unwrap();
	}
}
