mod file_process;
mod setup;

pub use self::file_process::*;
pub use self::setup::*;

use crate::context;

macro_rules! create_call {
    ($call_name:ident, $symbol:expr, $($fn:tt)+) => {
        #[allow(non_snake_case)]
         pub mod $call_name {
            #[allow(unused)]
            use super::*;
            pub static SYMBOL: &'static [u8] = $symbol;
            pub type Fn = unsafe $($fn)+;
         }
    };
}

// Cloud Setup
create_call!(
    CloudSetupCall,
    b"cloud_setup",
    fn() -> anyhow::Result<setup::CloudSetup>
);
create_call!(
    CloudContextCall,
    b"cloud_context",
    fn() -> anyhow::Result<context::CloudContextRaw>
);
create_call!(
    CloudDebugContextCall,
    b"cloud_debug_context",
    fn(context::CloudContextRaw) -> ()
);
create_call!(
    CloudPostSetupCall,
    b"cloud_post_setup",
    fn(context::CloudContextRaw) -> ()
);

// File Processing
create_call!(
    CloudFileResolveCall,
    b"cloud_file_resolve",
    fn(
        file_process::CloudFile,
        context::CloudContextRaw,
    ) -> anyhow::Result<file_process::CloudFileResolve>
);
create_call!(
    CloudBeforeManifestCall,
    b"cloud_before_manifest",
    fn() -> anyhow::Result<file_process::CloudManifestUpdate>
);
create_call!(
    CloudOptimizedManifestCall,
    b"cloud_manifest",
    fn(
        OptimizedTreeLeaf,
        String,
        String,
        String,
    ) -> anyhow::Result<file_process::CloudManifestUpdate>
);
