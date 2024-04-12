use super::_macro::def_command;
use std::env;
use std::{
    path::{Path, PathBuf},
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
use densky_core::densky_adapter::{log_warn, CloudVersion};
use densky_core::sky::search_cloud;
use densky_core::{
    anyhow,
    densky_adapter::{log_error, utils::join_paths},
    sky::CloudPlugin,
    CompileContext, ConfigFile, Manifest, Result,
};

def_command!(DevCommand("dev") {
    [folder]("Proyect folder") {
        default_value: ".",
        value_hint: ValueHint::DirPath,
        value_parser: value_parser!(PathBuf),
    },

    process: process
});

fn process(matches: &clap::ArgMatches) -> Result<()> {
    let folder = matches.get_one::<PathBuf>("folder").unwrap();
    let cwd = std::env::current_dir()?;
    let target_path: PathBuf = join_paths(folder, cwd).into();

    let config_file = ConfigFile::discover(&target_path)?;

    println!("{config_file:#?}");

    let watching_path = target_path.clone();
    let mut watching_poll = PollWatcher::new(watching_path)?;

    let compile_context = CompileContext {
        output_dir: config_file.output.display().to_string(),
        cwd: target_path.display().to_string(),
        verbose: true,
    };

    let clouds = &config_file.dependencies;
    let progress = progress::create_bar(clouds.len(), "Loading clouds");
    let mut loaded_clouds: Vec<CloudPlugin> = Vec::new();

    let densky_installation = env::var("DENSKY_INSTALL").unwrap_or_else(|_| {
        env::var("HOME")
            .map(|x| format!("{x}/.densky"))
            .unwrap_or_default()
    });
    let densky_installation: PathBuf = densky_installation.into();
    let cloud_search_entries = [vec![densky_installation], config_file.vendor.clone()].concat();

    for cloud in clouds.values() {
        progress.set_message(cloud.name.clone());
        let cloud_libname = format!("cloud_{}", cloud.name.replace("-", "_"));

        let cloud_path = match &cloud.version {
            CloudVersion::Path(p) => join_paths(&p, &target_path).into(),
            CloudVersion::Semver(_) => {
                // TODO: Implement version requirement
                log_warn!(["TODO"] "Ignoring version requirements");
                search_cloud(&cloud.name, &cloud_search_entries)
                    .ok_or(anyhow!("Can't find cloud"))?
            }
            CloudVersion::Unknown(_) => unreachable!(),
        };

        let mut cloud = CloudPlugin::new(cloud_libname, cloud_path)?;
        cloud.setup()?;
        loaded_clouds.push(cloud);

        progress.tick();
    }

    progress.finish();

    let progress = progress::create_spinner(Some("Discovering"));

    match write_aux_files(&compile_context, &config_file) {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Error on first build: {e}"));
        }
    };
    progress.tick();

    for cloud in loaded_clouds.iter() {
        let http_container = cloud.resolve_optimized_tree(&compile_context)?;

        Manifest::update(&http_container, &cloud, &compile_context)?;
        progress.tick();
    }

    progress.finish();

    let Ok(mut deno) = process::Command::new("deno")
        .args(["run", "-A"])
        .arg(format!("{}/.densky/dev.ts", target_path.display()))
        .spawn()
    else {
        log_error!(["RUNTIME"] "Deno command failed to run.\nCheck your Deno installation:\n > deno --version");
        std::process::exit(1);
    };

    let term = Arc::new(AtomicBool::new(false));
    let sigint =
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term)).unwrap();

    '_loop: loop {
        let event = watching_poll.poll();
        if event.len() != 0 {
            for cloud in loaded_clouds.iter() {
                let http_container = cloud.resolve_optimized_tree(&compile_context)?;

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
            return Ok(());
        }

        thread::sleep(Duration::from_millis(200));
    }
}

fn send_update<I, P>(files: I)
where
    I: Iterator<Item = (WatchKind, P)>,
    P: AsRef<Path>,
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
        files_json += &file.1.as_ref().display().to_string();
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
