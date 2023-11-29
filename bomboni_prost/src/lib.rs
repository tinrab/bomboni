use config::CompileConfig;
use context::Context;
use enums::write_enum;
use itertools::Itertools;
use messages::write_message;
use proc_macro2::TokenStream;
use prost::Message;
use prost_types::FileDescriptorSet;
use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{Read, Write},
};
pub mod config;
pub mod path_map;
use quote::quote;
mod context;
mod enums;
mod messages;
mod oneofs;
mod utility;

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

    let flush = |package: &str, content: TokenStream| -> Result<(), Box<dyn Error>> {
        let output_path = config.output_path.join(format!("./{package}.plus.rs"));
        println!(
            "writing package `{}` to file `{}`",
            package,
            output_path.display()
        );
        let mut output_file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(output_path)
            .unwrap();
        if config.format {
            let file = syn::parse_file(&content.to_string())?;
            let formatted = prettyplease::unparse(&file);
            output_file.write_all(formatted.as_bytes())?;
        } else {
            output_file.write_all(content.to_string().as_bytes())?;
        }
        Ok(())
    };

    // Clear files
    for file in &descriptor.file {
        let package_name = file.package.clone().unwrap();
        let output_path = config.output_path.join(format!("{package_name}.plus.rs"));
        let mut output_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(output_path)
            .unwrap();
        output_file.write_all(b"")?;
    }

    let mut current_package: Option<String> = None;
    let mut current_content = quote!();
    for file in descriptor
        .file
        .iter()
        .sorted_by(|a, b| a.package.cmp(&b.package))
    {
        let package_name = file.package.clone().unwrap();
        if let Some(stale_package) = current_package.clone() {
            if package_name != stale_package {
                flush(&stale_package, current_content)?;
                current_package = Some(package_name.clone());
                current_content = quote!();
            }
        } else {
            current_package = Some(package_name.clone());
            current_content = quote!();
        }

        let context = Context {
            config: &config,
            descriptor: &descriptor,
            package_name,
            path: Vec::default(),
        };

        for message in &file.message_type {
            write_message(&context, &mut current_content, message);
        }
        for enum_type in &file.enum_type {
            write_enum(&context, &mut current_content, enum_type);
        }
    }

    // Handle leftover content
    if let Some(package_name) = current_package {
        if !current_content.is_empty() {
            flush(&package_name, current_content)?;
        }
    }

    Ok(())
}
