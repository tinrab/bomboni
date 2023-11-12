use std::{error::Error, path::PathBuf};

use bomboni_prost::{
    compile,
    config::{ApiConfig, CompileConfig},
};

fn main() -> Result<(), Box<dyn Error + 'static>> {
    #[cfg(feature = "testing")]
    {
        let mut config = prost_build::Config::new();
        config
            .file_descriptor_set_path("./tests/proto/fd.pb")
            .out_dir("./tests/proto")
            .protoc_arg("--experimental_allow_proto3_optional")
            .btree_map(["."])
            .compile_protos(&["./tests/proto/tools.proto"], &["./tests/proto/"])?;

        compile(CompileConfig {
            file_descriptor_set_path: "./tests/proto/fd.pb".into(),
            output_path: "./tests/proto".into(),
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

    for proto_path in proto_paths.iter() {
        println!("cargo:rerun-if-changed={}", proto_path.display());
    }

    let mut config = prost_build::Config::new();
    config
        .out_dir("./src/proto")
        .file_descriptor_set_path("./src/proto/fd.pb")
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
    for type_path in get_copy_type_paths() {
        config.type_attribute(type_path, r#"#[derive(Copy)]"#);
    }

    config.compile_protos(&proto_paths, &["./proto"])?;
    compile(CompileConfig {
        file_descriptor_set_path: "./src/proto/fd.pb".into(),
        output_path: "./src/proto".into(),
        api: ApiConfig {
            domain: Some("type.googleapis.com".into()),
            ..Default::default()
        },
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
    .map(|type_name| format!(".google.rpc.{}", type_name))
}

fn get_copy_type_paths() -> impl Iterator<Item = String> {
    ["Timestamp", "Empty", "Duration"]
        .into_iter()
        .map(|type_name| format!(".google.protobuf.{}", type_name))
}
