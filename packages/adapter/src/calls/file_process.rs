use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CloudFile {
    pub file_path: PathBuf,
    pub relative_path: String,
    pub output_path: PathBuf,
}

impl CloudFile {
    pub fn new(
        full_path: impl AsRef<str>,
        relative_path: impl AsRef<str>,
        output_path: impl AsRef<str>,
    ) -> CloudFile {
        CloudFile {
            file_path: full_path.as_ref().into(),
            relative_path: relative_path.as_ref().into(),
            output_path: output_path.as_ref().into(),
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CloudFileResolve {
    /// Pass through as regular static node
    #[default]
    Pass = 0,

    /// Skip the node
    Ignore,

    /// Set parent leaf node
    Index,
    /// It is used to mark a dynamic path.
    ///
    /// ## Structure
    /// ```ignore
    /// CloudFileResolve::Dynamic(prefix, var, suffix)
    /// ```
    /// # Example
    /// ```ignore
    /// // Ideal fit
    /// let my_pathname = "api/$version/swagger";
    /// CloudFileResolve::Dynamic("api", "version", "swagger");
    ///
    /// // Without prefix
    /// let my_pathname = "$version/swagger";
    /// CloudFileResolve::Dynamic("", "version", "swagger");
    ///
    /// // Without suffix
    /// let my_pathname = "api/$version";
    /// CloudFileResolve::Dynamic("api", "version", "");
    ///
    /// // Multi-var
    /// let my_pathname = "api/$version/u/$user";
    /// CloudFileResolve::Dynamic("api", "version", "u/$user"); // It's recursive
    /// ```
    Dynamic(String, String, String),
    SingleThorn(&'static str),
    MultiThorn(&'static str),
}
