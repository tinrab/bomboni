use std::path::PathBuf;

use crate::path_map::PathMap;

#[derive(Debug, Clone)]
pub struct CompileConfig {
    pub file_descriptor_set_path: PathBuf,
    pub output_path: PathBuf,
    pub format: bool,
    pub api: ApiConfig,
    pub external_paths: PathMap,
}

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub names: bool,
    pub field_names: bool,
    pub oneof_utility: bool,
    pub serde: bool,
    pub helpers_mod: Option<String>,
}

impl Default for CompileConfig {
    fn default() -> Self {
        Self {
            file_descriptor_set_path: PathBuf::from(std::env::var_os("OUT_DIR").unwrap())
                .join("fd.pb"),
            output_path: std::env::var_os("OUT_DIR").unwrap().into(),
            format: true,
            api: ApiConfig::default(),
            external_paths: PathMap::default(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            names: true,
            field_names: true,
            oneof_utility: true,
            serde: true,
            helpers_mod: None,
        }
    }
}
