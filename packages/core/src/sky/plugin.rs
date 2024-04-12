use densky_adapter::{
    anyhow, log_info, CloudBeforeManifestCall, CloudFilesStrategy, CloudManifestUpdate,
    CloudOptimizedManifestCall, ErrorContext, OptimizedTreeLeaf, Result,
};
use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};

use densky_adapter::{
    context::CloudContextRaw, log_trace, thiserror, CloudContextCall, CloudDebugContextCall,
    CloudFile, CloudFileResolve, CloudFileResolveCall, CloudSetup, CloudSetupCall,
};

use crate::optimized_tree::{optimized_tree_strategy, OptimizedTreeContainer};
use crate::CompileContext;

use super::open_cloud;

macro_rules! get_cloud_call {
    ($self:ident, $call:ident) => {
        $self.get_cloud_call::<$call::Fn>($call::SYMBOL)
    };
}

#[derive(Debug, thiserror::Error)]
pub enum CloudPluginError {
    #[error("Malformed input: {0}")]
    MalformedInput(&'static str),

    #[error(transparent)]
    Lib(#[from] libloading::Error),
}

#[derive(Debug)]
pub struct CloudPlugin {
    pub name: String,
    lib: Library,
    setup: Option<CloudSetup>,
    context: Option<CloudContextRaw>,
}

impl CloudPlugin {
    pub fn close(self) {
        self.lib.close().expect("I wanna cry");
    }
}

impl CloudPlugin {
    pub fn new(
        libname: String,
        lib_path: impl AsRef<Path>,
    ) -> Result<CloudPlugin, CloudPluginError> {
        unsafe {
            let lib = open_cloud(&libname, lib_path)?;
            Ok(CloudPlugin {
                name: libname,
                lib,
                setup: None,
                context: None,
            })
        }
    }

    pub fn get_setup(&self) -> Result<&CloudSetup> {
        match self.setup.as_ref() {
            Some(s) => Ok(s),
            None => Err(anyhow!("Using setup data before the cloud setup")),
        }
    }

    /// Get call symbol from loaded plugin library.
    /// ```ignore
    /// let mut plugin = CloudPlugin::new(lib_path).unwrap();
    /// let plugin_setup = plugin.get_cloud_call::<CloudSetupFn>(b"cloud_setup");
    /// plugin_setup();
    /// ```
    pub unsafe fn get_cloud_call<T>(&self, name: &[u8]) -> Result<Symbol<T>> {
        // File resolve call is called many times and fill all the screen
        // with their logging
        let omit_debug = name == CloudFileResolveCall::SYMBOL;

        let name_string = String::from_utf8_lossy(name).to_owned();

        if !omit_debug {
            log_trace!([self.name] "Getting call: {name_string:?}");
        }

        self.lib.get::<T>(name).map_err(|err| {
            log_trace!([self.name] "Can't get call {name_string:?}: {err:#?}");
            anyhow!("Can't get call {name_string:?}: {err:#?}")
        })
    }

    pub unsafe fn cloud_setup(&mut self) -> Result<()> {
        let lib_setup = get_cloud_call!(self, CloudSetupCall)?;
        let lib_setup = lib_setup()?;
        // log_info!([self.name] "Setup: {lib_setup:#?}");
        self.name = lib_setup.name.clone();
        self.setup = Some(lib_setup);
        Ok(())
    }

    pub unsafe fn cloud_context(&mut self) {
        let Ok(lib_context) = get_cloud_call!(self, CloudContextCall) else {
            self.context = None;
            return;
        };
        self.context = lib_context().ok();
    }

    pub unsafe fn cloud_debug_context(&mut self) {
        let Some(context) = &self.context else {
            log_info!([self.name] (FgYellow) "No context");
            return;
        };

        let Ok(lib_call) = get_cloud_call!(self, CloudDebugContextCall) else {
            return;
        };

        lib_call(*context);
    }

    pub unsafe fn cloud_file_resolve(&self, file: CloudFile) -> Result<CloudFileResolve> {
        let filename: PathBuf = file.relative_path.clone().into();
        let filename = filename
            .file_name()
            .with_context(|| format!("Unable to get file name of {}", filename.display()))?;
        let filename = filename
            .to_str()
            .with_context(|| format!("Unable to convert str \"{}\"", filename.to_string_lossy()))?;

        let Some(setup) = self.setup.as_ref() else {
            return Err(anyhow!("`file_resolve` was called before `setup`"));
        };

        if let Some(file_starts) = &setup.file_starts {
            if !filename.starts_with(file_starts) {
                return Ok(CloudFileResolve::Ignore);
            }
        }
        if let Some(file_ends) = &setup.file_ends {
            if !filename.ends_with(file_ends) {
                return Ok(CloudFileResolve::Ignore);
            }
        }

        let lib_call = get_cloud_call!(self, CloudFileResolveCall)?;
        lib_call(file, self.context.unwrap_or_else(CloudContextRaw::null))
    }

    pub unsafe fn cloud_before_manifest(&self) -> Result<CloudManifestUpdate> {
        let lib_call = get_cloud_call!(self, CloudBeforeManifestCall)?;
        lib_call()
    }

    pub unsafe fn cloud_optimized_manifest_call(
        &self,
        leaf: OptimizedTreeLeaf,
        static_children: String,
        children: String,
        dynamic_child: String,
    ) -> Result<CloudManifestUpdate> {
        let lib_call = get_cloud_call!(self, CloudOptimizedManifestCall)?;
        lib_call(leaf, static_children, children, dynamic_child)
    }

    pub fn setup(&mut self) -> Result<()> {
        unsafe {
            self.cloud_setup()?;
            self.cloud_context();
            Ok(())
        }
    }

    // pub fn file_resolve(&mut self, ctx: &CompileContext) -> Result<()> {
    //     use densky_adapter::CloudFilesStrategy::*;
    //     let setup = self.get_setup();
    //     match setup.file_strategy {
    //         None => println!("NONE FILE STRATEGY"),
    //         SimpleTree => println!("SimpleTree FILE STRATEGY"),
    //         OptimizedTree => {
    //             let input_paths = std::env::current_dir().unwrap().join("src");
    //             let input_paths = input_paths.join(setup.source_folder.clone());
    //             let (container, tree) = optimized_tree_strategy(input_paths, self, ctx);
    //         }
    //     }
    //     Some(())
    // }

    pub fn resolve_optimized_tree(&self, ctx: &CompileContext) -> Result<OptimizedTreeContainer> {
        let setup = self.get_setup()?;
        if setup.file_strategy != CloudFilesStrategy::OptimizedTree {
            return Err(anyhow!(
                "Incompatible call. Plugin `{}` is using file strategy {:?}",
                self.name,
                setup.file_strategy
            ));
        }

        let input_paths = std::env::current_dir()?.join("src");
        let input_paths = input_paths.join(setup.source_folder.clone());
        let (container, _) = optimized_tree_strategy(input_paths, self, ctx)?;

        // println!(
        //     "{:#?}",
        //     Fmt(|f| tree.read().unwrap().display(f, &container))
        // );

        Ok(container)
    }
}
