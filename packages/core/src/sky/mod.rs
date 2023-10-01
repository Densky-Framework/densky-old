mod plugin;

pub use self::plugin::{CloudPlugin, CloudPluginError};

use std::{ffi::OsStr, path::Path};

use libloading::{library_filename, Library};

pub unsafe fn open_cloud<P>(libname: impl AsRef<OsStr>, path: P) -> Library
where
    P: AsRef<Path>,
{
    Library::new(Path::join(path.as_ref(), library_filename(libname.as_ref())))
        .expect("Dynamic libraries are not supported by your system, please open us an issue and tell us all the context with your system")
}

#[macro_export]
macro_rules! try_call {
    ($self:ident ( $($args:tt)* )) => {
        if let Some($self) = $self {
            Some($self($($args)*))
        } else {
            None
        }
    };
}

pub use try_call;
