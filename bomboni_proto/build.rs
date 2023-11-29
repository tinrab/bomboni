use std::{error::Error, path::PathBuf};

use bomboni_prost::{
    compile,
    config::{ApiConfig, CompileConfig},
};

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let fd_path = out_dir.join("fd.pb");

    #[cfg(any(feature = "testing", debug_assertions))]
    {
        let fd_path = out_dir.join("test.pb");
        let mut config = prost_build::Config::new();
        config
            .file_descriptor_set_path(&fd_path)
            .protoc_arg("--experimental_allow_proto3_optional")
            .btree_map(["."])
            .compile_protos(&["./tests/proto/tools.proto"], &["./tests/proto/"])?;

        compile(CompileConfig {
            api: ApiConfig {
                domain: Some("tests".into()),
                ..Default::default()
            },
            file_descriptor_set_path: out_dir.join(fd_path),
            ..Default::default()
        })?;
    }

    let root_path = PathBuf::from("./proto");
    let proto_paths: Vec<_> = [
        "google/protobuf/timestamp.proto",
        "google/protobuf/wrappers.proto",
        "google/protobuf/any.proto",
        "google/protobuf/field_mask.proto",
        "google/protobuf/empty.proto",
        "google/rpc/error_details.proto",
        "google/rpc/code.proto",
        "google/rpc/status.proto",
    ]
    .into_iter()
    .map(|proto_path| root_path.join(proto_path))
    .collect();

    for proto_path in &proto_paths {
        println!("cargo:rerun-if-changed={}", proto_path.display());
    }

    let mut config = prost_build::Config::new();
    config
        .file_descriptor_set_path(&fd_path)
        .compile_well_known_types()
        .protoc_arg("--experimental_allow_proto3_optional")
        .btree_map(["."]);

    for type_path in get_camel_cased_type_paths() {
        config.type_attribute(
            type_path,
            r#"
                #[derive(::serde::Serialize, ::serde::Deserialize)]
                #[serde(rename_all = "camelCase")]
            "#,
        );
    }
    for type_path in get_default_type_paths() {
        config.field_attribute(
            type_path,
            r#"#[serde(default, skip_serializing_if = "crate::serde::helpers::is_default")]"#,
        );
    }
    for type_path in get_copy_type_paths() {
        config.type_attribute(type_path, r"#[derive(Copy)]");
    }
    config.type_attribute(
        ".google.rpc.Status",
        r"#[derive(::serde::Serialize, ::serde::Deserialize)]",
    );
    config.field_attribute(
        ".google.rpc.Status.details",
        r#"#[serde(with = "crate::google::rpc::status::details_serde")]"#,
    );
    config.field_attribute(
        ".google.rpc.Status.code",
        r#"#[serde(with = "crate::google::rpc::code_serde")]"#,
    );

    config.compile_protos(&proto_paths, &["./proto"])?;

    compile(CompileConfig {
        api: ApiConfig {
            domain: Some("type.googleapis.com".into()),
            ..Default::default()
        },
        file_descriptor_set_path: out_dir.join(fd_path),
        ..Default::default()
    })?;

    Ok(())
}

fn get_camel_cased_type_paths() -> impl Iterator<Item = String> {
    [
        "RetryInfo",
        "DebugInfo",
        "QuotaFailure",
        "ErrorInfo",
        "PreconditionFailure",
        "BadRequest",
        "RequestInfo",
        "ResourceInfo",
        "Help",
        "LocalizedMessage",
    ]
    .into_iter()
    .map(|type_name| format!(".google.rpc.{type_name}"))
}

fn get_default_type_paths() -> impl Iterator<Item = String> {
    std::iter::once("ErrorInfo.metadata").map(|type_name| format!(".google.rpc.{type_name}"))
}

fn get_copy_type_paths() -> impl Iterator<Item = String> {
    ["Timestamp", "Empty", "Duration"]
        .into_iter()
        .map(|type_name| format!(".google.protobuf.{type_name}"))
}
