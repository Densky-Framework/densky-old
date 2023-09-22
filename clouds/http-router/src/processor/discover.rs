use std::sync::{Arc, Mutex};

use crate::{
    walker::{walker_tree_discover, WalkerContainer, WalkerTree},
    CompileContext,
};

pub fn http_discover(
    compile_context: &CompileContext,
) -> (WalkerContainer, Arc<Mutex<WalkerTree>>) {
    walker_tree_discover(
        "http",
        compile_context.routes_path.clone(),
        &compile_context,
    )
}
