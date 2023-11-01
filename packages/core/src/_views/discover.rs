use crate::{walker::simple_discover, CompileContext};

use super::ViewLeaf;

pub fn view_discover(compile_context: &CompileContext) -> impl Iterator<Item = ViewLeaf> {
    simple_discover(
        "views",
        compile_context.views_path.clone(),
        &compile_context,
    )
    .filter_map(|a| a) // Only the Some type
    .map(ViewLeaf::from)
}
