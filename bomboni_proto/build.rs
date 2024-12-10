use bomboni_prost::{
    compile,
    config::{ApiConfig, CompileConfig},
};
use prost_build::Config;
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let fd_path = out_dir.join("fd.pb");

    #[cfg(feature = "testing")]
    {
        let fd_path = out_dir.join("test.pb");

        let proto_paths = ["./tests/proto/tools.proto"];
        for proto_path in &proto_paths {
            println!("cargo:rerun-if-changed={proto_path}");
        }

        let mut config = Config::new();
        config
            .file_descriptor_set_path(&fd_path)
            .compile_well_known_types()
            .extern_path(".google", "::bomboni_proto::google")
            .protoc_arg("--experimental_allow_proto3_optional")
            .btree_map(["."])
            .enable_type_names()
            .type_name_domain(["."], "tests")
            .compile_protos(&proto_paths, &["./proto", "./tests/proto/"])?;

        compile(CompileConfig {
            api: ApiConfig {
                helpers_mod: Some("helpers".into()),
                ..Default::default()
            },
            file_descriptor_set_path: out_dir.join(fd_path),
            external_paths: [(".google", "::bomboni_proto::google")].into(),
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
        "google/protobuf/struct.proto",
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

    let mut config = Config::new();
    config
        .file_descriptor_set_path(&fd_path)
        .compile_well_known_types()
        .protoc_arg("--experimental_allow_proto3_optional")
        .enable_type_names()
        .type_name_domain(["."], "type.googleapis.com")
        .btree_map(["."]);

    build_serde(&mut config);
    if std::env::var("CARGO_CFG_TARGET_FAMILY") == Ok("wasm".into()) && cfg!(feature = "wasm") {
        build_wasm(&mut config);
    }

    config.compile_protos(&proto_paths, &["./proto"])?;

    compile(CompileConfig {
        api: ApiConfig {
            helpers_mod: Some("helpers".into()),
            ..Default::default()
        },
        file_descriptor_set_path: out_dir.join(fd_path),
        ..Default::default()
    })?;

    Ok(())
}

fn build_serde(config: &mut Config) {
    // Camel cased
    for message_name in [
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
    ] {
        config.message_attribute(
            format!(".google.rpc.{message_name}"),
            r#"
                #[derive(::serde::Serialize, ::serde::Deserialize)]
                #[serde(rename_all = "camelCase")]
            "#,
        );
    }

    // Skip defaults
    config.field_attribute(
        ".google.rpc.ErrorInfo.metadata",
        r#"#[serde(default, skip_serializing_if = "crate::serde::helpers::is_default")]"#,
    );

    config.message_attribute(
        ".google.rpc.Status",
        r"#[derive(::serde::Serialize, ::serde::Deserialize)]",
    );
    config.field_attribute(
        ".google.rpc.Status.code",
        r#"#[serde(with = "crate::google::rpc::helpers::code_serde")]"#,
    );
    config.field_attribute(
        ".google.rpc.Status.details",
        r#"#[serde(with = "crate::rpc::status::details_serde")]"#,
    );
}

fn build_wasm(config: &mut Config) {
    let error_details = [
        "Status",
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
    ];
    for type_name in &error_details {
        config.message_attribute(
            format!(".google.rpc.{type_name}"),
            r"
                #[derive(bomboni_wasm::Wasm)]
                #[wasm(bomboni_wasm_crate = bomboni_wasm, wasm_abi)]
            ",
        );
    }
    config.field_attribute(
        ".google.rpc.Status.code",
        "#[wasm(override_type = \"string\")]",
    );
    config.field_attribute(
        ".google.rpc.Status.details",
        format!(
            "#[wasm(override_type = \"(\n{})[]\")]",
            error_details
                .iter()
                .skip(1)
                .map(|type_name| format!(
                    "  ( {{'@type': 'type.googleapis.com/google.rpc.{type_name}';}} & ({type_name}) )\n"
                ))
                .collect::<Vec<_>>()
                .join(" | ")
        ),
    );

    config.message_attribute(
        ".google.protobuf.Duration",
        r#"
            #[derive(bomboni_wasm::Wasm)]
            #[wasm(
                bomboni_wasm_crate = bomboni_wasm,
                wasm_abi,
                js_value { convert_string },
                override_type = "`${number}.${number}s` | `${number}s`",
            )]
        "#,
    );
    config.message_attribute(
        ".google.protobuf.Struct",
        r#"
            #[derive(bomboni_wasm::Wasm)]
            #[wasm(
                bomboni_wasm_crate = bomboni_wasm,
                wasm_abi,
                js_value,
                rename = "JsonObject",
                override_type = "{[key: string]: JsonValue}",
            )]
        "#,
    );
    config.message_attribute(
        ".google.protobuf.Value",
        r#"
            #[derive(bomboni_wasm::Wasm)]
            #[wasm(
                bomboni_wasm_crate = bomboni_wasm,
                wasm_abi,
                js_value,
                rename = "JsonValue",
                override_type = "string | number | boolean | null | JsonObject | Array<JsonValue>",
            )]
        "#,
    );

    if cfg!(feature = "js") {
        config.message_attribute(
            ".google.protobuf.Empty",
            r#"
                #[derive(bomboni_wasm::Wasm)]
                #[wasm(
                    bomboni_wasm_crate = bomboni_wasm,
                    wasm_abi,
                    js_value,
                    override_type = "undefined | null",
                )]
            "#,
        );
        config.message_attribute(
            ".google.protobuf.Timestamp",
            r#"
                #[derive(bomboni_wasm::Wasm)]
                #[wasm(
                    bomboni_wasm_crate = bomboni_wasm,
                    wasm_abi,
                    js_value,
                    override_type = "Date",
                )]
            "#,
        );
    } else {
        config.message_attribute(
            ".google.protobuf.Empty",
            r#"
                #[derive(bomboni_wasm::Wasm)]
                #[wasm(
                    bomboni_wasm_crate = bomboni_wasm,
                    wasm_abi,
                    js_value,
                    override_type = "null",
                )]
            "#,
        );
        config.message_attribute(
            ".google.protobuf.Timestamp",
            "
                #[derive(bomboni_wasm::Wasm)]
                #[wasm(
                    bomboni_wasm_crate = bomboni_wasm,
                    wasm_abi,
                    js_value { convert_string },
                )]
            ",
        );
    }
}
