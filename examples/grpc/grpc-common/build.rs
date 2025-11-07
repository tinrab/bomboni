use std::{error::Error, path::PathBuf};

use bomboni_prost::{
    compile,
    config::{ApiConfig, CompileConfig},
};
use prost_build::Config;

fn main() -> Result<(), Box<dyn Error + 'static>> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let fd_path = out_dir.join("common.fd");

    let root_path = PathBuf::from("./proto");
    let proto_paths: Vec<_> = ["errors/error.proto"]
        .into_iter()
        .map(|proto_path| root_path.join(proto_path))
        .collect();

    for proto_path in &proto_paths {
        println!("cargo:rerun-if-changed={}", proto_path.display());
    }

    let mut config = Config::default();
    config
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(&fd_path)
        .enable_type_names()
        .type_name_domain(&[".common"], "common.rabzelj.com")
        .btree_map(["."]);

    tonic_prost_build::configure()
        .build_server(false)
        .build_client(false)
        .compile_with_config(config, &proto_paths, &["./proto".into()])?;

    compile(CompileConfig {
        api: ApiConfig {
            helpers_mod: Some("helpers".into()),
            ..Default::default()
        },
        file_descriptor_set_path: fd_path,
        ..Default::default()
    })?;

    Ok(())
}
