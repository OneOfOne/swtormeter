use notify::event::{AccessKind, AccessMode};
use notify::{
	Config, Event, EventKind, INotifyWatcher, RecommendedWatcher, RecursiveMode, Result, Watcher,
};
use std::borrow::{Borrow, BorrowMut};
use std::fs::File;
use std::io::{self, BufRead, Seek, SeekFrom};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::yield_now;

use super::{Encounter, Line};

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
				let b: String = unsafe { String::from_utf8_unchecked(buf.clone()).trim().into() };
				if let Some(l) = Line::new(&b) {
					self.tx.send(l).await.unwrap();
				}
				buf.clear();
				yield_now().await;
			}

			let pos = f.metadata().unwrap().len();
			for res in &wrx {
				//dbg!(&res);
				match res {
					Ok(_) => {
						let npos = f.metadata().unwrap().len();
						if npos == pos {
							continue;
						}

						continue 'out;
					}
					Err(error) => println!("{error:?}"),
				}
			}
		}
	}
}
fn blah(s: Rc<&str>) {}
pub async fn tail_file<'a>(path: &'a str) -> Result<Receiver<Line>> {
	let (tx, mut rx) = channel::<Line>(8);
	let path = path.clone();
	let mut rd = Reader::new(tx);
	rd.load(path).await.unwrap();

	let mut enc = Encounter::new();
	while let Some(l) = rx.recv().await {
		if let Some(m) = enc.append(l.clone()) {
			print!("{esc}c", esc = 27 as char);
			for (k, v) in &enc.heal {
				println!("x {} {:?}", k, &v);
			}
			println!("\n{}: {:?}", l.ability.name, l.value);
		}
	}
	Ok(rx)
}
