mod importer;
mod url_to_matcher;

use std::path::PathBuf;

use dprint_plugin_typescript::{configuration as dprint_config, format_text};

pub use self::importer::*;
pub use self::url_to_matcher::*;

pub fn format_js(txt: impl Into<String>) -> String {
    let config = dprint_config::ConfigurationBuilder::new()
        .line_width(80)
        .build();

    format_text(&PathBuf::from("/tmp/file.ts"), &txt.into(), &config)
        .unwrap()
        .unwrap()
}
