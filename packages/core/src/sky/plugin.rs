use densky_adapter::{CloudFilesStrategy, CloudOptimizedTreeProcessCall, OptimizedTreeLeaf};
use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};

use densky_adapter::{
    context::CloudContextRaw, log_info, log_trace, log_warn, utils::Fmt, CloudContextCall,
    CloudDebugContextCall, CloudFile, CloudFileResolve, CloudFileResolveCall, CloudSetup,
    CloudSetupCall,
};

use crate::optimized_tree::{optimized_tree_strategy, OptimizedTreeContainer};
use crate::CompileContext;

use super::{open_cloud, try_call};

macro_rules! get_cloud_call {
    ($self:ident, $call:ident) => {
        $self.get_cloud_call::<$call::Fn>($call::SYMBOL)
    };
}

#[derive(Debug)]
pub enum CloudPluginError {
    MalformedInput(&'static str),
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
    pub fn new(lib_path: impl AsRef<Path>) -> Result<CloudPlugin, CloudPluginError> {
        let lib_path = lib_path.as_ref();
        let libname = lib_path
            .file_name()
            .ok_or_else(|| CloudPluginError::MalformedInput("lib_path"))?;
        let dir_path = lib_path
            .parent()
            .ok_or_else(|| CloudPluginError::MalformedInput("lib_path"))?;

        unsafe {
            let lib = open_cloud(libname, dir_path);
            Ok(CloudPlugin {
                name: libname.to_str().unwrap().to_string(),
                lib,
                setup: None,
                context: None,
            })
        }
    }

    pub fn get_setup(&self) -> &CloudSetup {
        self.setup
            .as_ref()
            .expect("Using setup data before the cloud setup")
    }

    /// Get call symbol from loaded plugin library.
    /// ```ignore
    /// let mut plugin = CloudPlugin::new(lib_path).unwrap();
    /// let plugin_setup = plugin.get_cloud_call::<CloudSetupFn>(b"cloud_setup");
    /// plugin_setup();
    /// ```
    pub unsafe fn get_cloud_call<T>(&self, name: &[u8]) -> Option<Symbol<T>> {
        // File resolve call is called many times and fill all the screen
        // with their logging
        let omit_debug = name == CloudFileResolveCall::SYMBOL;

        let name_string = String::from_utf8_lossy(name).to_owned();

        if !omit_debug {
            log_trace!([self.name] "Getting call: {name_string:?}");
        }

        match self.lib.get::<T>(name) {
            Ok(call) => Some(call),
            Err(err) => {
                log_trace!([self.name] "Can't get call {name_string:?}: {err:#?}");
                None
            }
        }
    }

    pub unsafe fn cloud_setup(&mut self) -> Option<()> {
        let lib_setup = get_cloud_call!(self, CloudSetupCall);
        let lib_setup = try_call!(lib_setup())?;
        log_info!([self.name] "Setup: {lib_setup:#?}");
        self.name = lib_setup.name.clone();
        self.setup = Some(lib_setup);
        Some(())
    }

    pub unsafe fn cloud_context(&mut self) {
        let lib_context = get_cloud_call!(self, CloudContextCall);
        let lib_context = try_call!(lib_context());
        self.context = lib_context;
    }

    pub unsafe fn cloud_debug_context(&mut self) {
        let context = match &self.context {
            Some(expr) => expr,
            None => {
                log_warn!([self.name] (FgYellow) "No context");
                return;
            }
        };
        let lib_context = get_cloud_call!(self, CloudDebugContextCall);
        try_call!(lib_context(*context));
    }

    pub unsafe fn cloud_file_resolve(&self, file: CloudFile) -> Option<CloudFileResolve> {
        let filename: PathBuf = file.relative_path.clone().into();
        let filename = filename.file_name().unwrap_or_default();
        let filename = filename.to_str().unwrap_or_default();
        if let Some(file_starts) = &self
            .setup
            .as_ref()
            .expect("Called before setup")
            .file_starts
        {
            if !filename.starts_with(file_starts) {
                return Some(CloudFileResolve::Ignore);
            }
        }
        if let Some(file_ends) = &self.setup.as_ref().expect("Called before setup").file_ends {
            if !filename.ends_with(file_ends) {
                return Some(CloudFileResolve::Ignore);
            }
        }

        let lib_file_resolve = get_cloud_call!(self, CloudFileResolveCall)?;
        let lib_file_resolve =
            lib_file_resolve(file, self.context.unwrap_or_else(CloudContextRaw::null));
        Some(lib_file_resolve)
    }

    pub unsafe fn cloud_optimized_tree_process(
        &self,
        leaf: OptimizedTreeLeaf,
        static_children: String,
        children: String,
        dynamic_child: String,
    ) -> Option<String> {
        let lib_file_resolve = get_cloud_call!(self, CloudOptimizedTreeProcessCall)?;
        let lib_file_resolve = lib_file_resolve(leaf, static_children, children, dynamic_child);
        Some(lib_file_resolve)
    }

    pub fn setup(&mut self) -> Option<()> {
        unsafe {
            self.cloud_setup()?;
            self.cloud_context();
        }
        Some(())
    }

    // pub fn file_resolve(&mut self, ctx: &CompileContext) -> Option<()> {
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

    pub fn resolve_optimized_tree(&self, ctx: &CompileContext) -> Option<OptimizedTreeContainer> {
        let setup = self.get_setup();
        if setup.file_strategy != CloudFilesStrategy::OptimizedTree {
            return None;
        }

        let input_paths = std::env::current_dir().unwrap().join("src");
        let input_paths = input_paths.join(setup.source_folder.clone());
        let (container, _) = optimized_tree_strategy(input_paths, self, ctx);

        // println!(
        //     "{:#?}",
        //     Fmt(|f| tree.read().unwrap().display(f, &container))
        // );

        Some(container)
    }
}
