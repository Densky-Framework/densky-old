/// Configure your cloud setup, here you can set your cloud's name, dependencies, and such more.
///
/// # Complete reference
/// ```ignore
/// cloud_setup!(CLOUD_NAME:path {
///     source_folder: SOURCE_FOLDER:expr, // required
///     file_starts: FILE_STARTS:expr,
///     file_ends: FILE_ENDS:expr,
///     file_strategy: FILE_STRATEGY:ident,
///     dependencies: [
///         DEPENDENCY:path =>(?) VERSION:expr,
///     ],
///     peer: CLOUD:path => VERSION:expr
/// });
/// ```
/// # Example usage
/// ```ignore
/// cloud_setup!(view::engines::react {
///     source_folder: "views",
///     file_ends: ".tsx",
///     dependencies: [
///         view::engine::common => "1.0.0",
///         tailwind::react =>? "0.2.0"
///     ],
///     peer: view::engine => "1.0.0"
/// });
/// ```
#[macro_export]
macro_rules! cloud_setup {
    (!add-to-list, $vec:ident, $dependency:path, $version:expr, $optional:expr, $($tail:tt)*) => {
        $vec.push($crate::CloudDependency {
            name: stringify!($dependency).to_string(),
            version: ($version).into(),
            optional: $optional
        });
        $crate::cloud_setup!(!list, $vec, $($tail)*);
    };
    (!list, $vec:ident, ) => {};
    (!list, $vec:ident, $dependency:path =>? $version:expr, $($tail:tt)*) => {
        $crate::cloud_setup!(!add-to-list, $vec, $dependency, $version, true, $($tail:tt)*);
    };
    (!list, $vec:ident, $dependency:path => $version:expr, $($tail:tt)*) => {
        $crate::cloud_setup!(!add-to-list, $vec, $dependency, $version, false, $($tail)*);
    };

    ($cloud_name:path {
        source_folder: $source_folder:expr ,
        $(file_starts: $file_starts:expr ,)?
        $(file_ends: $file_ends:expr ,)?
        $(file_strategy: $file_strategy:ident ,)?
        $(dependencies: [
            $($dependency:tt)*
        ])?
    }) => {
        static CLOUD_NAME: &'static str = stringify!($cloud_name);

        #[no_mangle]
        pub fn cloud_setup() -> $crate::CloudSetup {
            let version = env!("CARGO_PKG_VERSION");

            let file_starts: Option<String> = None;
            $( let file_starts = Some(($file_starts).into());)?

            let file_ends: Option<String> = None;
            $( let file_ends = Some(($file_ends).into());)?

            let file_strategy = $crate::CloudFilesStrategy::default();
            $( let file_strategy = $crate::CloudFilesStrategy::$file_strategy;)?

            let mut dependencies = Vec::new();
            $($crate::cloud_setup!(!list, dependencies, $($dependency)*))?;

            $crate::CloudSetup {
                name: CLOUD_NAME.into(),
                version: version.into(),
                source_folder: ($source_folder).into(),
                file_starts,
                file_ends,
                file_strategy,
                dependencies
            }
        }
    };
}

#[macro_export]
macro_rules! cloud_context {
    ($context:ident) => {
        #[no_mangle]
        pub fn cloud_context() -> $crate::context::CloudContextRaw {
            use $crate::context::CloudContext;
            $context::default().to_raw()
        }

        #[no_mangle]
        pub fn cloud_debug_context(context: $crate::context::CloudContextRaw) {
            let context = context.to_rusty::<$context>();
            $crate::log_debug!([CLOUD_NAME] "Debug context: {context:#?}");
        }
    };
}

pub use cloud_context;
pub use cloud_setup;
