use chrono::NaiveDateTime;
use tokio::sync::Mutex;

use std::fs::read_dir;
use std::io::{Result, SeekFrom};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::sleep;

use super::Line;

pub struct Reader;
impl Reader {
	pub async fn process_dir(dir: &str) -> Result<Receiver<Line>> {
		//let name = &fp[fp.find("combat_").unwrap() + 7..fp.find(".txt").unwrap()];

		let (tx, rx) = channel::<Line>(8);
		// let mut f = File::open(fp).await?;
		// if seek_to_end {
		// 	_ = f.seek(SeekFrom::End(0)).await?;
		// }

		//self.f.replace(Arc::new(f));

		let dir = dir.to_owned();
		let fpath: Arc<Mutex<String>> = Arc::default();
		let mut fp = fpath.clone();
		tokio::spawn(async move { Self::watch_dir(dir.to_owned(), fp).await });
		let mut fp = fpath.clone();
		tokio::spawn(async move { Self::process(tx, fp).await });

		Ok(rx)
	}

	async fn watch_dir(dir: String, ff: Arc<Mutex<String>>) {
		loop {
			{
				let (fp, _) = latest_log(dir.as_str()).unwrap();
				*ff.lock().await = fp;
			}
			sleep(Duration::from_secs(10)).await
		}
	}

	async fn process(tx: Sender<Line>, ff: Arc<Mutex<String>>) {
		let mut buf = Vec::with_capacity(1024);
		let mut fname: Option<String> = None;
		let mut f: Option<File> = None;
		loop {
			{
				if let fp = ff.lock().await {
					if !fp.is_empty() {
						let fp = fp.clone();
						if fname.is_none() || fname.clone().unwrap() != fp {
							fname = Some(fp.clone());
							f.replace(File::open(fp.clone()).await.unwrap());
						}
					}
				}
				if f.is_none() {
					sleep(Duration::from_secs(1)).await;
					continue;
				}
			}

			let ff = f.take().unwrap();
			let ffc = ff.try_clone().await.unwrap();

			let mut rd = BufReader::new(ffc);
			while let Ok(ln) = rd.read_until(b'\n', &mut buf).await {
				if ln == 0 {
					break;
				}
				// the log uses a weird encoding
				let s: String = buf.iter().map(|&c| c as char).collect();
				if !s.ends_with('\n') {
					break;
				}
				let s = &s.clone();
				for ss in s.trim().lines() {
					if let Some(l) = Line::new(ss.trim()) {
						tx.send(l).await.unwrap();
					}
				}
				buf.clear();
			}
			sleep(Duration::from_millis(500)).await
		}
	}
}

pub fn latest_log(dir: &str) -> std::io::Result<(String, String)> {
	let paths = read_dir(dir)?;
	let mut paths: Vec<_> = paths
		.map(|p| p.unwrap().path().display().to_string())
		.collect();
	paths.sort();

	let path = paths.get(paths.len() - 2).unwrap();
	let name = Path::new(&path).file_name().unwrap().to_str().unwrap();

	Ok((path.to_owned(), name.to_owned()))
}
