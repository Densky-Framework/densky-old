use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use pathdiff::diff_paths;
use walkdir::WalkDir;

use crate::{utils::join_paths, CompileContext};

use super::{container::WalkerContainer, WalkerLeaf, WalkerTree};

#[derive(Debug)]
pub enum WalkerDiscoverError {}

pub fn walker_tree_discover<F, R>(
    folder_name: F,
    input_path: R,
    ctx: &CompileContext,
) -> (WalkerContainer, Arc<Mutex<WalkerTree>>)
where
    F: AsRef<Path>,
    R: AsRef<Path>,
{
    let output_dir = join_paths(folder_name, &ctx.output_dir);

    let mut container = WalkerContainer::new(output_dir.clone());
    let root = container.create_root();

    let walk_dir = WalkDir::new(&input_path).into_iter().filter_map(Result::ok);
    for entry in walk_dir {
        let file_path = entry.path();

        if let Some(ext) = file_path.extension() {
            if ext != "ts" {
                continue;
            }
        } else {
            continue;
        }

        let relative = match diff_paths(file_path, &input_path) {
            Some(path) => path,
            None => continue,
        };
        let path = "/".to_string() + &relative.with_extension("").display().to_string();
        let file_path = file_path.display().to_string();
        let output_path = join_paths(relative, &output_dir);

        let leaf = WalkerLeaf::new(path, file_path, output_path);
        let leaf = container.add_leaf(leaf);
        root.lock().unwrap().add_child(leaf, &mut container);
    }

    (container, root)
}

pub fn simple_discover<F, R>(
    folder_name: F,
    input_path: R,
    ctx: &CompileContext,
) -> impl Iterator<Item = Option<WalkerLeaf>>
where
    F: AsRef<Path>,
    R: AsRef<Path>,
{
    let output_dir = join_paths(folder_name, &ctx.output_dir);

    WalkDir::new(&input_path).into_iter().map(move |result| {
        let file = match result {
            Ok(file) => file,
            Err(_) => return None,
        };

        let file_type = file.file_type();
        if file_type.is_dir() {
            return None;
        }

        let entry = file.path();

        let relative = match diff_paths(&entry, &input_path) {
            Some(path) => path,
            None => return None,
        };
        let path = relative.with_extension("").display().to_string();
        let file_path = entry.display().to_string();
        let output_path = join_paths(relative, &output_dir);

        Some(WalkerLeaf::new(path, file_path, output_path))
    })
}
