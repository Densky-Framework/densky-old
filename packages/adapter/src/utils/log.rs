use std::sync::RwLock;

static mut LAST_MESSAGE: RwLock<String> = RwLock::new(String::new());
static mut REPEATED_TIMES: RwLock<u16> = RwLock::new(0);

const LOG_FILTER_ENV: &str = "DENSKY_LOG_FILTER";
static mut LOG_FILTER: RwLock<LogFilter> = RwLock::new(LogFilter::Unset);

const LOG_LEVEL_ENV: &str = "DENSKY_LOG";
static mut LOG_LEVEL: RwLock<LogLevel> = RwLock::new(LogLevel::Unset);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Unset,
    None,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<&str> for LogLevel {
    fn from(value: &str) -> Self {
        match value {
            "n" | "no" | "none" => LogLevel::None,
            "e" | "err" | "error" => LogLevel::Error,
            "w" | "wrn" | "warn" => LogLevel::Warn,
            "i" | "inf" | "info" => LogLevel::Info,
            "d" | "dbg" | "debug" => LogLevel::Debug,
            "t" | "trc" | "trace" => LogLevel::Trace,
            _ => {
                log_info!(["LOGGER"] "Unrecognized log level. Using info level as default.");
                LogLevel::Info
            }
        }
    }
}

#[derive(Clone, PartialEq)]
enum LogFilter {
    Unset,
    None,
    All,
    List(Vec<String>),
}

#[macro_export(local_inner_macros)]
macro_rules! _log {
    (($level:ident, $label:expr)[$title_color:expr, $label_fmt:expr] [$body_color:expr] $($body:tt)*) => {'log: {
        use $crate::utils::Color::*;

        let label = ::std::format!("{}", $label);
        if !$crate::log::_log_must_show($crate::log::LogLevel::$level, &label) {
            break 'log;
        }

        let formatted = ::std::format!(
            "{} {}",
            ($title_color).color(::std::format!($label_fmt, label)),
            ($body_color).color(::std::format!($($body)*))
        );
        let repeated_times = $crate::log::_log_is_repeated(&formatted);
        if repeated_times >= 1 {
            ::std::println!(
                "\x1b[1A{formatted}    {Dim}x{repeated_times}{Reset}",
            );
        } else {
            ::std::println!("{formatted}");
        }
    }};
}

#[macro_export(local_inner_macros)]
macro_rules! log_error {
    ([$title:expr] ($body_color:ident) $($body:tt)*) => {
        $crate::log::_log!((Error, $title)[FgRed | Bold, "[{}]"] [$body_color] $($body)*);
    };
    ([$title:expr] $($body:tt)*) => {
        $crate::log::_log!((Error, $title)[FgRed | Bold, "[{}]"] [FgYellow] $($body)*);
    };
}

#[macro_export(local_inner_macros)]
macro_rules! log_warn {
    ([$title:expr] ($body_color:ident) $($body:tt)*) => {
        $crate::log::_log!((Warn, $title)[FgYellow | Bold, "[{}]"] [$body_color] $($body)*);
    };
    ([$title:expr] $($body:tt)*) => {
        $crate::log::_log!((Warn, $title)[FgYellow | Bold, "[{}]"] [FgYellow] $($body)*);
    };
}

#[macro_export(local_inner_macros)]
macro_rules! log_info {
    ([$title:expr] ($body_color:ident) $($body:tt)*) => {
        $crate::log::_log!((Info, $title)[FgBlue, "[{}]"] [$body_color] $($body)*);
    };
    ([$title:expr] $($body:tt)*) => {
        $crate::log::_log!((Info, $title)[FgBlue, "[{}]"] [Reset] $($body)*);
    };
}

#[macro_export(local_inner_macros)]
macro_rules! log_debug {
    ([$title:expr] ($body_color:ident) $($body:tt)*) => {
        $crate::log::_log!((Debug, $title)[FgBlue | Dim | Italic, "[{}]"] [$body_color | Dim] $($body)*);
    };
    ([$title:expr] $($body:tt)*) => {
        $crate::log::_log!((Debug, $title)[FgBlue | Dim | Italic, "[{}]"] [Dim] $($body)*);
    };
}

