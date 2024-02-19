mod plugin;

pub use self::plugin::{CloudPlugin, CloudPluginError};

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use densky_adapter::{log::PathDebugDisplay, log_warn, Result};
use libloading::{library_filename, Error, Library};

pub unsafe fn open_cloud<P>(libname: impl AsRef<OsStr>, path: P) -> Result<Library, Error>
where
    P: AsRef<Path>,
{
    Library::new(Path::join(path.as_ref(), library_filename(libname.as_ref())))
        .map_err(|i| {
            eprintln!("Dynamic libraries are not supported by your system or the file doesn't exists. Please open us an issue and tell us all the context with your system");
            i
        })
}

pub fn search_cloud(libname: impl AsRef<str>, entries: &[PathBuf]) -> Option<PathBuf> {
    let libname = libname.as_ref();

    for entry in entries {
        let Some(read_dir) = fs::read_dir(entry).ok() else {
            continue;
        };
        for item in read_dir.filter_map(Result::ok) {
            let Ok(file_type) = item.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }

            let file_path = item.path();
            let Some(file_name) = file_path.file_name() else {
                log_warn!(["CLOUD"] "Can't get file name from {}", file_path.display_debug());
                continue;
            };

            let Some(file_name) = file_name.to_str() else {
                continue;
            };

            if file_name.contains(libname) {
                return Some(item.path());
            }
        }
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;
    use densky_adapter::log::PathDebugDisplay;

    #[test]
    fn search_cloud_test() {
        let cwd = std::env::current_dir().unwrap();
        let cwd = cwd.parent().unwrap();
        let cwd = cwd.parent().unwrap();

        let args = std::env::args().skip(3);

        let entries = args
            .map(|a| format!("{}/{a}", cwd.display()).into())
            .collect::<Vec<PathBuf>>();
        println!("{entries:?}");

        let cloud_path = search_cloud("http-router", &entries);
        println!("{}", cloud_path.display_debug());
    }
}
