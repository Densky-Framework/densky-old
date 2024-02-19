use std::path::PathBuf;
use std::str::FromStr;

use ahash::AHashMap;

#[derive(Debug)]
pub struct CloudSetup {
    pub name: String,
    pub version: String,
    pub source_folder: String,
    pub file_starts: Option<String>,
    pub file_ends: Option<String>,
    pub file_strategy: CloudFilesStrategy,
    pub dependencies: Vec<CloudDependency>,
}

#[derive(Debug, Clone)]
pub enum CloudVersion {
    Semver(semver::VersionReq),
    Path(PathBuf),
    Unknown(String),
}

impl From<&str> for CloudVersion {
    fn from(value: &str) -> Self {
        let version = semver::VersionReq::parse(&value);
        if let Ok(version) = version {
            CloudVersion::Semver(version)
        } else if let Ok(path) = PathBuf::from_str(&value) {
            CloudVersion::Path(path)
        } else {
            CloudVersion::Unknown(value.to_string())
        }
    }
}
impl From<String> for CloudVersion {
    fn from(value: String) -> Self {
        let version = semver::VersionReq::parse(&value);
        if let Ok(version) = version {
            CloudVersion::Semver(version)
        } else if let Ok(path) = PathBuf::from_str(&value) {
            CloudVersion::Path(path)
        } else {
            CloudVersion::Unknown(value)
        }
    }
}

#[derive(Debug, Clone)]
pub struct CloudDependency {
    pub name: String,
    pub version: CloudVersion,
    pub optional: bool,

    pub options: AHashMap<String, CloudDependencyOption>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CloudDependencyOption {
    /// A string value.
    String(String),
    /// A 64-bit integer value.
    Integer(i64),
    /// A 64-bit float value.
    Float(f64),
    /// A boolean value.
    Boolean(bool),
    /// An inline array of values.
    Array(Vec<CloudDependencyOption>),
}

impl From<Vec<CloudDependencyOption>> for CloudDependencyOption {
    fn from(v: Vec<CloudDependencyOption>) -> Self {
        Self::Array(v)
    }
}

impl From<bool> for CloudDependencyOption {
    fn from(v: bool) -> Self {
        Self::Boolean(v)
    }
}

impl From<f64> for CloudDependencyOption {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<i64> for CloudDependencyOption {
    fn from(v: i64) -> Self {
        Self::Integer(v)
    }
}

impl From<String> for CloudDependencyOption {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl CloudDependencyOption {
    pub fn as_string(&self) -> Option<&String> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_integer(&self) -> Option<&i64> {
        if let Self::Integer(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<&f64> {
        if let Self::Float(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_boolean(&self) -> Option<&bool> {
        if let Self::Boolean(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_array(&self) -> Option<&Vec<CloudDependencyOption>> {
        if let Self::Array(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CloudFilesStrategy {
    #[default]
    None = 0,
    SimpleTree,
    OptimizedTree,
}
