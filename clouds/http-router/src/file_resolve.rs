use densky_adapter::{ErrorContext, Result};
use std::path::PathBuf;

use densky_adapter::{context::CloudContextRaw, CloudFile, CloudFileResolve};

#[no_mangle]
pub fn cloud_file_resolve(file: CloudFile, _context: CloudContextRaw) -> Result<CloudFileResolve> {
    let relative_path: PathBuf = file.relative_path.into();

    let filename = relative_path.file_name().with_context(|| {
        format!(
            "Invalid file path on file resolve: {}",
            relative_path.display()
        )
    })?;
    let filename = filename
        .to_str()
        .with_context(|| format!("Can't parse filename: {}", filename.to_string_lossy()))?;
    let filename_first_char = &filename[0..1];
    match filename_first_char {
        "_" => match filename {
            "_index.ts" => Ok(CloudFileResolve::Index),
            "_middleware.ts" => Ok(CloudFileResolve::SingleThorn("middleware")),
            "_fallback.ts" => Ok(CloudFileResolve::SingleThorn("fallback")),
            _ => Ok(CloudFileResolve::Ignore),
        },
        _ => {
            let path_segments: Vec<&std::ffi::OsStr> = relative_path.iter().collect();
            let dynamic_part = path_segments
                .iter()
                .enumerate()
                .find(|f| (**f.1).to_string_lossy().starts_with('$'));

            if let Some(dynamic_part) = dynamic_part {
                let mut prefix: Vec<String> = vec![];
                let mut suffix: Vec<String> = vec![];

                for (i, part) in path_segments.iter().enumerate() {
                    if i < dynamic_part.0 {
                        prefix.push(part.to_string_lossy().into());
                    } else if i > dynamic_part.0 {
                        suffix.push(part.to_string_lossy().into());
                    }
                }

                Ok(CloudFileResolve::Dynamic(
                    prefix.join("/").replace(".ts", ""),
                    dynamic_part.1.to_string_lossy().replace(".ts", "").into(),
                    suffix.join("/").replace(".ts", ""),
                ))
            } else {
                Ok(CloudFileResolve::Pass)
            }
        }
    }
}
