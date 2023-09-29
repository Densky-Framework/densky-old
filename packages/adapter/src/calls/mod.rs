mod file_process;
mod setup;

pub use self::file_process::{CloudFile, CloudFileResolve};
pub use self::setup::{CloudDependency, CloudFilesStrategy, CloudSetup};

use crate::context::CloudContextRaw;

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
create_call!(CloudSetupCall, b"cloud_setup", fn() -> CloudSetup);
create_call!(CloudContextCall, b"cloud_context", fn() -> CloudContextRaw);
create_call!(
    CloudDebugContextCall,
    b"cloud_debug_context",
    fn(CloudContextRaw) -> ()
);
create_call!(
    CloudPostSetupCall,
    b"cloud_post_setup",
    fn(CloudContextRaw) -> ()
);

// File Processing
create_call!(
    CloudFileResolveCall,
    b"cloud_file_resolve",
    fn(CloudFile, CloudContextRaw) -> CloudFileResolve
);
create_call!(
    CloudFileProcessCall,
    b"cloud_file_process",
    fn(CloudFile, CloudContextRaw) -> ()
);
