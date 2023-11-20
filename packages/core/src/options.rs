use std::{fmt::Debug, path::PathBuf};

use ahash::AHashMap;
use densky_adapter::{utils::join_paths, CloudDependency};
use toml_edit::{Document, TomlError};

pub struct CompileOptions {
    pub verbose: bool,
}

pub struct CompileContext {
    pub output_dir: String,
    pub cwd: String,
    pub verbose: bool,
}

#[derive(Clone)]
pub struct ConfigFile {
    pub doc: Document,
    pub verbose: bool,
    pub output: PathBuf,
    pub dependencies: AHashMap<String, CloudDependency>,
}

impl Debug for ConfigFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigFile")
            .field("verbose", &self.verbose)
            .field("output_dir", &self.output.display())
            .field("dependencies", &self.dependencies)
            .finish()
    }
}

impl ConfigFile {
    pub fn parse(toml_file: String, cwd: &PathBuf) -> Result<ConfigFile, TomlError> {
        let doc = toml_file.parse::<Document>()?;

        let densky = &doc["densky"];
        let verbose = densky["verbose"].as_bool().unwrap_or_default();
        let output = densky["verbose"].as_str().unwrap_or(".densky");
        let output: PathBuf = join_paths(output, cwd).into();

        let clouds = doc["cloud"].as_table();
        let dependencies = match clouds {
            None => AHashMap::new(),
            Some(clouds) => {
                let mut dependencies = AHashMap::new();

                for (cloud_name, cloud) in clouds.iter() {
                    let dependency = CloudDependency {
                        name: cloud_name.to_string(),
                        version: cloud["version"].as_str().unwrap_or("*").to_string(),
                        optional: false,
                    };

                    dependencies.insert(cloud_name.into(), dependency);
                }

                dependencies
            }
        };

        Ok(ConfigFile {
            doc,
            verbose,
            output,
            dependencies,
        })
    }
}
