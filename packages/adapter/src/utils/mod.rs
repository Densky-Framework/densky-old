use std::fmt;
use std::path::{Component, Path, PathBuf};

mod color;
pub mod log;
mod url_to_matcher;

pub use color::Color;
pub use url_to_matcher::{UrlMatcher, UrlMatcherSegment};

pub fn relative_path<T: AsRef<Path>, B: AsRef<Path>>(target: T, base: B) -> Option<PathBuf> {
    let relative = pathdiff::diff_paths(target.as_ref(), base.as_ref())?;
    let relative = relative.to_str()?;

    let relative = if &relative.chars().nth(0) == &Some('.') {
        relative.to_string()
    } else {
        format!("./{}", relative)
    };

    Some(relative.into())
}

pub fn normalize_path<T: AsRef<Path>>(target: T) -> String {
    let target = PathBuf::from(target.as_ref());

    // Consider add Windows prefix support
    // ```
    // let path_prefix = std::env::current_dir()
    //     .unwrap()
    //     .components()
    //     .next()
    //     .map(|component| match component {
    //         std::path::Component::Prefix(prefix) => Some(prefix.as_os_str()),
    //         _ => None,
    //     })
    //     .unwrap_or_default();
    // ```

    let mut base = PathBuf::from("/");

    for section in target.iter() {
        match section.to_str().unwrap() {
            "." => {
                continue;
            }
            ".." => {
                base.pop();
            }
            str => base.push(str),
        }
    }

    base.display().to_string()
}

pub fn join_paths<T: AsRef<Path>, B: AsRef<Path>>(target: T, base: B) -> String {
    let target = PathBuf::from(target.as_ref());

    if target.has_root() {
        return target.display().to_string();
    }

    let mut base = PathBuf::from(base.as_ref());

    for section in target.iter() {
        match section.to_str().unwrap() {
            "." => {
                continue;
            }
            ".." => {
                base.pop();
            }
            str => base.push(str),
        }
    }

    base.display().to_string()
}

pub trait ToPosix {
    fn to_posix(&self) -> String;
}

impl<T: AsRef<Path>> ToPosix for T {
    fn to_posix(&self) -> String {
        self.as_ref()
            .components()
            .filter_map(|c| match c {
                Component::Prefix(_) => None,
                Component::CurDir => None,
                Component::RootDir => Some(""),
                Component::Normal(c) => Some(c.to_str().unwrap()),
                Component::ParentDir => Some(".."),
            })
            .collect::<Vec<_>>()
            .join("/")
    }
}

/// Custom formatter
/// # Usage example:
/// ```ignore
/// fn custom_display(f: &mut fmt::Formatter, a: u8, b: u8) -> fmt::Result {
///     writeln!(f, "{a} + {b} = {}", a + b)
/// }
///
/// println!("{}", Fmt(|f| custom_display(f, 2, 3)));
/// println!("{:?}", Fmt(|f| custom_display(f, 2, 3)));
/// ```
pub struct Fmt<F>(pub F)
where
    F: Fn(&mut fmt::Formatter) -> fmt::Result;

impl<F> fmt::Debug for Fmt<F>
where
    F: Fn(&mut fmt::Formatter) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self.0)(f)
    }
}

impl<F> fmt::Display for Fmt<F>
where
    F: Fn(&mut fmt::Formatter) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self.0)(f)
    }
}

pub trait StringStripExtend {
    fn strip_prefix_if_can<'a, P>(&'a self, prefix: P) -> &'a str
    where
        P: AsRef<str>;
}

impl StringStripExtend for String {
    fn strip_prefix_if_can<'a, P>(&'a self, prefix: P) -> &'a str
    where
        P: AsRef<str>,
    {
        match self.strip_prefix(prefix.as_ref()) {
            Some(stripped) => stripped,
            None => self.as_str(),
        }
    }
}

impl StringStripExtend for str {
    fn strip_prefix_if_can<'a, P>(&'a self, prefix: P) -> &'a str
    where
        P: AsRef<str>,
    {
        match self.strip_prefix(prefix.as_ref()) {
            Some(stripped) => stripped,
            None => self,
        }
    }
}
