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

#[derive(Debug)]
pub struct CloudDependency {
    pub name: String,
    pub version: String,
    pub optional: bool,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(u8)]
pub enum CloudFilesStrategy {
    #[default]
    None = 0,
    SimpleTree,
    OptimizedTree,
}
