use std::path::PathBuf;

use densky_core::{
    densky_adapter::utils::join_paths, optimized_tree::OptimizedTreeContainer, sky::CloudPlugin,
    CompileContext,
};

use super::_macro::def_command;

def_command!(PluginTestCommand("plugin-test") {
    <lib>("Library path, without 'lib' prefix or extensions") {
        value_hint: clap::ValueHint::FilePath,
        value_parser: clap::value_parser!(PathBuf),
    },

    process: process
});

fn process(matches: &clap::ArgMatches) {
    let lib_path = matches.get_one::<PathBuf>("lib").unwrap();

    let mut plugin = CloudPlugin::new(lib_path).unwrap();
    plugin
        .setup()
        .expect("CloudSetup is required on any cloud plugin");

    unsafe {
        plugin.cloud_debug_context();
    }

    let target_path = std::env::current_dir().unwrap();
    let compile_context = CompileContext {
        output_dir: join_paths(".densky", &target_path),
        cwd: target_path.display().to_string(),
        verbose: true,
    };

    let container = plugin.resolve_optimized_tree(&compile_context).unwrap();

    let processed_root = process_node(container.get_root_id().unwrap(), &mut plugin, &container);
    let processed_root = densky_core::utils::format_js(&processed_root);
    println!("{processed_root}");

    // println!(
    //     "{}",
    //     Fmt(|f| container
    //         .get_root()
    //         .unwrap()
    //         .read()
    //         .unwrap()
    //         .display(f, &container))
    // );

    plugin.close();
}

fn process_node(id: u64, plugin: &mut CloudPlugin, container: &OptimizedTreeContainer) -> String {
    let node = container.nodes.get_reader(id).unwrap();

    let mut static_children = String::new();
    let mut children = String::new();

    for (pathname, id) in node.static_children.iter() {
        static_children.reserve_exact(pathname.len() + 13);
        static_children.push('"');
        static_children += pathname;
        static_children.push_str("\": () => {");
        static_children += &process_node(*id, plugin, container);
        static_children.push('}');
        static_children.push(',');
    }

    for (_, id) in node.dynamic_children.iter() {
        children += &process_node(*id, plugin, container);
    }

    let dynamic_child = if let Some((id, varname)) = node.dynamic.as_ref() {
        let mut child = container.nodes.get_writer(*id).unwrap();
        child.varname = Some(varname.clone());
        drop(child);

        process_node(*id, plugin, container)
    } else {
        String::new()
    };

    let leaf = node.into_leaf(container);
    println!("{leaf:#?}");
    unsafe {
        plugin
            .cloud_optimized_tree_process(leaf, static_children, children, dynamic_child)
            .unwrap()
    }
}
