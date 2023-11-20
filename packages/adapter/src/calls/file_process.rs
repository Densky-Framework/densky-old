use std::ops;
use std::path::PathBuf;

use ahash::AHashMap;

/// This is the minimum unit for a Optimized Tree.
/// This is used for transport basic data like file paths (i/o)
/// between the core and plugins
#[derive(Debug)]
pub struct OptimizedTreeLeaf {
    pub pathname: String,
    pub relative_pathname: String,

    pub index: Option<String>,

    /// Map<Name, Vec<FilePath>>
    pub single_thorns: AHashMap<String, Vec<String>>,

    pub is_root: bool,
    pub is_static: bool,
    pub varname: Option<String>,
}

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

    /// Remove the last part
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

    /// Convert route to a `thorn` (marker) with the provided name.
    ///
    /// > This checks if already exists this thorn, error expected.
    /// > Remember that we check the name of thorn.
    ///
    /// ## Structure
    /// ```ignore
    /// CloudFileResolve::SingleThorn(name)
    /// ```
    ///
    /// ## Example
    /// ```ignore
    /// CloudFileResolve::SingleThorn("middleware");
    /// CloudFileResolve::SingleThorn("fallback");
    ///
    /// // If try to create the same thorn on the same route
    /// // It will create a conflict, so we show an error
    /// CloudFileResolve::SingleThorn("middleware");
    /// // > [OptimizedTree] Conflicting thorn "middleware". Inserting "xxx" when "xxx" already is
    /// //   "middleware" thorn.
    ///
    /// // When you get it
    /// let middlewares = container.single_thorn.get_all("middleware", self.absolute_path);
    /// // : Vec<u64>
    /// for middleware in middlewares {
    ///     println!("Middleware ID: {middleware}");
    /// }
    /// ```
    SingleThorn(&'static str),

    /// Convert route to a `thorn` (marker) with the provided name.
    ///
    /// > This thorn can be repeated.
    ///
    /// ## Structure
    /// ```ignore
    /// CloudFileResolve::MultiThorn(name)
    /// ```
    ///
    /// ## Example
    /// ```ignore
    /// CloudFileResolve::MultiThorn("custom_handler");
    /// CloudFileResolve::MultiThorn("other_handler");
    /// CloudFileResolve::MultiThorn("custom_handler"); // Valid
    ///
    /// // When you get them
    /// let custom_handlers = container.multi_thorn.get("custom_handler", self.absolute_path);
    /// // : Vec<u64>
    /// for custom_handler in custom_handlers {
    ///     println!("Custom handler ID: {custom_handler}");
    /// }
    /// ```
    MultiThorn(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudManifestUpdate {
    arguments: AHashMap<String, String>,
    content: Option<String>,
    imports: AHashMap<String, String>,
}

impl CloudManifestUpdate {
    pub fn new() -> Self {
        Self {
            arguments: AHashMap::new(),
            content: None,
            imports: AHashMap::new(),
        }
    }

    pub fn new_content(content: impl Into<String>) -> Self {
        Self {
            arguments: AHashMap::new(),
            content: Some(content.into()),
            imports: AHashMap::new(),
        }
    }

    pub fn set_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn append_content(mut self, content: impl AsRef<str>) -> Self {
        match self.content.as_mut() {
            Some(c) => *c += content.as_ref(),
            None => self.content = Some(content.as_ref().into()),
        }

        self
    }

    pub fn add_import(mut self, items: impl Into<String>, path: impl Into<String>) -> Self {
        self.imports.insert(path.into(), items.into());
        self
    }

    pub fn add_argument(mut self, name: impl Into<String>, def: impl Into<String>) -> Self {
        self.arguments.insert(name.into(), def.into());
        self
    }

    pub fn arguments(&self) -> &AHashMap<String, String> {
        &self.arguments
    }

    pub fn content(&self) -> Option<&String> {
        self.content.as_ref()
    }

    pub fn imports(&self) -> &AHashMap<String, String> {
        &self.imports
    }
}

impl ops::AddAssign<&CloudManifestUpdate> for CloudManifestUpdate {
    fn add_assign(&mut self, rhs: &CloudManifestUpdate) {
        for (path, items) in rhs.imports() {
            let a = self.imports.get_mut(path);

            match a {
                Some(a) => *a += items,
                None => {
                    self.imports.insert(path.into(), items.into());
                }
            }
        }

        for (arg, def) in rhs.arguments() {
            let old_def = self.arguments.get(arg);

            if let Some(old_def) = old_def {
                if def != old_def {
                    panic!("Conflicting argument {}. {:#?} != {:#?}", arg, def, old_def);
                }
            } else {
                self.arguments.insert(arg.into(), def.into());
            }
        }
    }
}

impl ops::AddAssign<CloudManifestUpdate> for CloudManifestUpdate {
    fn add_assign(&mut self, rhs: CloudManifestUpdate) {
        for (path, items) in rhs.imports() {
            let a = self.imports.get_mut(path);

            match a {
                Some(a) => *a += items,
                None => {
                    self.imports.insert(path.into(), items.into());
                }
            }
        }

        for (arg, def) in rhs.arguments() {
            let old_def = self.arguments.get(arg);

            if let Some(old_def) = old_def {
                if def != old_def {
                    panic!("Conflicting argument {}. {:#?} != {:#?}", arg, def, old_def);
                }
            } else {
                self.arguments.insert(arg.into(), def.into());
            }
        }
    }
}
