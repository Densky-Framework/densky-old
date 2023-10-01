use std::cmp::Ordering;
use std::path::Path;
use std::sync::MutexGuard;

use crate::utils::{import, relative_path, UrlMatcher};
use crate::walker::{WalkerContainer, WalkerTree};

use super::{HttpLeaf, HttpParseError, REQ_PARAM};

pub struct HttpTree;

impl HttpTree {
    pub fn resolve_import<P: AsRef<Path>>(this: &MutexGuard<'_, WalkerTree>, path: P) -> String {
        let path = path.as_ref().display().to_string();
        match path.chars().nth(0) {
            Some('/') => {
                let output_dirname = this.output_path.parent().expect("Output path is root");

                relative_path(path, output_dirname)
                    .map(|path| path.display().to_string())
                    .expect("Output path is root")
            }
            _ => path,
        }
    }

    pub fn generate_file(
        this: &mut MutexGuard<'_, WalkerTree>,
        container: &mut WalkerContainer,
    ) -> Result<String, HttpParseError> {
        let url_matcher = UrlMatcher::new(this.rel_path.to_owned());
        let leaf_parts = this
            .leaf
            .as_ref()
            .map(|&leaf| HttpLeaf::get_parts(&container.get_leaf_locked(leaf).unwrap()));
        let leaf_parts = if let Some(parts) = leaf_parts {
            match parts {
                Ok(expr) => Some(expr),
                Err(e) => return Err(e.clone()),
            }
        } else {
            None
        };

        let middlewares = if this.is_middleware {
            vec![]
        } else {
            this.get_middlewares(container)
        };

        let empty_string = String::new();
        let leaf_imports = leaf_parts.as_ref().map_or(&empty_string, |parts| &parts.0);
        let leaf_content = leaf_parts.as_ref().map_or(&empty_string, |parts| &parts.2);
        let leaf_handlers = leaf_parts.as_ref().map_or(&empty_string, |parts| &parts.1);
        let is_empty_handlers = String::is_empty(&leaf_handlers);
        let fallback_import = this.fallback.as_ref().map_or_else(
            || String::new(),
            |&fallback| {
                import(
                    "$__fallback__$",
                    Self::resolve_import(
                        this,
                        &container.get_leaf_locked(fallback).unwrap().output_path,
                    ),
                )
            },
        );
        let children_import: Vec<String> = this
            .children
            .iter()
            .enumerate()
            .map(|(index, &child)| {
                import(
                    format!("$__child__${index}"),
                    Self::resolve_import(
                        this,
                        &container.get_tree_locked(child).unwrap().output_path,
                    ),
                )
            })
            .collect();
        let children_import = children_import.join("\n");
        let middlewares_import = if this.is_middleware {
            String::new()
        } else {
            middlewares
                .iter()
                .enumerate()
                .map(|(index, &middleware)| {
                    import(
                        format!("$__middleware__${index}"),
                        Self::resolve_import(
                            this,
                            &container.get_leaf_locked(middleware).unwrap().output_path,
                        ),
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")
        };
        let imports = format!(
            "import * as $_Densky_Runtime_$ from \"densky/runtime.ts\";\n{}\n{}\n{}{}",
            fallback_import, children_import, middlewares_import, leaf_imports
        );

        let middlewares_handlers = if this.is_middleware {
            String::new()
        } else {
            middlewares
                .iter()
                .enumerate()
                .map(|(index, _)| {
                    format!(
                        "{{ 
                          let _ = await $__middleware__${}(__req_param__); 
                          if (_) return _; 
                        }};",
                        index
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")
        };

        let fallback_handler = if this.fallback.is_some() {
            "return $__fallback__$(__req_param__);"
        } else {
            ""
        };

        let top_content = format!(
            "{imports}\n{serial}\n{content}",
            content = leaf_content,
            serial = url_matcher.serial_decl(),
        );

        let mut children_content: Vec<(usize, String)> = (0..this.children.len())
            .map(|index| {
                (
                    *this.children.get(index).unwrap(),
                    format!(
                        "{{ 
                          const r = await $__child__${0}({1}); 
                          if (!!r || r === null) return r; 
                        }};",
                        index, REQ_PARAM
                    ),
                )
            })
            .collect();
        children_content.sort_by(|a, b| {
            let a_child = container.get_tree_locked(a.0).unwrap();
            let b_child = container.get_tree_locked(b.0).unwrap();

            let a_is_deep = a_child.rel_path.find('/').is_some();
            let b_is_deep = b_child.rel_path.find('/').is_some();

            match (a_is_deep, b_is_deep) {
                (true, true) => (),
                (false, false) => {
                    let a_is_var = a_child.rel_path.starts_with('$');
                    let b_is_var = b_child.rel_path.starts_with('$');

                    match (a_is_var, b_is_var) {
                        (true, true) | (false, false) => (),
                        (true, false) => return Ordering::Greater,
                        (false, true) => return Ordering::Less,
                    }
                }
                (true, false) => return Ordering::Greater,
                (false, true) => return Ordering::Less,
            }

            let a_path: Vec<&str> = a_child.rel_path.split('/').collect();
            let b_path: Vec<&str> = b_child.rel_path.split('/').collect();

            a_path.len().cmp(&b_path.len())
        });
        let children_content: Vec<String> =
            children_content.iter().map(|a| a.1.to_owned()).collect();
        let children_content = children_content.join("\n");

        let handler_content = if is_empty_handlers {
            String::new()
        } else {
            format!(
                "if ({exact}) {{ 
                  {middlewares} 
                  {handlers} 
                  ;return new Response(\"Method not handled\", {{ status: 401 }}); 
                }} ",
                middlewares = middlewares_handlers,
                handlers = leaf_handlers,
                exact = url_matcher.exact_decl(REQ_PARAM, this.children.len() != 0),
            )
        };

        let handler_content = format!(
            "{}\n{}\n{}",
            handler_content, children_content, fallback_handler
        );

        let inner_content = if this.is_root || this.children.len() == 0 {
            handler_content
        } else if this.is_fallback {
            leaf_handlers.clone()
        } else {
            format!(
                "if ({start}) {{ 
                  {update} 
                  {inner_content} 
                  return null;
                }}",
                inner_content = handler_content,
                start = url_matcher.start_decl(REQ_PARAM),
                update = url_matcher.update_decl(REQ_PARAM),
            )
        };
        let (pretty, _) = prettify_js::prettyprint(
            format!(
                "// deno-lint-ignore-file 
                // {0}
                // File auto-generated by Densky Framework
                {top_content}
                (!!Deno.env.has(\"DENSKY_MODULES_LOAD_DEBUG\")) && console.log(\"[Module Load Debug]\", import.meta.url);
                ;export default async function(__req_param__: $_Densky_Runtime_$.HTTPRequest) {{
                  {inner_content}
                }}",
                &this.rel_path,
            )
            .as_str(),
        );

        return Ok(pretty);
    }
}
