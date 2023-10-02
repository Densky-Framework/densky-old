use std::path::{Path, PathBuf};

use super::WalkerEntity;

/// Endpoint of the walker tree
#[derive(Debug, Clone)]
pub struct WalkerLeaf {
    id: usize,
    /// The absolute path (url) for this leaf
    pub path: String,
    /// The path (url) relative to parent.
    pub rel_path: String,

    /// The path (fs) to the current file
    pub file_path: PathBuf,
    /// The path (fs) to the output file
    pub output_path: PathBuf,

    pub owner: usize,

    pub content: Option<String>,
}

impl WalkerEntity for WalkerLeaf {
    fn get_id(&self) -> usize {
        self.id
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}

impl WalkerLeaf {
    pub fn new<F, O>(path: String, file_path: F, output_path: O) -> WalkerLeaf
    where
        F: AsRef<Path>,
        O: AsRef<Path>,
    {
        WalkerLeaf {
            id: 0,
            path,
            rel_path: String::new(),
            file_path: file_path.as_ref().to_path_buf(),
            output_path: output_path.as_ref().to_path_buf(),
            owner: 0,
            content: None,
        }
    }
}
