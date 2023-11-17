use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CompileConfig {
    pub file_descriptor_set_path: PathBuf,
    pub output_path: PathBuf,
    pub format: bool,
    pub api: ApiConfig,
}

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub names: bool,
    pub field_names: bool,
    pub type_url: bool,
    pub oneof_utility: bool,
    pub domain: Option<String>,
    pub serde: bool,
}

impl Default for CompileConfig {
    fn default() -> Self {
        Self {
            file_descriptor_set_path: PathBuf::from(std::env::var_os("OUT_DIR").unwrap())
                .join("fd.pb"),
            output_path: std::env::var_os("OUT_DIR").unwrap().into(),
            format: true,
            api: ApiConfig::default(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            names: true,
            field_names: true,
            type_url: true,
            oneof_utility: true,
            domain: None,
            serde: true,
        }
    }
}
