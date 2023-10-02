use std::{cell::RefCell, fmt};

use pathdiff::diff_paths;
use recur_fn::{recur_fn, RecurFn};
use regex::Regex;

#[derive(Debug, Clone)]
pub enum HttpParseError {
    Empty(String),
    InvalidSyntax(String, String),
    CurrentDir,
}

impl fmt::Display for HttpParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentDir => f.write_str("Unable to read current dir"),
            Self::Empty(rel_path) | Self::InvalidSyntax(rel_path, _) => {
                f.write_str(format!("[{}]", rel_path).as_str())?;
                f.write_str(match self {
                    Self::Empty(_) => "The file has no handlers",
                    Self::InvalidSyntax(_, message) => &message,
                    _ => "",
                })
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum HTTPMethod {
    GET,
    POST,
    DELETE,
    PATCH,
    OPTIONS,
    ANY,
}

impl HTTPMethod {
    pub fn from_string(value: String) -> Option<Self> {
        match value.to_uppercase().as_str() {
            "GET" => Some(Self::GET),
            "POST" => Some(Self::POST),
            "DELETE" => Some(Self::DELETE),
            "PATCH" => Some(Self::PATCH),
            "OPTIONS" => Some(Self::OPTIONS),
            "ANY" => Some(Self::ANY),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpHandler {
    pub method: HTTPMethod,
    pub body: String,
    pub req_param: Option<String>,
}

/*
 * export function      GET(reqParam: TYPE): TYPE {
 * ^ Handler definition  ^ METHOD
 *   // ..
 *   ^ BODY
 * }
 */

pub const REQ_PARAM: &str = "__req_param__";

// https://regex101.com/r/64vt2Q/2
const HANDLER_REGEX: &str = r"(?m)export\s+(?:async\s+)?function\s+(\w+)\s*\((?:(\w+)(?::\s+[^)]+)?)?\)(?::\s+.+)?\s*\{|export\s+const\s+(\w+)\s*=\s*(?:async)?\s*\((?:(\w+)(?::\s+[^)]+)?)?\)(?::\s+.+)?\s*=>\s*\{";

// https://regex101.com/r/IiUXQZ/3
const DEFAULT_REGEX: &str = r"(?m)export\s+default\s+(?:async\s+)?function\s+\w*\s*\((?:(\w+)(?::\s+[^)]+)?)?\)(?::\s+.+)?\s*\{|export\s+default\s+(?:async)?\s*\((?:(\w+)(?::\s+[^)]+)?)?\)(?::\s+.+)?\s*=>\s*\{";

pub fn http_parse(
    content: String,
    file_path: String,
) -> Result<(RefCell<Vec<HttpHandler>>, String), HttpParseError> {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(_) => return Err(HttpParseError::CurrentDir),
    };
    let rel_path = diff_paths(&file_path, cwd).unwrap();
    let rel_path = rel_path.display().to_string();

    let handlers: RefCell<Vec<HttpHandler>> = RefCell::new(Vec::new());
    let content = content.to_string();

    let result_content = RefCell::new(String::new());

    let handler_regex = Regex::new(HANDLER_REGEX).unwrap();
    let default_regex = Regex::new(DEFAULT_REGEX).unwrap();

    let process_handler = |remain: &String, req_param: Option<String>| {
        let mut brace_count = 1;
        let mut tmp_remain = remain.clone();
        let mut length = 0;

        while brace_count > 0 {
            let near_close_bracket = tmp_remain.find('}');
            let near_open_bracket = tmp_remain.find('{');

            // If 'close_bracket' is more close then substract one to braceCount
            // Open = None, Close = 0..
            // else, add one to 'brace_count'
            // Open = 0.., Close = None..
            let (delta, bracket_pos) = match (near_close_bracket, near_open_bracket) {
                (Some(near_close_bracket), None) => (-1, near_close_bracket),
                (None, Some(near_open_bracket)) => (1, near_open_bracket),
                // Win the most near
                (Some(near_close_bracket), Some(near_open_bracket)) => {
                    if near_open_bracket > near_close_bracket {
                        (-1, near_close_bracket)
                    } else {
                        (1, near_open_bracket)
                    }
                }
                // Both can't be none
                (None, None) => {
                    return (
                        None,
                        None,
                        Some(HttpParseError::InvalidSyntax(
                            rel_path.clone(),
                            "Unclosed bracket".to_string(),
                        )),
                    );
                }
            };
            brace_count += delta;
            length += bracket_pos + 1;
            tmp_remain = tmp_remain[(bracket_pos + 1)..].to_string();
        }
        // Set variable only if it's different to REQ_PARAM
        let req_decl = if let Some(req_param) = req_param.clone() {
            if req_param.as_str() == REQ_PARAM {
                "".to_string()
            } else {
                format!("let {} = {};\n", req_param, REQ_PARAM)
            }
        } else {
            "".to_string()
        };

        let end = req_decl + remain[0..(length - 1)].trim();

        let remain = remain[length..].to_string();

        (Some(remain), Some(end), None)
    };

    let next_handler = recur_fn(
        |next_handler, remain: String| -> (String, Option<HttpParseError>) {
            // Empty file
            if remain.len() <= 1 {
                return (remain, None);
            }

            let handler = handler_regex.captures(&remain);
            let handler = match handler {
                Some(handler) => handler,
                None => return (remain, None),
            };

            let method = handler
                .get(1)
                .or_else(|| handler.get(3))
                .map(|method| method.as_str().to_string())
                .unwrap();

            let req_param = handler
                .get(2)
                .or_else(|| handler.get(4))
                .map(|req_param| req_param.as_str().to_string());

            let start_idx = handler.get(0).unwrap().start();
            let body_idx = handler.get(0).unwrap().end();

            *result_content.borrow_mut() += remain[..start_idx].trim();
            let remain = remain[body_idx..].trim().to_string();

            let (remain, body, err) = process_handler(&remain, req_param.clone());

            if err.is_some() {
                return (String::new(), err);
            }

            let (remain, body) = (remain.unwrap(), body.unwrap());

            handlers.borrow_mut().push(HttpHandler {
                method: HTTPMethod::from_string(method).unwrap(),
                body,
                req_param,
            });

            next_handler(remain)
        },
    );

    match next_handler.call(content) {
        (_, Some(err)) => Err(err),
        (remain, None) => {
            'l: {
                let handler = default_regex.captures(&remain);
                let handler = match handler {
                    Some(handler) => handler,
                    None => break 'l,
                };

                let req_param = handler
                    .get(1)
                    .or_else(|| handler.get(2))
                    .map(|req_param| req_param.as_str().to_string());

                let start_idx = handler.get(0).unwrap().start();
                let body_idx = handler.get(0).unwrap().end();

                *result_content.borrow_mut() += remain[..start_idx].trim();
                let remain = &remain[body_idx..].trim().to_string();

                let (_, body, err) = process_handler(&remain, req_param.clone());

                if err.is_some() {
                    break 'l;
                }

                let body = body.unwrap();

                handlers.borrow_mut().push(HttpHandler {
                    method: HTTPMethod::ANY,
                    body,
                    req_param,
                });
            };

            if handlers.borrow().len() == 0 {
                Err(HttpParseError::Empty(rel_path))
            } else {
                Ok((handlers, result_content.borrow().to_string()))
            }
        }
    }
}
