#[macro_use]
extern crate densky_adapter;

pub mod context;

use std::path::PathBuf;

use densky_adapter::{
    context::CloudContextRaw, CloudFile, CloudFileResolve, CloudManifestUpdate, OptimizedTreeLeaf,
};

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

    let filename = relative_path.file_name().unwrap();
    let filename = filename.to_str().unwrap();
    let filename_first_char = &filename[0..1];
    match filename_first_char {
        "_" => match filename {
            "_index.ts" => CloudFileResolve::Index,
            "_middleware.ts" => CloudFileResolve::SingleThorn("middleware"),
            "_fallback.ts" => CloudFileResolve::SingleThorn("fallback"),
            _ => CloudFileResolve::Ignore,
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

                CloudFileResolve::Dynamic(
                    prefix.join("/").replace(".ts", ""),
                    dynamic_part.1.to_string_lossy().replace(".ts", "").into(),
                    suffix.join("/").replace(".ts", ""),
                )
            } else {
                CloudFileResolve::Pass
            }
        }
    }
}

#[no_mangle]
pub fn cloud_before_manifest() -> CloudManifestUpdate {
    CloudManifestUpdate::new()
        .add_import("{ type HTTPRequest }", "densky/http-router.ts")
        .add_argument("req", "HTTPRequest")
}

#[no_mangle]
pub fn cloud_manifest(
    leaf: OptimizedTreeLeaf,
    static_children: String,
    children: String,
    dynamic_child: String,
) -> CloudManifestUpdate {
    let pathname_comment = format!("// {}", leaf.pathname);
    let children = if static_children.is_empty() {
        children
    } else {
        format!(
            "{{
                const __DENSKY_static_children = {{ {static_children} }};
                const out = __DENSKY_static_children[req.__accumulator__.path];
                if (out) return out();
            }};
            {children}"
        )
    };

    let inner = leaf.index.as_ref().map(|input_path| {
        let middlewares = leaf
            .single_thorns
            .get("middleware")
            .map(|t| t.iter().map(|t| format!("{t:?},")).collect::<String>())
            .unwrap_or(String::new());

        let fallbacks = leaf
            .single_thorns
            .get("fallback")
            .map(|t| t.iter().map(|t| format!("{t:?},")).collect::<String>())
            .unwrap_or(String::new());

        format!(
            "return {{
                middlewares: [{middlewares}],
                fallbacks: [{fallbacks}],
                controller: {input_path:?}
            }};"
        )
    });

    if leaf.is_root {
        let inner = inner
            .map(|i| format!("if (req.__accumulator__.segments.length === 0) {{ {i} }}"))
            .unwrap_or_default();

        CloudManifestUpdate::new_content(format!(
            r#"{children}
                {pathname_comment}
                {inner}
                {dynamic_child}"#,
        ))
    } else {
        if leaf.is_static {
            CloudManifestUpdate::new_content(format!(
                "{pathname_comment}\n{}",
                inner.expect("Static leafs should have index")
            ))
        } else {
            let inner = inner
                .map(|i| format!("if (req.__accumulator__.segments.length === 0) {{ {i} }}"))
                .unwrap_or_default();

            if let Some(varname) = leaf.varname {
                let varname = &varname[1..];

                CloudManifestUpdate::new_content(format!(
                    r#"{pathname_comment}
                            {{
                                const __var_{varname} = req.__accumulator__.segments.shift();
                                req.params.set("{varname}", __var_{varname});
                                req.__accumulator__.path = req.__accumulator__.segments.join("/");

                                {children}
                                {inner}
                                {dynamic_child}
                            }}"#
                ))
            } else {
                let slash_count = leaf.relative_pathname.chars().filter(|c| c == &'/').count();
                // The first part doesn't have slash
                let slash_count = slash_count + 1;

                CloudManifestUpdate::new_content(format!(
                    r#"{pathname_comment}
                        if (req.__accumulator__.path.startsWith("{}/")) {{
                            req.__accumulator__.segments = req.__accumulator__.segments.slice({slash_count});
                            req.__accumulator__.path = req.__accumulator__.segments.join("/");

                            {children}
                            {inner}
                            {dynamic_child}
                        }}"#,
                    leaf.relative_pathname
                ))
            }
        }
    }
}
