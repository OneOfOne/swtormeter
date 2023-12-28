use chrono::NaiveDateTime;
use std::io::SeekFrom;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::yield_now;
use tokio::time::sleep;

use super::Line;

#[derive(Debug, Clone)]
pub struct Reader;

impl Reader {
	pub async fn parse(fp: &str) -> std::io::Result<Receiver<Line>> {
		let name = &fp[fp.find("combat_").unwrap() + 7..fp.find(".txt").unwrap()];
		let (start, _) = NaiveDateTime::parse_and_remainder(name, "%Y-%m-%d_%H_%M").unwrap();
		dbg!(start);

		let (tx, rx) = channel::<Line>(8);
		let mut f = File::open(fp).await?;
		//	_ = f.seek(SeekFrom::End(0)).await?;

		//self.f.replace(Arc::new(f));
		tokio::spawn(Self::process(tx, f));

		Ok(rx)
	}

	async fn process(tx: Sender<Line>, f: File) {
		let mut buf = Vec::with_capacity(1024);
		loop {
			let f = f.try_clone().await.unwrap();
			let mut rd = BufReader::new(f);
			while let Ok(ln) = rd.read_until(b'\n', &mut buf).await {
				if ln == 0 {
					break;
				}
				// the log uses a weird encoding
				let s: &String = &buf.iter().map(|&c| c as char).collect();
				if !s.ends_with('\n') {
					break;
				}
				for ss in s.trim().lines() {
					if let Some(l) = Line::new(ss.trim()) {
						tx.send(l).await.unwrap();
					}
				}
				buf.clear();
			}
			sleep(Duration::from_millis(500)).await;
		}
	}
}
