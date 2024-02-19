use densky_adapter::{ErrorContext, Result};
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Generate an import statement with version hash for prevent import caching.
/// The hash is only used with relative imports.
pub fn import<T: Display, F: Display>(t: T, filename: F) -> String {
    format!("import {t} from \"{filename}\";")
}

/// Generate a filename with cache hash.
/// Note: Don't use quotes, the output is clean
pub fn import_filename<F: Display>(filename: F) -> String {
    filename.to_string()
}

pub fn discover_file(attempts: Vec<impl AsRef<Path>>) -> Result<(PathBuf, File)> {
    todo!()
}

pub fn read_file(mut file: File, path: impl Display) -> Result<String> {
    let mut config_file_contents = String::new();
    file.read_to_string(&mut config_file_contents)
        .with_context(|| densky_adapter::anyhow!("Can't read {path}."))?;

    Ok(config_file_contents)
}
