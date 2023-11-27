/// ```
/// # use std::path::PathBuf;
/// # use clap::{ValueHint, value_parser};
/// #
/// def_command!(MyCommand("cmd") {
///     <path>("Project path") {
///         value_hint: ValueHint::DirPath,
///         value_parser: value_parser!(PathBuf)
///     },
///     [output_dir]("Project output") {
///         default_value: ".output",
///         value_hint: ValueHint::DirPath,
///         value_parser: value_parser!(PathBuf),
///     },
///     --debug(-d, "Debug level") {
///         default_value: 0,
///         value_parser: value_parser!(u8)
///     },
///     command(cmd) {
///         cmd
///     },
///     process: process
/// });
/// ```
#[macro_export]
macro_rules! def_command {
    ($name:ident ( $cmd_name:expr ) {
        $(
        <$req_arg:ident>($req_arg_desc:expr) {
            $($req_arg_key:ident : $req_arg_value:expr ,)*
            $(|$req_arg_ident:ident| $req_arg_decl:expr )?
        }$(,)?
        )*
        $(
        [$arg:ident]($arg_desc:expr) {
            $($arg_key:ident : $arg_value:expr ,)*
            $(|$arg_ident:ident| $arg_decl:expr )?
        }$(,)?
        )*
        $(
        --$flag:ident($(-$flag_short:ident,)? $(=$flag_value:ident,)? $flag_desc:expr) {
            $($flag_key:ident : $flag_kvalue:expr ,)*
            $(|$flag_ident:ident| $flag_decl:expr )?
        }$(,)?
        )*
        $(command( $cmd:ident ) {
            $cmd_decl:expr
        }$(,)?)?
        $(process: $process:ident)? $(,)?
    }) => {
        pub struct $name;

        impl $name {
            pub fn command() -> clap::Command {
                let cmd = clap::Command::new($cmd_name);
                $(
                let cmd = {
                    let $req_arg = clap::arg!(<$req_arg> $req_arg_desc);
                    $(
                    let $req_arg = clap::Arg::$req_arg_key($req_arg, $req_arg_value);
                    )*
                    $(
                    let $req_arg_ident = $req_arg;
                    let $req_arg = $req_arg_decl;
                    )?
                    cmd.arg($req_arg)
                };
                )*

                $(
                let cmd = {
                    let $arg = clap::arg!([$arg] $arg_desc);
                    $(
                    let $arg = clap::Arg::$arg_key($arg, $arg_value);
                    )*
                    $(
                    let $arg_ident = $arg;
                    let $arg = $arg_decl;
                    )?
                    cmd.arg($arg)
                };
                )*

                $(
                let cmd = {
                    let $flag = clap::arg!(--$flag $(-$flag_short)? $(<$flag_value>)? $flag_desc);
                    $(
                    let $flag = clap::Arg::$flag_key($flag, $flag_kvalue);
                    )*
                    $(
                    let $flag_ident = $flag;
                    let $flag = $flag_decl;
                    )?
                    cmd.arg($flag)
                };
                )*
                $(
                let $cmd = cmd;
                let cmd = $cmd_decl;
                )?
                cmd
            }

            pub fn process( matches: &::clap::ArgMatches ) {
                #[allow(unused)]
                let r: densky_core::Result<()> = Ok(());
                $(let r = $process(matches);)?

                if let Err(err) = r {
                    eprintln!("{err:?}");
                }
            }
        }
    };
}

pub use def_command;
