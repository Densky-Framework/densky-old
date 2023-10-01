mod discover;
#[cfg(test)]
mod mod_test;
mod parser;
mod tree;
#[cfg(test)]
mod tree_test;

use std::cell::RefCell;
use std::fs;
use std::io;
use std::sync::MutexGuard;

pub use discover::*;
pub use parser::*;
pub use tree::*;

use crate::utils::import;
use crate::utils::{join_paths, relative_path};
use crate::walker::WalkerLeaf;

/// Http endpoint node
pub struct HttpLeaf;

impl HttpLeaf {
    pub fn get_content(this: &MutexGuard<'_, WalkerLeaf>) -> io::Result<String> {
        fs::read_to_string(&this.file_path)
    }

    pub fn resolve_import<P: AsRef<str>>(
        this: &MutexGuard<'_, WalkerLeaf>,
        path: P,
    ) -> Option<String> {
        let path = path.as_ref().to_string();
        match path.chars().nth(0) {
            Some(char) if char == '.' || char == '/' => {
                let absolute = if char == '.' {
                    let input_dirname = this.file_path.parent()?;
                    join_paths(path, input_dirname)
                } else {
                    path
                };

                let output_dirname = this.output_path.parent()?;

                relative_path(absolute, output_dirname).map(|path| path.display().to_string())
            }
            _ => Some(path),
        }
    }

    fn get_imports(
        this: &MutexGuard<'_, WalkerLeaf>,
        content: String,
    ) -> Result<(String, String), HttpParseError> {
        let content = RefCell::new(content);

        let mut imports: Vec<String> = vec![];

        loop {
            let mut content_mut = content.borrow_mut();
            let import_idx = match &content_mut.find("import") {
                Some(idx) => idx.clone(),
                None => break,
            };

            let content = &content_mut[(import_idx + "import ".len())..];
            let quote_idx = match &content.find("\"") {
                Some(idx) => idx.clone(),
                None => break,
            };

            let inner = if quote_idx < "  from ".len() {
                None
            } else {
                let from_idx = match &content.find("from") {
                    Some(idx) => idx.clone(),
                    None => {
                        return Err(HttpParseError::InvalidSyntax(
                            this.rel_path.clone(),
                            "Malformed import. Missing 'from' keyword".to_string(),
                        ))
                    }
                };
                Some(&content[..(from_idx - 1)])
            };

            let last_quote_idx = match &content.chars().skip(quote_idx + 1).position(|c| c == '"') {
                Some(idx) => idx.clone(),
                None => {
                    return Err(HttpParseError::InvalidSyntax(
                        this.rel_path.clone(),
                        "Malformed import. Missing closing quote.".to_string(),
                    ))
                }
            };

            let out_idx = quote_idx + last_quote_idx + 2;
            let path = &content[(quote_idx + 1)..(out_idx - 1)];
            let path = Self::resolve_import(this, path).unwrap();
            let import_statement = if let Some(inner) = inner {
                import(inner, path)
            } else {
                import("{}", path)
            };
            let content = &content[(out_idx)..];
            *content_mut = content.to_string();

            imports.push(import_statement);
        }

        let content = content.borrow();
        Ok((imports.join(";\n"), content.to_string()))
    }

    fn get_handlers(
        this: &MutexGuard<'_, WalkerLeaf>,
        content: String,
    ) -> Result<(String, String), HttpParseError> {
        let (handlers, content) = match http_parse(content, this.file_path.display().to_string()) {
            Ok(h) => h,
            Err(e) => return Err(e),
        };
        let handlers: Vec<String> = handlers
            .borrow()
            .iter()
            .map(|handler| {
                let if_condition = match &handler.method {
                    &HTTPMethod::ANY => None,
                    &HTTPMethod::GET => Some("GET"),
                    &HTTPMethod::POST => Some("POST"),
                    &HTTPMethod::PATCH => Some("PATCH"),
                    &HTTPMethod::DELETE => Some("DELETE"),
                    &HTTPMethod::OPTIONS => Some("OPTIONS"),
                };

                if let Some(if_condition) = if_condition {
                    format!(
                        "if ({}.method == \"{}\") {{\n{}\n}}",
                        REQ_PARAM, if_condition, &handler.body
                    )
                } else {
                    handler.body.to_string()
                }
            })
            .collect();
        Ok((handlers.join("\n"), content))
    }

    pub fn get_parts(
        this: &MutexGuard<'_, WalkerLeaf>,
    ) -> Result<(String, String, String), HttpParseError> {
        let content = match Self::get_content(this) {
            Ok(c) => c,
            Err(_) => return Err(HttpParseError::Empty(this.rel_path.clone())),
        };

        let (imports, content) = Self::get_imports(this, content)?;
        let (handlers, content) = Self::get_handlers(this, content)?;

        return Ok((imports, handlers, content));
    }

    pub fn generate_file(this: &MutexGuard<'_, WalkerLeaf>) -> Result<String, HttpParseError> {
        let leaf_parts = Self::get_parts(this)?;

        let leaf_imports = leaf_parts.0;
        let leaf_content = leaf_parts.2;
        let leaf_handlers = leaf_parts.1;
        let is_empty_handlers = String::is_empty(&leaf_handlers);
        let imports = "import * as $_Densky_Runtime_$ from \"densky/runtime.ts\";";
        let imports = format!("{imports}\n{leaf_imports}");
        let top_content = format!("{imports}\n{leaf_content}");

        let handler_content = if is_empty_handlers {
            String::new()
        } else {
            leaf_handlers
        };

        let (pretty, _) = prettify_js::prettyprint(
            format!(
                "// deno-lint-ignore-file
                // File auto-generated by Densky Framework
                {top_content}
                (!!Deno.env.has(\"DENSKY_MODULES_LOAD_DEBUG\")) && console.log(\"[Module Load Debug]\", import.meta.url);
                export default async function(__req_param__: $_Densky_Runtime_$.HTTPRequest) {{
                  {handler_content}
                }}",
            )
            .as_str(),
        );

        return Ok(pretty);
    }
}
