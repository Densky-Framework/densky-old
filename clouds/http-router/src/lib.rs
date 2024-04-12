#[macro_use]
extern crate densky_adapter;

pub mod context;
mod file_resolve;
mod manifest;
mod manifest_process;

pub use file_resolve::cloud_file_resolve;
pub use manifest_process::{cloud_before_manifest, cloud_manifest};

cloud_setup!(http::router {
    source_folder: "http",
    file_ends: ".ts",
    file_strategy: OptimizedTree,
    dependencies: [
        database::orm =>? "^1.0.0" ,
    ]
});

#[no_mangle]
pub extern "C" fn cloud_post_setup() -> () {
    ()
}
