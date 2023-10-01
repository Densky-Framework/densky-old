mod importer;
mod url_to_matcher;

use std::path::PathBuf;

use dprint_plugin_typescript::{configuration as dprint_config, format_text};

pub use self::importer::*;
pub use self::url_to_matcher::*;

static mut GLOBAL_NEXT_NODE_ID: u64 = 0;

pub fn next_node_id() -> u64 {
    unsafe {
        GLOBAL_NEXT_NODE_ID += 1;
        GLOBAL_NEXT_NODE_ID
    }
}

pub fn format_js(txt: impl Into<String>) -> String {
    let config = dprint_config::ConfigurationBuilder::new()
        .line_width(80)
        .prefer_hanging(true)
        .prefer_single_line(false)
        .quote_style(dprint_config::QuoteStyle::PreferSingle)
        .next_control_flow_position(dprint_config::NextControlFlowPosition::SameLine)
        .build();

    format_text(&PathBuf::from("/tmp/file.ts"), &txt.into(), &config)
        .unwrap()
        .unwrap()
}
