use std::cell::RefCell;
use std::fs;
use std::io;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use ahash::{HashMap, HashSet};

use super::utils::{walk_dir, DirIterator};
use super::MAIN_HASHER;

#[derive(Debug, Clone)]
pub enum WatchKind {
    Create,
    Remove,
    Modify,
}

static MEGABYTE: u64 = 1000000;
static FILE_SIZE_THRESHOLD: u64 = MEGABYTE * 10;

#[derive(Debug)]
pub struct WatchEvent {
    pub kind: WatchKind,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct PollWatcher {
    will_drop: bool,
    folder: PathBuf,
    files: RefCell<HashMap<PathBuf, u64>>,
}

impl PollWatcher {
    pub fn new(folder: PathBuf) -> io::Result<PollWatcher> {
        let files = walk_dir(&folder)?;

        Ok(PollWatcher {
            will_drop: false,
            folder,
            files: RefCell::new(files),
        })
    }

    pub fn get_hash_with_size(path: &PathBuf, size: u64) -> u64 {
        if size >= FILE_SIZE_THRESHOLD {
            size
        } else {
            let c = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => return 0,
            };
            MAIN_HASHER.with(|m| m.hash_one(c))
        }
    }

    pub fn get_hash(path: &PathBuf) -> u64 {
        let size = match fs::metadata(&path) {
            #[cfg(unix)]
            Ok(m) => m.size(),

            #[cfg(windows)]
            Ok(m) => m.file_size(),

            #[cfg(not(any(unix, windows)))]
            Ok(_) => 0,

            Err(_) => 0,
        };

        Self::get_hash_with_size(path, size)
    }

    pub fn poll(&mut self) -> Vec<WatchEvent> {
        let a = fs::read_dir(&self.folder).unwrap();
        let a = DirIterator::new(a);

        let removed_files = self
            .files
            .borrow()
            .iter()
            .map(|(path, _)| path.clone())
            .collect();
        let mut removed_files: HashSet<PathBuf> = HashSet::from(removed_files);

        let mut events = Vec::new();
        for entry in a {
            let path = entry.path();
            if !Self::is_valid_filename(&path) {
                continue;
            }

            let file_hash = Self::get_hash(&path);

            removed_files.remove(&path);

            let mut files = self.files.borrow_mut();
            if let Some(old_hash) = files.get(&path) {
                if *old_hash != file_hash {
                    files.insert(path.clone(), file_hash);
                    events.push(WatchEvent {
                        kind: WatchKind::Modify,
                        path,
                    });
                }
            } else {
                files.insert(path.clone(), file_hash);
                events.push(WatchEvent {
                    kind: WatchKind::Create,
                    path,
                });
            }
        }

        for entry in removed_files {
            self.files.borrow_mut().remove(&entry);
            events.push(WatchEvent {
                kind: WatchKind::Remove,
                path: entry.into(),
            });
        }

        events
    }

    pub fn is_valid_filename(path: &PathBuf) -> bool {
        let is_nvim_cache = path.display().to_string().ends_with('~');
        let is_nvim_file = path.ends_with("4913");
        let is_output = path
            .components()
            .find(|c| c.as_os_str() == ".densky")
            .is_some();

        !is_nvim_cache && !is_nvim_file && !is_output
    }

    pub fn scheduling_poll<F>(&mut self, interval: Duration, rx: Receiver<()>, mut f: F)
    where
        F: FnMut(Vec<WatchEvent>),
    {
        let mut time_elapsed = Duration::ZERO;

        loop {
            let time = Instant::now();

            if let Ok(_) = rx.recv_timeout(Duration::from_millis(1)) {
                println!("[PollWatcher] Shutdown");
                break;
            }

            if time_elapsed >= interval {
                time_elapsed = Duration::ZERO;

                let r = self.poll();
                if r.len() != 0 {
                    f(r);
                }
            }

            let elapsed = time.elapsed();
            let sleep_time = 50u64.wrapping_sub(elapsed.as_millis() as u64);
            thread::sleep(Duration::from_millis(sleep_time as u64));
            time_elapsed += time.elapsed();
        }
    }
}

impl Drop for PollWatcher {
    fn drop(&mut self) {
        self.will_drop = true;
    }
}
