extern crate ahash;
extern crate anyhow;
extern crate pathdiff;
pub extern crate semver;
pub extern crate thiserror;

pub use ahash::{AHashMap, AHashSet};
pub use anyhow::{anyhow, Context as ErrorContext, Error, Result};

mod calls;
pub mod context;
pub mod macros;
pub mod utils;

pub use calls::*;
pub use utils::log;

pub struct CompileContext {
    pub output_dir: String,
    pub cwd: String,
    pub verbose: bool,
}
