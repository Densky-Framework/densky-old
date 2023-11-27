use std::{fmt::Debug, path::PathBuf};

use densky_adapter::{utils::join_paths, AHashMap, CloudDependency, CloudDependencyOption};
use toml_edit::{Document, Item, TomlError, Value};

pub struct CompileOptions {
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

        let dependencies = if let Some(clouds) = doc["cloud"].as_table() {
            let mut dependencies = AHashMap::new();

            for (cloud_name, cloud) in clouds.iter() {
                let name = cloud_name.to_string();
                let version: String = cloud["version"].as_str().unwrap_or("*").into();

                let options = if let Some(options) = cloud.as_table_like() {
                    let mut opts = AHashMap::new();

                    for (name, opt) in options.iter() {
                        // Reserved options
                        if matches!(name, "version") {
                            continue;
                        }

                        let Some(opt) = opt.as_value() else {
                            eprint!("Invalid option type '{name}' in '{cloud_name}'");
                            continue;
                        };

                        let Some(opt) = parse_opt(opt) else {
                            eprint!("Invalid option type '{name}' in '{cloud_name}'");
                            continue;
                        };

                        opts.insert(name.to_string(), opt);
                    }

                    opts
                } else {
                    AHashMap::new()
                };

                let dependency = CloudDependency {
                    name,
                    version,
                    optional: false,

                    options,
                };

                dependencies.insert(cloud_name.into(), dependency);
            }

            dependencies
        } else {
            AHashMap::new()
        };

        Ok(ConfigFile {
            doc,
            verbose,
            output,
            dependencies,
        })
    }
}

fn parse_opt(opt: &Value) -> Option<CloudDependencyOption> {
    match opt {
        Value::Float(ref v) => Some(CloudDependencyOption::Float(v.value().clone())),
        Value::Integer(ref v) => Some(CloudDependencyOption::Integer(v.value().clone())),
        Value::String(ref v) => Some(CloudDependencyOption::String(v.value().clone())),
        Value::Boolean(ref v) => Some(CloudDependencyOption::Boolean(v.value().clone())),
        Value::Array(ref v) => Some(CloudDependencyOption::Array(
            v.iter().map_while(|opt| parse_opt(opt)).collect(),
        )),
        _ => None,
    }
}
