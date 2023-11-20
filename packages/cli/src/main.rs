extern crate ahash;
extern crate anstyle;
extern crate clap;
extern crate densky_core;
extern crate indicatif;
extern crate ureq;

pub mod commands;
pub mod compiler;
pub mod progress;
pub mod watcher;

use self::commands::DevCommand;
use anstyle::{AnsiColor, Color, Style};
use clap::{builder::Styles, command};

// use crate::commands::DevCommand;

fn main() {
    #[allow(unused_mut)]
    let mut command = command!()
        .name("Denky CLI")
        .author("ApikaLuca")
        .propagate_version(true)
        .help_template(
            "\
{before-help}{name} v{version} ({author})
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
",
        )
        .styles(
            Styles::styled()
                .header(Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlue))))
                .error(Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightRed))))
                .usage(Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlue))))
                .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightYellow))))
                .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow))))
                .valid(Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightGreen))))
                .invalid(Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightRed)))),
        )
        .subcommand(DevCommand::command());
    // .subcommand(BuildCommand::command())
    // .subcommand(PluginTestCommand::command());

    #[cfg(not(debug_assertions))]
    {
        command = command.help_expected(true);
    };

    let matches = command.get_matches();

    match matches.subcommand() {
        Some(("dev", sub_matches)) => DevCommand::process(sub_matches),
        // Some(("build", sub_matches)) => BuildCommand::process(sub_matches),
        // Some(("plugin-test", sub_matches)) => PluginTestCommand::process(sub_matches),
        Some((cmd_name, _)) => println!("Unknown command: {cmd_name}"),
        None => todo!("Main entry"),
    }
}
