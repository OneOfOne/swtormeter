use notify::event::{AccessKind, AccessMode};
use notify::{
	Config, Event, EventKind, INotifyWatcher, RecommendedWatcher, RecursiveMode, Result, Watcher,
};
use std::borrow::{Borrow, BorrowMut};
use std::fs::File;
use std::io::{self, BufRead, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::yield_now;

type INotifyRes = Result<Event>;
type StdReceiver = std::sync::mpsc::Receiver<INotifyRes>;

#[derive(Debug, Clone)]
pub struct Reader {
	tx: Sender<String>,
}

impl Reader {
	pub fn new(tx: Sender<String>) -> Self {
		Self { tx }
	}

	pub async fn load(&mut self, fp: &str) -> Result<()> {
		let f = File::open(fp)?;
		let (wtx, wrx) = std::sync::mpsc::channel();
		let mut watcher = RecommendedWatcher::new(wtx, Config::default())?;
		watcher.watch(fp.as_ref(), RecursiveMode::NonRecursive)?;

		//self.f.replace(Arc::new(f));
		let mut s = self.clone();
		tokio::spawn(async move {
			let w = watcher;
			s.process(f, wrx).await;
		});
		Ok(())
	}

	async fn process(&mut self, f: File, wrx: StdReceiver) {
		let mut f = &f;
		let mut buf = Vec::with_capacity(1024);

		'out: loop {
			let mut rd = io::BufReader::new(f);
			while let Ok(ln) = rd.read_until(b'\n', &mut buf) {
				if ln == 0 {
					break;
				}
				self.tx
					.send(String::from_utf8_lossy(&buf[..ln]).trim_end().to_string())
					.await
					.unwrap();
				buf.clear();
				yield_now().await;
			}

			let mut pos: u64 = f.metadata().unwrap().len();
			for res in &wrx {
				dbg!(&res);
				match res {
					Ok(Event {
						kind: EventKind::Access(AccessKind::Close(AccessMode::Write)),
						..
					}) => {
						let npos = f.metadata().unwrap().len();
						if npos == pos {
							continue;
						}

						continue 'out;
					}
					Err(error) => println!("{error:?}"),
					_ => {}
				}
			}
		}
	}
}

pub async fn tail_file(path: &str) -> Result<Receiver<String>> {
	let (tx, mut rx) = channel::<String>(128);
	let mut rd = Reader::new(tx);
	rd.load(path).await.unwrap();
	while let Some(v) = rx.recv().await {
		println!("x {}", v);
	}
	// let mut f = File::open(path)?;
	// let mut pos = f.metadata()?.len();
	// // set up watcher
	// let (tx, rx) = std::sync::mpsc::channel();
	// let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
	// watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
	//
	// tokio::spawn(async move {
	// 	let _watcher = watcher;
	// 	let f = f;
	// 	let mut send_buf = async |init: bool, f: &File| {
	// 		if !init {
	// 			let npos = f.metadata().unwrap().len();
	// 			if npos == pos.clone() {
	// 				return;
	// 			}
	// 			pos = npos;
	//
	// 			f.seek(SeekFrom::Start(pos + 1)).unwrap();
	// 		}
	//
	// 		let mut rd = io::BufReader::new(f);
	// 		let mut buf = vec![];
	// 		loop {
	// 			if let Ok(ln) = rd.read_until(b'\n', &mut buf) {
	// 				if ln == 0 {
	// 					break;
	// 				}
	// 				// ln -2 because the lines end with \r\n
	// 				rtx.send(String::from_utf8_lossy(&buf[..ln - 2]).to_string())
	// 					.await
	// 					.unwrap();
	// 				buf.clear();//
	// 			} else {
	// 				break;
	// 			}
	// 		}
	// 	};
	//
	// 	send_buf(true, &f);
	//
	// 	for res in rx {
	// 		match res {
	// 			Ok(_event) => send_buf(false, &f).await,
	// 			Err(error) => println!("{error:?}"),
	// 		}
	// 	}
	// });
	Ok(rx)
}
