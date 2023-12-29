use std::{fs::read_dir, path::Path};

use chrono::NaiveDateTime;
use tokio::sync::mpsc::Receiver;

pub mod action;
use action::*;

pub mod actor;
use actor::*;
pub mod consts;
use consts::*;

pub mod encounter;
use encounter::*;

pub mod line;
use line::*;

pub mod metadata;
use metadata::*;

pub mod reader;
use reader::*;

pub mod value;
use value::*;

pub mod utils;

pub async fn parse<F: Fn(&Encounter)>(dir: &str, process: F) -> std::io::Result<()> {
	let paths = read_dir(dir)?;
	let mut paths: Vec<_> = paths
		.map(|p| p.unwrap().path().display().to_string())
		.collect();
	paths.sort();

	let path = paths.get(paths.len() - 1).unwrap();
	let name = Path::new(&path).file_name().unwrap().to_str().unwrap();
	// println!("loaded {}", name);

	let mut rx = Reader::parse(path.as_str()).await?;
	let mut enc = Encounters::new(name);
	enc.process(&mut rx, process).await;
	Ok(())
}

pub fn logs_path() -> Option<String> {
	let dir = if let Ok(dir) = std::env::var("LOGS_PATH") {
		dir
	} else {
		std::env::var("HOME").unwrap() +
		"/.local/share/Steam/steamapps/compatdata/1286830/pfx/drive_c/users/steamuser/Documents/Star Wars - The Old Republic/CombatLogs/"
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
		parse(logs_path().unwrap().as_str(), |enc| {
			//print!("{esc}c{esc}c", esc = 27 as char);
			println!("area: {}\n", enc.area);
			println!(
				"npcs: {}\n",
				enc.npcs.clone().drain().collect::<Vec<String>>().join(", ")
			);
			println!("----- hps -----\n");

			for v in enc.heal.clone() {
				println!("{}", &v);
			}

			println!("\n----- dps -----\n");

			for v in enc.dmg.clone() {
				println!("{}", &v);
			}
		})
		.await
		.unwrap();
	}
}
