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
    pub domain: Option<String>,
    pub enable_field_names: bool,
}

impl Default for CompileConfig {
    fn default() -> Self {
        Self {
            file_descriptor_set_path: PathBuf::from(std::env::var("OUT_DIR").unwrap())
                .join("fd.pb"),
            output_path: std::env::var("OUT_DIR").unwrap().into(),
            format: true,
            api: Default::default(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            domain: None,
            enable_field_names: true,
        }
    }
}
