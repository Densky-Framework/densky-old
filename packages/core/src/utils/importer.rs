use std::fmt::Display;

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
