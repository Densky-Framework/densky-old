mod plugin;

pub use self::plugin::{CloudPlugin, CloudPluginError};

use std::{ffi::OsStr, path::Path};

use densky_adapter::Result;
use libloading::{library_filename, Error, Library};

pub unsafe fn open_cloud<P>(libname: impl AsRef<OsStr>, path: P) -> Result<Library, Error>
where
    P: AsRef<Path>,
{
    Library::new(Path::join(path.as_ref(), library_filename(libname.as_ref())))
        .map_err(|i| {
            eprintln!("Dynamic libraries are not supported by your system, please open us an issue and tell us all the context with your system");
            i
        })
}
