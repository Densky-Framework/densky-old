use super::_macro::def_command;
use std::{
    ffi::OsStr,
    path::PathBuf,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::{
    compiler::write_aux_files,
    progress,
    watcher::{PollWatcher, WatchKind},
};
use clap::{value_parser, ValueHint};
use densky_core::{
    densky_adapter::utils::{join_paths, Fmt},
    sky::CloudPlugin,
    CompileContext, Manifest,
};

def_command!(DevCommand("dev") {
    [folder]("Proyect folder") {
        default_value: ".",
        value_hint: ValueHint::DirPath,
        value_parser: value_parser!(PathBuf),
    },

    --lib(=lib, "Library path, without 'lib' prefix or extensions") {
        value_hint: clap::ValueHint::FilePath,
        value_parser: clap::value_parser!(PathBuf),
        |f| f.required(true)
    },

    process: process
});

fn process(matches: &clap::ArgMatches) {
    let lib_path = matches.get_one::<PathBuf>("lib").unwrap();

    let mut plugin = CloudPlugin::new(lib_path).unwrap();
    plugin
        .setup()
        .expect("CloudSetup is required on any cloud plugin");

    let folder = matches.get_one::<PathBuf>("folder").unwrap();
    let cwd = std::env::current_dir().unwrap();
    let target_path: PathBuf = join_paths(folder, cwd).into();

    let watching_path = target_path.clone();
    let mut watching_poll = PollWatcher::new(watching_path).unwrap();

    let compile_context = CompileContext {
        output_dir: join_paths(".densky", &target_path),
        cwd: target_path.display().to_string(),
        verbose: true,
    };

    let progress = progress::create_spinner(Some("Discovering"));

    match write_aux_files(&compile_context) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error on first build: {e}");
            return;
        }
    };
    progress.tick();

    let http_container = plugin.resolve_optimized_tree(&compile_context).unwrap();
    progress.tick();

    Manifest::update(&http_container, &plugin, &compile_context).unwrap();
    progress.tick();

    // let views = view_discover(&compile_context);

    progress.finish();
    // for view in views {
    //     process_view(view);
    // }

    println!(
        "\x1B[2J\x1B[1;1H{}\n",
        Fmt(|f| http_container
            .get_root()
            .unwrap()
            .read()
            .unwrap()
            .display(f, &http_container))
    );

    let mut deno = process::Command::new("deno")
        .args(["run", "-A"])
        .arg(format!("{}/.densky/dev.ts", target_path.display()))
        .spawn()
        .expect("deno command failed to run");

    let term = Arc::new(AtomicBool::new(false));
    let sigint =
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term)).unwrap();

    '_loop: loop {
        handle_update(&compile_context, &plugin, &mut watching_poll);

        // wait to interrupt
        if term.load(Ordering::Relaxed) {
            // TODO: Check memory leaks on this line
            assert!(signal_hook::low_level::unregister(sigint));
            let _ = deno.kill(); // Err(): Command wasn't running
            return;
        }

        thread::sleep(Duration::from_millis(200));
    }
}

fn handle_update(
    compile_context: &CompileContext,
    plugin: &CloudPlugin,
    watching_poll: &mut PollWatcher,
) {
    let event = watching_poll.poll();
    if event.len() != 0 {
        let http_container = plugin.resolve_optimized_tree(&compile_context).unwrap();

        // let views = view_discover(&compile_context);
        // for view in views {
        //     process_view(view);
        // }

        match Manifest::update(&http_container, &plugin, &compile_context) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error updating manifest: {err}")
            }
        }

        send_update(event.iter().map(|e| (e.kind.clone(), &e.path)));
    }
}

fn send_update<I, P>(files: I)
where
    I: Iterator<Item = (WatchKind, P)>,
    P: AsRef<OsStr>,
{
    let mut files_json = "[".to_owned();
    for file in files {
        use self::WatchKind::*;
        let kind = match file.0 {
            Create => "create",
            Remove => "remove",
            Modify => "modify",
        };
        files_json += "[\"";
        files_json += kind;
        files_json += "\",\"";
        files_json += file.1.as_ref().to_str().unwrap();
        files_json += "\"],";
    }
    files_json.pop();
    files_json += "]";
    // TODO: print good error
    let res = ureq::post("http://localhost:8000/$/dev")
        .set("Content-Type", "application/json")
        .send_string(&files_json);

    if let Err(err) = res {
        match err {
            ureq::Error::Status(_, _) => (),
            ureq::Error::Transport(err) => println!("[Dev Error] {}", err),
        }
    }
}
