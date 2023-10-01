use std::{
    fs,
    sync::{Arc, Mutex},
};

use densky_core::{
    http::{http_discover, HttpLeaf, HttpTree},
    utils::join_paths,
    views::{view_discover, ViewLeaf},
    walker::{WalkerContainer, WalkerLeaf, WalkerTree},
    CompileContext,
};

fn process_leaf(http_leaf: Arc<Mutex<WalkerLeaf>>) {
    let http_tree = http_leaf.lock().unwrap();
    let output = match HttpLeaf::generate_file(&http_tree) {
        Ok(o) => o,
        Err(e) => panic!("{:?}", e),
    };
    let output_path = &http_tree.output_path;
    println!("{}", output_path.display());
    let _ = fs::create_dir_all(output_path.parent().unwrap());
    fs::write(output_path, output).unwrap();
}

fn process_entry(http_tree: Arc<Mutex<WalkerTree>>, container: &mut WalkerContainer) {
    let mut http_tree = http_tree.lock().unwrap();

    let output = match HttpTree::generate_file(&mut http_tree, container) {
        Ok(o) => o,
        Err(e) => panic!("{:?}", e),
    };
    let output_path = &http_tree.output_path;
    println!("{}", output_path.display());
    let _ = fs::create_dir_all(output_path.parent().unwrap());
    fs::write(output_path, output).unwrap();

    let children = http_tree.children.clone();

    if let Some(fallback) = &http_tree.fallback {
        let fallback = container.get_leaf(*fallback).unwrap();
        process_leaf(fallback);
    }
    if let Some(middleware) = &http_tree.middleware {
        let middleware = container.get_leaf(*middleware).unwrap();
        process_leaf(middleware);
    }

    drop(http_tree);

    for child in children.iter() {
        process_entry(container.get_tree(*child).unwrap(), container);
    }
}

fn process_view(view: ViewLeaf) -> Option<()> {
    let output = view
        .generate_file()
        .map(|c| prettify_js::prettyprint(&c.0.to_owned()).0)?;

    let output_path = view.output_path();
    println!("{}", output_path.display());
    let _ = fs::create_dir_all(output_path.parent().unwrap());
    fs::write(output_path, output).unwrap();

    Some(())
}

fn main() {
    let path = std::env::current_dir().unwrap();
    let mut rel_path = std::env::args();
    let rel_path = match rel_path.nth(1) {
        None => panic!("Provide a server path"),
        Some(path) => {
            if path.len() == 0 {
                panic!("Provide a server path")
            } else {
                path
            }
        }
    };
    let example_server = join_paths(rel_path, path);

    let compile_context = CompileContext {
        output_dir: join_paths(".densky", &example_server),
        routes_path: join_paths("src/routes", &example_server),
        views_path: join_paths("src/views", &example_server),
        static_path: join_paths("src/static", &example_server),
        verbose: true,
        static_prefix: "static/".to_owned(),
    };

    let views = view_discover(&compile_context);
    for view in views {
        process_view(view);
    }

    let (mut container, http_tree) = http_discover(&compile_context);

    process_entry(http_tree, &mut container);

    fs::write(join_paths("http.main.ts", &compile_context.output_dir), "
// THIS FILE WAS GENERATED BY DENSKY-BACKEND (By Apika Luca)
import * as $Densky$ from \"densky/runtime.ts\";
import mainHandler from \"./http/_index.ts\";

function toResponse (
  req: $Densky$.HTTPRequest,
  response: Response | $Densky$.HTTPError | Error | void
): Response {
  if (response instanceof Error) 
    response = $Densky$.HTTPError.fromError(response);

  if (response instanceof $Densky$.HTTPError) 
    response = response.toResponse();

  if (response instanceof Response) 
    return new Response(response.body, {
      status: response.status,
      statusText: response.statusText,
      headers: Object.fromEntries([...req.headers.entries(), ...response.headers.entries()]),
    });

  throw new Error(\"Unreachable code\");
}

export default async function requestHandler(req: $Densky$.HTTPRequest): Promise<Response> {
  return toResponse(req, await mainHandler(req) ?? new $Densky$.HTTPError($Densky$.StatusCode.NOT_FOUND));
}").unwrap();

    fs::write(join_paths("main.ts", &compile_context.output_dir), format!("
// THIS FILE WAS GENERATED BY DENSKY-BACKEND (By Apika Luca)
import * as $Densky$ from \"densky\";
import httpHandler from \"./http.main.ts\";

$Densky$.HTTPResponse.viewsPath = \"{}\";

export default async function requestHandler(request: Deno.RequestEvent, conn: Deno.Conn): Promise<Response> {{
  const req = new $Densky$.HTTPRequest(request);
  await req.prepare();

  return await httpHandler(req);
}}", join_paths("views", &compile_context.output_dir))).unwrap();

    // let http_tree = http_tree.lock().unwrap();
    // println!("{}", Fmt(|f| http_tree.display(f, &container)));
    // println!("{}", http_tree);
    // println!("{}", http_tree.generate_file().unwrap())
    // if let Some(http_leaf) = &http_tree.leaf {
    //     let http_leaf = http_leaf.borrow();
    //     println!("{:?}", http_leaf.generate_file());
    // }
}
