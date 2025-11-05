#![doc = include_str!("../README.md")]

use config::CompileConfig;
use context::Context;
use enums::write_enum;
use helpers::write_helpers;
use messages::write_message;
use prost::Message;
use prost_types::{FileDescriptorProto, FileDescriptorSet};
use quote::quote;
use std::{
    collections::BTreeMap,
    error::Error,
    fs::{File, OpenOptions},
    io::{Read, Write},
};

pub mod config;
pub use config::ApiConfig;
mod context;
mod enums;
mod helpers;
mod messages;
mod oneofs;
/// Path mapping utilities for external protobuf references.
pub mod path_map;

/// Compiles protobuf files using the provided configuration.
///
/// This function reads the file descriptor set, processes all protobuf files,
/// and generates Rust code with additional helper functions and utilities.
///
/// # Errors
///
/// Returns an error if:
/// - The file descriptor set cannot be read
/// - Protobuf compilation fails
/// - File I/O operations fail
/// - Code generation encounters invalid protobuf definitions
///
/// # Panics
///
/// Panics if a protobuf file does not have a package name defined.
///
/// # Examples
///
/// ```rust
/// use bomboni_prost::{compile, config::CompileConfig, ApiConfig, path_map::PathMap};
///
/// let config = CompileConfig {
///     file_descriptor_set_path: "descriptor.bin".into(),
///     output_path: "src/generated".into(),
///     format: true,
///     api: ApiConfig::default(),
///     external_paths: PathMap::default(),
/// };
///
/// // compile(config)?;  // This would compile protobuf files
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn compile(config: CompileConfig) -> Result<(), Box<dyn Error>> {
    let mut buf = Vec::new();
    File::open(&config.file_descriptor_set_path)
        .map_err(|err| {
            format!(
                "failed to open file descriptor set at `{}`: {}",
                config.file_descriptor_set_path.display(),
                err
            )
        })?
        .read_to_end(&mut buf)?;
    let descriptor = FileDescriptorSet::decode(buf.as_slice())?;

    let files_per_package = descriptor.file.iter().fold(
        BTreeMap::<String, Vec<&FileDescriptorProto>>::new(),
        |mut files, file| {
            let package_name = file.package.clone().unwrap();
            files.entry(package_name).or_default().push(file);
            files
        },
    );

    for (package_name, files) in files_per_package {
        let mut src = quote!();

        let context = Context {
            config: &config,
            package_name: package_name.clone(),
            path: Vec::default(),
        };

        for file in &files {
            for message in &file.message_type {
                write_message(&context, &mut src, message);
            }
            for enum_type in &file.enum_type {
                write_enum(&context, &mut src, enum_type);
            }
        }

        write_helpers(&context, &mut src, &files);

        // Write content to file
        let output_path = config.output_path.join(format!("./{package_name}.plus.rs"));
        println!(
            "writing package `{}` to file `{}`",
            package_name,
            output_path.display()
        );
        let mut output_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(output_path)
            .unwrap();
        if config.format {
            let file = syn::parse_file(&src.to_string())?;
            let formatted = prettyplease::unparse(&file);
            output_file.write_all(formatted.as_bytes())?;
        } else {
            output_file.write_all(src.to_string().as_bytes())?;
        }
    }

    Ok(())
}
