#[macro_use]
extern crate densky_adapter;

pub mod context;

use std::path::PathBuf;

use densky_adapter::{context::CloudContextRaw, CloudFile, CloudFileResolve};

// use context::HttpRouterContext;

cloud_setup!(http::router {
    source_folder: "http",
    file_ends: ".ts",
    file_strategy: OptimizedTree,
    dependencies: [
        database::orm =>? "^1.0.0" ,
    ]
});

// cloud_context!(HttpRouterContext);

#[no_mangle]
pub extern "C" fn cloud_post_setup() -> () {
    ()
}

#[no_mangle]
pub fn cloud_file_resolve(file: CloudFile, _context: CloudContextRaw) -> CloudFileResolve {
    let relative_path: PathBuf = file.relative_path.into();

    let path_parts: Vec<&std::ffi::OsStr> = relative_path.iter().collect();
    let dynamic_part = path_parts
        .iter()
        .enumerate()
        .find(|f| (**f.1).to_string_lossy().starts_with('$'));

    use CloudFileResolve::*;
    if let Some(dynamic_part) = dynamic_part {
        let mut prefix: Vec<String> = vec![];
        let mut suffix: Vec<String> = vec![];

        for (i, part) in path_parts.iter().enumerate() {
            if i < dynamic_part.0 {
                prefix.push(part.to_string_lossy().into());
            } else if i > dynamic_part.0 {
                suffix.push(part.to_string_lossy().into());
            }
        }

        return Dynamic(
            prefix.join("/").replace(".ts", ""),
            dynamic_part.1.to_string_lossy().replace(".ts", "").into(),
            suffix.join("/").replace(".ts", ""),
        );
    };

    let filename = relative_path.file_name().unwrap();
    let filename = filename.to_str().unwrap();
    let filename_first_char = &filename[0..1];
    match filename_first_char {
        "_" => match filename {
            "_index.ts" => Index,
            "_middleware.ts" => SingleThorn("middleware"),
            "_fallback.ts" => SingleThorn("fallback"),
            _ => Ignore,
        },
        _ => Pass,
    }
}

#[no_mangle]
pub fn cloud_file_processor() -> CloudFile {
    CloudFile::new("full_path", "relative_path", "output_path")
}
