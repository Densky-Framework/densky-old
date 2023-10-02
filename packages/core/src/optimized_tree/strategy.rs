use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use densky_adapter::{
    log::PathDebugDisplay, log_trace, utils::join_paths, CloudFile, CloudFileResolve,
};
use pathdiff::diff_paths;
use walkdir::WalkDir;

use crate::{
    optimized_tree::{
        node::OptimizedTreeNodeInsertResult, OptimizedTreeContainer, OptimizedTreeNode,
    },
    sky::CloudPlugin,
    CompileContext,
};

pub fn optimized_tree_strategy(
    input_path: impl AsRef<Path>,
    plugin: &CloudPlugin,
    ctx: &CompileContext,
) -> (OptimizedTreeContainer, Arc<RwLock<OptimizedTreeNode>>) {
    let output_dir = join_paths(&plugin.get_setup().source_folder, &ctx.output_dir);

    let mut container = OptimizedTreeContainer::new(output_dir.clone());
    let root = container.create_root();

    log_trace!([plugin.name] "WALKING: {}", input_path.as_ref().display());
    let walk_dir = WalkDir::new(&input_path).into_iter().filter_map(Result::ok);
    for entry in walk_dir {
        let file_path = entry.path();

        if let Some(ext) = file_path.extension() {
            if ext != "ts" {
                continue;
            }
        } else {
            continue;
        }

        let relative = match diff_paths(file_path, &input_path) {
            Some(path) => path,
            None => continue,
        };
        let relative_path = relative.display().to_string();
        let extension = relative
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let path = format!("{}", &relative.with_extension("").display());
        let file_path_debug = file_path.display_debug();
        let file_path = file_path.display().to_string();
        let output_path = join_paths(relative, &output_dir);

        log_trace!([plugin.name] "Resolving file: {file_path_debug}");

        let cloud_file = CloudFile::new(&file_path, relative_path, &output_path);
        let resolved_file = unsafe { plugin.cloud_file_resolve(cloud_file) };
        let resolved_file = resolved_file.unwrap_or_default();
        log_trace!([plugin.name] "Resolved as {resolved_file:?}");

        let dummy_leaf = OptimizedTreeNode::new_leaf(
            path.clone() + ".dummy",
            Some(file_path.clone().into()),
            output_path.clone().into(),
        );
        let mut leaf = OptimizedTreeNode::new_leaf(
            path,
            Some(file_path.clone().into()),
            output_path.clone().into(),
        );
        let dummy_leaf = container.nodes.add(dummy_leaf);
        leaf.index = Some(dummy_leaf);

        let leaf = container.nodes.add(leaf);

        let next_iter = root
            .write()
            .unwrap()
            .insert(leaf, resolved_file, &mut container);

        perform_insert_action(
            leaf,
            next_iter,
            &mut InsertContext {
                container: &mut container,
                plugin,
                file_path,
                output_path,
                extension,
            },
        );

        // println!(
        //     "{:#?}",
        //     Fmt(|f| root.read().unwrap().display(f, &container))
        // )
    }

    (container, root)
}

struct InsertContext<'a> {
    container: &'a mut OptimizedTreeContainer,
    plugin: &'a CloudPlugin,
    file_path: String,
    output_path: String,
    extension: String,
}

fn perform_insert_action(
    node: u64,
    action: OptimizedTreeNodeInsertResult,
    context: &mut InsertContext<'_>,
) {
    match action {
        OptimizedTreeNodeInsertResult::Resolve(new_parent, suffix) => {
            let resolved_file = resolve_file(node, &suffix, context);

            let root = context.container.nodes.get(new_parent).unwrap().clone();
            let next_iter =
                root.write()
                    .unwrap()
                    .insert(node, resolved_file, &mut context.container);

            perform_insert_action(node, next_iter, context)
        }
        OptimizedTreeNodeInsertResult::RemoveNode => {
            log_trace!(["OTreeStrategy"] "Removing {node}");
            context.container.nodes.remove(node);
        }
        OptimizedTreeNodeInsertResult::MergeNodes(new_node, node_b_suffix) => {
            log_trace!(["OTreeStrategy"] "Merging ({})", node_b_suffix);

            let root = context.container.nodes.get(new_node).unwrap().clone();

            let resolved_node_b = resolve_file(node, &node_b_suffix, context);
            let next_iter_b =
                root.write()
                    .unwrap()
                    .insert(node, resolved_node_b, &mut context.container);

            perform_insert_action(node, next_iter_b, context)
        }
        OptimizedTreeNodeInsertResult::None => {}
    };
}

fn resolve_file<'a>(
    leaf: u64,
    suffix: &String,
    context: &mut InsertContext<'a>,
) -> CloudFileResolve {
    log_trace!([context.plugin.name] "Semi-Inserted to /{suffix}");

    context
        .container
        .nodes
        .get_writer(leaf)
        .unwrap()
        .relative_pathname = suffix.to_owned();

    let suffix = PathBuf::from(suffix).with_extension(&context.extension);
    log_trace!([context.plugin.name] "Resolving file: {}", suffix.display_debug());

    let cloud_file = CloudFile::new(
        &context.file_path,
        suffix.display().to_string(),
        &context.output_path,
    );
    let resolved_file = unsafe { context.plugin.cloud_file_resolve(cloud_file) };
    let resolved_file = resolved_file.unwrap_or_default();
    log_trace!([context.plugin.name] "Resolved as {resolved_file:?}");

    resolved_file
}
