use notify::{Config, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::fs::File;
use std::io::{self, BufRead, Seek, SeekFrom};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

pub fn tail_file(path: &str) -> Result<Receiver<String>> {
	let (rtx, rrx) = channel::<String>();
	let mut f = File::open(path)?;
	let mut pos = f.metadata()?.len();
	// set up watcher
	let (tx, rx) = channel();
	let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
	watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

	thread::spawn(move || {
		let _watcher = watcher;

		let mut send_buf = |init: bool| {
			if !init {
				let npos = f.metadata().unwrap().len();
				if npos == pos {
					return;
				}
				pos = npos;

				f.seek(SeekFrom::Start(pos + 1)).unwrap();
			}

			let mut rd = io::BufReader::new(&f);
			let mut buf = vec![];
			loop {
				if let Ok(ln) = rd.read_until(b'\n', &mut buf) {
					if ln == 0 {
						break;
					}
					// ln -2 because the lines end with \r\n
					rtx.send(String::from_utf8_lossy(&buf[..ln - 2]).to_string())
						.unwrap();
					buf.clear();
				} else {
					break;
				}
			}
		};

		send_buf(true);

		for res in rx {
			match res {
				Ok(_event) => send_buf(false),
				Err(error) => println!("{error:?}"),
			}
		}
	});
	Ok(rrx)
}
