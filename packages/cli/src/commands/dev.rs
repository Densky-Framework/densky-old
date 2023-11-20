use super::_macro::def_command;
use std::{
    ffi::OsStr,
    fs,
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
    densky_adapter::utils::join_paths, sky::CloudPlugin, CompileContext, ConfigFile, Manifest,
};

def_command!(DevCommand("dev") {
    [folder]("Proyect folder") {
        default_value: ".",
        value_hint: ValueHint::DirPath,
        value_parser: value_parser!(PathBuf),
    },

    process: process
});

fn process(matches: &clap::ArgMatches) {
    let folder = matches.get_one::<PathBuf>("folder").unwrap();
    let cwd = std::env::current_dir().unwrap();
    let target_path: PathBuf = join_paths(folder, cwd).into();

    let config_file = fs::read_to_string(join_paths("densky.toml", &target_path)).unwrap();
    let config_file = ConfigFile::parse(config_file, &target_path).unwrap();

    println!("{config_file:#?}");

    let watching_path = target_path.clone();
    let mut watching_poll = PollWatcher::new(watching_path).unwrap();

    let compile_context = CompileContext {
        output_dir: config_file.output.display().to_string(),
        cwd: target_path.display().to_string(),
        verbose: true,
    };

    let clouds = &config_file.dependencies;
    let progress = progress::create_bar(clouds.len(), "Loading clouds");
    let mut loaded_clouds: Vec<CloudPlugin> = Vec::new();

    for cloud in clouds.values() {
        progress.set_message(cloud.name.clone());
        let cloud_libname = format!("cloud_{}", cloud.name.replace("-", "_"));
        let cloud_path = join_paths(&cloud.version, &target_path);
        let mut cloud = CloudPlugin::new(cloud_libname, cloud_path).unwrap();
        cloud.setup();
        loaded_clouds.push(cloud);

        progress.tick();
    }

    progress.finish();

    let progress = progress::create_spinner(Some("Discovering"));

    match write_aux_files(&compile_context, &config_file) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error on first build: {e}");
            return;
        }
    };
    progress.tick();

    for cloud in loaded_clouds.iter() {
        let http_container = cloud.resolve_optimized_tree(&compile_context).unwrap();

        Manifest::update(&http_container, &cloud, &compile_context).unwrap();
    }

    progress.finish();

    let mut deno = process::Command::new("deno")
        .args(["run", "-A"])
        .arg(format!("{}/.densky/dev.ts", target_path.display()))
        .spawn()
        .expect("deno command failed to run");

    let term = Arc::new(AtomicBool::new(false));
    let sigint =
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term)).unwrap();

    '_loop: loop {
        let event = watching_poll.poll();
        if event.len() != 0 {
            for cloud in loaded_clouds.iter() {
                let http_container = cloud.resolve_optimized_tree(&compile_context).unwrap();

                match Manifest::update(&http_container, &cloud, &compile_context) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("Error updating manifest: {err}")
                    }
                }
            }

            send_update(event.iter().map(|e| (e.kind.clone(), &e.path)));
        }

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
