use std::{fmt::Debug, path::PathBuf};

use densky_adapter::{utils::join_paths, AHashMap, CloudDependency};
use densky_adapter::{CloudVersion, ErrorContext};
use jsonc_parser::{JsonObject, JsonValue};

use crate::utils::{discover_file, read_file};

pub struct CompileOptions {
    pub verbose: bool,
}

#[derive(Clone)]
pub struct ConfigFile {
    pub verbose: bool,
    pub output: PathBuf,
    pub vendor: Vec<PathBuf>,
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
    pub fn discover(cwd: &PathBuf) -> densky_adapter::Result<ConfigFile> {
        let (config_file_path, config_file) = discover_file(vec![
            join_paths("deno.jsonc", &cwd),
            join_paths("deno.json", &cwd),
            join_paths("densky.jsonc", &cwd),
            join_paths("densky.json", &cwd),
        ])?;
        let config_file = read_file(config_file, config_file_path.display())?;
        Self::parse(config_file, &cwd)
    }

    pub fn parse(contents: String, cwd: &PathBuf) -> Result<ConfigFile, densky_adapter::Error> {
        let doc = match jsonc_parser::parse_to_value(&contents, &Default::default())
            .with_context(|| format!("Can't parse json"))?
        {
            None => JsonObject::new(Default::default()),
            Some(jsonc_parser::JsonValue::Object(value)) => value,
            Some(_) => return Err(densky_adapter::anyhow!("config file should be an object",)),
        };

        let densky = &JsonObject::new(Default::default());
        let densky = doc.get_object("densky").unwrap_or(densky);
        let verbose = densky.get_boolean("verbose").unwrap_or_default();
        let output = densky
            .get_string("output")
            .unwrap_or(&std::borrow::Cow::Borrowed(".densky"));
        let output: PathBuf = join_paths(output.clone().into_owned(), cwd).into();

        let vendor = densky.get_array("vendor");
        let vendor = if let Some(vendor) = vendor {
            vendor
                .iter()
                .filter_map(|x| match x {
                    JsonValue::String(ref v) => {
                        Some(join_paths(v.clone().into_owned(), &cwd).into())
                    }
                    _ => None,
                })
                .collect::<Vec<PathBuf>>()
        } else {
            Vec::new()
        };

        let dependencies = if let Some(clouds) = densky.get_object("clouds") {
            let mut dependencies = AHashMap::new();

            for (cloud_name, cloud) in clouds.clone().into_iter() {
                match cloud {
                    JsonValue::String(v) => {
                        let version = v.into_owned();
                        let version = CloudVersion::from(version);

                        if let CloudVersion::Unknown(version) = version {
                            eprintln!("Invalid version '{version}'");
                            continue;
                        }
                        let dependency = CloudDependency {
                            name: cloud_name.clone(),
                            version,
                            optional: false,

                            options: Default::default(),
                        };

                        dependencies.insert(cloud_name.into(), dependency);
                    }
                    JsonValue::Object(ref cloud) => {
                        let version = cloud
                            .get_string("version")
                            .cloned()
                            .unwrap_or(std::borrow::Cow::Borrowed("*"))
                            .into_owned();
                        let version = CloudVersion::from(version);

                        if let CloudVersion::Unknown(version) = version {
                            eprintln!("Invalid version '{version}'");
                            continue;
                        }

                        // let options = if let Some(options) = cloud.as_table_like() {
                        //     let mut opts = AHashMap::new();

                        // for (name, opt) in options.iter() {
                        //     // Reserved options
                        //     if matches!(name, "version") {
                        //         continue;
                        //     }
                        //
                        //     let Some(opt) = opt.as_value() else {
                        //         eprint!("Invalid option type '{name}' in '{cloud_name}'");
                        //         continue;
                        //     };
                        //
                        //     let Some(opt) = parse_opt(opt) else {
                        //         eprint!("Invalid option type '{name}' in '{cloud_name}'");
                        //         continue;
                        //     };
                        //
                        //     opts.insert(name.to_string(), opt);
                        // }

                        //     opts
                        // } else {
                        let options = Default::default();
                        // };

                        let dependency = CloudDependency {
                            name: cloud_name.clone(),
                            version,
                            optional: false,

                            options,
                        };

                        dependencies.insert(cloud_name.into(), dependency);
                    }
                    _ => {
                        return Err(densky_adapter::anyhow!(
                            "Invalid cloud definition. Should be object or string version"
                        ))
                    }
                }
            }
            dependencies
        } else {
            AHashMap::new()
        };

        Ok(ConfigFile {
            verbose,
            output,
            vendor,
            dependencies,
        })
    }
}

// TODO: use jsonc_parser to parse CloudDependencyOption
//
// fn parse_opt(opt: &JsonValue<'_>) -> Option<CloudDependencyOption> {
//     match opt {
//         JsonValue::Number(ref v) => Some(CloudDependencyOption::Float(v.value().clone())),
//         JsonValue::String(ref v) => Some(CloudDependencyOption::String(v.value().clone())),
//         JsonValue::Object(ref v) => Some(CloudDependencyOption::String(v.value().clone())),
//         JsonValue::Boolean(ref v) => Some(CloudDependencyOption::Boolean(v.value().clone())),
//         JsonValue::Array(ref v) => Some(CloudDependencyOption::Array(
//             v.iter().map_while(|opt| parse_opt(opt)).collect(),
//         )),
//         _ => None,
//     }
// }
