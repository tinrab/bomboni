//! Build script for bookstore-api crate.

use prost_build::Config;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let fd_path = out_dir.join("bookstore_v1.fd");

    let root_path = PathBuf::from("./proto/v1");
    let proto_paths: Vec<_> = [
        "book_resources.proto",
        "book_service.proto",
        "author_resources.proto",
        "errors/book_error.proto",
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
        // .type_name_domain(&[".bookstore"], "bookstore.com")
        .extern_path(
            ".google.protobuf.Timestamp",
            "::bomboni_proto::google::protobuf::Timestamp",
        )
        .extern_path(
            ".google.protobuf.FieldMask",
            "::bomboni_proto::google::protobuf::FieldMask",
        )
        .btree_map(["."]);

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .client_mod_attribute("bookstore.v1", r#"#[cfg(feature = "client")]"#)
        .server_mod_attribute("bookstore.v1", r#"#[cfg(feature = "server")]"#)
        .compile_with_config(config, &proto_paths, &["./proto".into()])?;

    Ok(())
}