#[macro_export(local_inner_macros)]
macro_rules! log_trace {
    ([$title:expr] ($body_color:ident) $($body:tt)*) => {
        $crate::log::_log!((Trace, $title)[FgBlue | Dim | Italic, "[{}]"] [$body_color | Dim] $($body)*);
    };
    ([$title:expr] $($body:tt)*) => {
        $crate::log::_log!((Trace, $title)[FgBlue | Dim | Italic, "[{}]"] [Dim] $($body)*);
    };
}

pub fn _log_is_repeated(new_message: &String) -> u16 {
    let last_message = unsafe { LAST_MESSAGE.read().unwrap() };

    if &*last_message == new_message {
        let repeated_times = unsafe { REPEATED_TIMES.get_mut().unwrap() };
        *repeated_times += 1;
        *repeated_times
    } else {
        drop(last_message);
        let last_message = unsafe { LAST_MESSAGE.get_mut().unwrap() };
        *last_message = new_message.clone();
        let repeated_times = unsafe { REPEATED_TIMES.get_mut().unwrap() };
        *repeated_times = 0;
        0
    }
}

pub fn _log_must_show(target_level: LogLevel, label: &String) -> bool {
    let log_level = get_level();
    if target_level.gt(&log_level) {
        return false;
    }

    let filter = get_filter();
    match filter {
        LogFilter::None => false,
        LogFilter::All => true,
        LogFilter::List(filter_labels) => filter_labels
            .iter()
            .find(|l| label.starts_with(*l))
            .is_some(),
        _ => unreachable!("Unset variant is handled above"),
    }
}

fn get_level() -> LogLevel {
    let level = unsafe { LOG_LEVEL.read().unwrap() };
    if LogLevel::Unset == *level {
        drop(level);
        let level_env = std::env::var(LOG_LEVEL_ENV);
        unsafe {
            let level = if let Ok(level) = level_env {
                LogLevel::from(level.to_lowercase().as_str())
            } else {
                LogLevel::Info
            };
            let mut level_writer = LOG_LEVEL.write().unwrap();
            *level_writer = level;

            level
        }
    } else {
        *level
    }
}

fn get_filter() -> LogFilter {
    let filter = unsafe { LOG_FILTER.read().unwrap() };
    if LogFilter::Unset == *filter {
        drop(filter);
        let filter_env = std::env::var(LOG_FILTER_ENV);
        unsafe {
            let filter = if let Ok(filter) = filter_env {
                if filter.to_lowercase() == "None" {
                    LogFilter::None
                } else {
                    let filter_list = filter
                        .split(',')
                        .map(|l| l.to_owned())
                        .collect::<Vec<String>>();
                    LogFilter::List(filter_list)
                }
            } else {
                LogFilter::All
            };

            let mut filter_writer = LOG_FILTER.write().unwrap();
            *filter_writer = filter.clone();

            filter
        }
    } else {
        filter.clone()
    }
}

pub trait PathDebugDisplay {
    fn display_debug(&self) -> String;
}

impl PathDebugDisplay for std::path::PathBuf {
    fn display_debug(&self) -> String {
        if !self.has_root() {
            return self.display().to_string();
        }
        let cwd = std::env::current_dir().unwrap();
        let path = match self.strip_prefix(cwd) {
            Ok(p) => p,
            Err(_) => return self.display().to_string(),
        };
        format!("!/{}", path.display())
    }
}

impl PathDebugDisplay for std::path::Path {
    fn display_debug(&self) -> String {
        if !self.has_root() {
            return self.display().to_string();
        }
        let cwd = std::env::current_dir().unwrap();
        let path = match self.strip_prefix(cwd) {
            Ok(p) => p,
            Err(_) => return self.display().to_string(),
        };
        format!("!/{}", path.display())
    }
}

impl<T: PathDebugDisplay> PathDebugDisplay for Option<T> {
    fn display_debug(&self) -> String {
        format!("{:?}", self.as_ref().map(|p| p.display_debug()))
    }
}

pub use _log;
pub use log_debug;
pub use log_error;
pub use log_info;
pub use log_trace;
pub use log_warn;
