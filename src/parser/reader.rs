use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::io::SeekFrom;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncSeekExt, BufReader};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::yield_now;
use tokio::time::sleep;

use super::{Encounter, Encounters, Line};

type INotifyRes = Result<Event>;
type StdReceiver = std::sync::mpsc::Receiver<INotifyRes>;

#[derive(Debug, Clone)]
pub struct Reader {
	tx: Sender<Line>,
}

impl Reader {
	pub fn new(tx: Sender<Line>) -> Self {
		Self { tx }
	}

	pub async fn load(&mut self, fp: &str) -> Result<()> {
		let mut f = File::open(fp).await?;
		_ = f.seek(SeekFrom::End(0)).await;
		let (wtx, wrx) = std::sync::mpsc::channel();
		let mut watcher = RecommendedWatcher::new(wtx, Config::default())?;
		watcher.watch(fp.as_ref(), RecursiveMode::NonRecursive)?;

		//self.f.replace(Arc::new(f));
		let mut s = self.clone();
		tokio::spawn(async move {
			_ = watcher;
			s.process(f, wrx).await;
		});
		Ok(())
	}

	async fn process(&mut self, f: File, wrx: StdReceiver) {
		let mut buf = Vec::with_capacity(1024);
		loop {
			let f = f.try_clone().await.unwrap();
			let mut rd = BufReader::new(f);
			while let Ok(ln) = rd.read_until(b'\n', &mut buf).await {
				if ln == 0 {
					break;
				}
				// the og uses weird encoding
				//
				yield_now().await;
				let s: &String = &buf.iter().map(|&c| c as char).collect();
				// if let Some(l) = Line::new(s.trim()) {
				// 	self.tx.send(l).await.unwrap();
				// }
				if !s.ends_with('\n') {
					break;
				}
				for ss in s.trim().lines() {
					if let Some(l) = Line::new(ss.trim()) {
						self.tx.send(l).await.unwrap();
					}
				}
				buf.clear();
				// if let Some(l) = Line::new(s.trim()) {
				// 	self.tx.send(l).await.unwrap();
				// }
			}
			sleep(Duration::from_millis(250)).await;
		}
	}
}

pub async fn tail_file<'a>(path: &'a str) -> Result<Receiver<Line>> {
	let (tx, mut rx) = channel::<Line>(8);
	let mut rd = Reader::new(tx);
	rd.load(path).await.unwrap();

	let mut enc = Encounters::default();
	while let Some(l) = rx.recv().await {
		if let Some(enc) = enc.append(l.clone()) {
			print!("{esc}c", esc = 27 as char);
			println!("----- healing -----\n");
			let mut vec = enc.heal.iter().collect::<Vec<_>>();
			vec.sort_by(|(_, &ref a), (_, &ref b)| b.xps.total_cmp(&a.xps));

			for (k, v) in vec {
				println!("{:40} | {:?}", k, &v);
			}

			println!("\n----- damage -----\n");
			let mut vec = enc.dmg.iter().collect::<Vec<_>>();
			vec.sort_by(|(_, &ref a), (_, &ref b)| b.xps.total_cmp(&a.xps));

			for (k, v) in vec {
				println!("{:40} | {:?}", k, &v);
			}

			//println!("\n{}: {:?}", l.ability.name, l.value);
		}
	}
	Ok(rx)
}
