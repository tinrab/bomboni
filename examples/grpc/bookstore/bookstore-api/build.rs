//! Build script for bookstore-api crate.

use std::path::PathBuf;

use bomboni_prost::{
    compile,
    config::{ApiConfig, CompileConfig},
};
use prost_build::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let fd_path = out_dir.join("bookstore_v1.fd");

    let root_path = PathBuf::from("./proto/v1");
    let proto_paths: Vec<_> = [
        "book_resources.proto",
        "book_service.proto",
        "author_resources.proto",
        "author_service.proto",
        "errors/book_error.proto",
        "errors/author_error.proto",
        "errors/bookstore_error.proto",
    ]
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
        .type_name_domain([".bookstore"], "bookstore.rabzelj.com")
        .extern_path(
            ".google.protobuf.Timestamp",
            "::bomboni::proto::google::protobuf::Timestamp",
        )
        .extern_path(
            ".google.protobuf.FieldMask",
            "::bomboni::proto::google::protobuf::FieldMask",
        )
        .extern_path(".common", "::grpc_common::proto")
        .btree_map(["."]);

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .client_mod_attribute("bookstore.v1", r#"#[cfg(feature = "client")]"#)
        .server_mod_attribute("bookstore.v1", r#"#[cfg(feature = "server")]"#)
        .compile_with_config(
            config,
            &proto_paths,
            &[
                "./proto".into(),
                "../../grpc-common/proto".into(),
                "../../../../bomboni_proto/proto".into(),
            ],
        )?;

    compile(CompileConfig {
        api: ApiConfig {
            helpers_mod: Some("helpers".into()),
            ..Default::default()
        },
        file_descriptor_set_path: fd_path,
        external_paths: [
            (".google", "::bomboni::proto::google"),
            (".common", "::grpc_common::proto"),
        ]
        .into(),
        ..Default::default()
    })?;

    Ok(())
}
