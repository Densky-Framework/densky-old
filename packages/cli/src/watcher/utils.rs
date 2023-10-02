use std::{fs, io, path::PathBuf};

use ahash::HashMap;

use super::PollWatcher;

#[inline(always)]
pub fn walk_dir(cwd: &PathBuf) -> io::Result<HashMap<PathBuf, u64>> {
    let mut files = HashMap::default();
    let dir = DirIterator::new(fs::read_dir(cwd)?);

    for entry in dir.into_iter() {
        let entry = entry.path();

        if PollWatcher::is_valid_filename(&entry) {
            let hash = PollWatcher::get_hash(&entry.clone());
            files.insert(entry, hash);
        }
    }

    Ok(files)
}

pub struct DirIterator {
    stack: Vec<fs::DirEntry>,
    current: fs::ReadDir,
}

impl DirIterator {
    pub fn new(dir: fs::ReadDir) -> DirIterator {
        DirIterator {
            stack: Vec::new(),
            current: dir,
        }
    }
}

impl Iterator for DirIterator {
    type Item = fs::DirEntry;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(entry) = self.current.next().and_then(|f| f.ok()) {
                if entry.file_type().ok()?.is_dir() {
                    self.stack.push(entry);
                } else {
                    return Some(entry);
                }
            } else {
                let dir = self.stack.pop()?;
                self.current = fs::read_dir(dir.path()).ok()?;
            }
        }
    }
}
