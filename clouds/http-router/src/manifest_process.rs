use densky_adapter::{CloudManifestUpdate, OptimizedTreeLeaf, Result};

#[no_mangle]
pub fn cloud_before_manifest() -> Result<CloudManifestUpdate> {
    Ok(CloudManifestUpdate::new()
        .add_import("{ type HTTPRequest }", "densky/http-router.ts")
        .add_argument("req", "HTTPRequest"))
}

#[no_mangle]
pub fn cloud_manifest(
    leaf: OptimizedTreeLeaf,
    static_children: String,
    children: String,
    dynamic_child: String,
) -> Result<CloudManifestUpdate> {
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
            .unwrap_or_default();

        let fallbacks = leaf
            .single_thorns
            .get("fallback")
            .map(|t| t.iter().map(|t| format!("{t:?},")).collect::<String>())
            .unwrap_or_default();

        format!(
            "return {{
                middlewares: [{middlewares}],
                fallbacks: [{fallbacks}],
                controller: {input_path:?}
            }};"
        )
    });

    if leaf.is_root {
        // Root
        let inner = inner
            .map(|i| format!("if (req.__accumulator__.segments.length === 0) {{ {i} }}"))
            .unwrap_or_default();

        Ok(CloudManifestUpdate::new_content(format!(
            "{children}\n{pathname_comment}\n{inner}\n{dynamic_child}\nreturn null;",
        )))
    } else {
        if leaf.is_static {
            // Static Node
            Ok(CloudManifestUpdate::new_content(format!(
                "{pathname_comment}\n{}",
                inner.expect("Static leafs should have index")
            )))
        } else {
            // Dynamic Node
            let inner = inner
                .map(|i| format!("if (req.__accumulator__.segments.length === 0) {{ {i} }}"))
                .unwrap_or_default();

            if let Some(varname) = leaf.varname {
                // Dynamic Named Node
                let varname = &varname[1..];

                Ok(CloudManifestUpdate::new_content(format!(
                    r#"{pathname_comment}
                        if (req.__accumulator__.segments.length > 0) {{
                            const __var_{varname} = req.__accumulator__.segments.shift();
                            if (__var_{varname} === undefined) {{
                                throw new Error("Unreachable");
                            }}
                            req.params.set("{varname}", __var_{varname});
                            // @ts-ignore READ-ONLY
                            req.__accumulator__.path = req.__accumulator__.segments.join("/");

                            {children}
                            {inner}
                            {dynamic_child}
                            return null;
                        }}"#
                )))
            } else {
                // Index Node
                let slash_count = leaf.relative_pathname.chars().filter(|c| c == &'/').count();
                // The first part doesn't have slash
                let slash_count = slash_count + 1;

                Ok(CloudManifestUpdate::new_content(format!(
                    r#" {pathname_comment}
                        if (req.__accumulator__.path === "{pathname}" || req.__accumulator__.path.startsWith("{pathname}/")) {{
                            // @ts-ignore READ-ONLY
                            req.__accumulator__.segments = req.__accumulator__.segments.slice({slash_count});
                            // @ts-ignore READ-ONLY
                            req.__accumulator__.path = req.__accumulator__.segments.join("/");

                            {children}
                            {inner}
                            {dynamic_child}
                            return null;
                        }}"#,
                    pathname = leaf.relative_pathname
                )))
            }
        }
    }
}
