//! Build script for compiling protobuf files for the gRPC common library.
//!
//! This build script compiles common protobuf definitions and generates
//! Rust code with proper serde serialization support.

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

    let root_path = PathBuf::from("./proto/common");
    let proto_paths: Vec<_> = ["access_token.proto", "error.proto"]
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
        .type_name_domain([".common"], "common.rabzelj.com")
        .extern_path(
            ".google.protobuf.Timestamp",
            "::bomboni_proto::google::protobuf::Timestamp",
        )
        .extern_path(
            ".google.protobuf.FieldMask",
            "::bomboni_proto::google::protobuf::FieldMask",
        )
        .btree_map(["."]);

    configure_serde(&mut config);

    tonic_prost_build::configure()
        .build_server(false)
        .build_client(false)
        .compile_with_config(
            config,
            &proto_paths,
            &["./proto".into(), "../../../bomboni_proto/proto".into()],
        )?;

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

fn configure_serde(config: &mut Config) {
    for type_name in [
        "AccessToken",
        "AccessTokenData",
        "AccessTokenIdentity",
        "AccessTokenIdentity.kind",
        "EmailIdentity",
        "AccessTokenAccount",
    ] {
        config.type_attribute(
            format!("common.{type_name}"),
            r#"
            #[derive(::serde::Serialize, ::serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
        "#,
        );
    }

    for field_name in [
        "AccessToken.expiration",
        "AccessToken.issued_at",
        "AccessToken.data",
    ] {
        config.field_attribute(
            format!("common.{field_name}"),
            r#"#[serde(default, skip_serializing_if = "Option::is_none")]"#,
        );
    }

    config.field_attribute(
        "common.AccessToken.expiration",
        r#"#[serde(rename = "exp", with = "serde_helpers::timestamp_as_seconds::option")]"#,
    );
    config.field_attribute(
        "common.AccessToken.issued_at",
        r#"#[serde(rename = "iat", with = "serde_helpers::timestamp_as_seconds::option")]"#,
    );
    config.field_attribute("common.AccessToken.issuer", r#"#[serde(rename = "iss")]"#);
    config.field_attribute("common.AccessToken.subject", r#"#[serde(rename = "sub")]"#);

    config.field_attribute("common.AccessTokenIdentity.kind", r"#[serde(flatten)]");
    config.type_attribute(
        "common.AccessTokenIdentity.kind",
        "#[serde(rename = \"AccessTokenIdentityKind\")]",
    );

    config.type_attribute("common.AccessToken", "#[derive(Eq)]");
    config.type_attribute("common.AccessTokenData", "#[derive(Eq)]");

    config.type_attribute(
        "common.CommonErrorReason",
        "#[allow(clippy::missing_const_for_fn)]",
    );
    config.type_attribute(
        "common.AccessTokenIdentity",
        "#[allow(clippy::missing_const_for_fn)]",
    );
}
