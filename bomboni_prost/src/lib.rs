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
use quote::quote;
mod context;
mod enums;
mod messages;
mod oneofs;
mod utility;

pub fn compile(config: CompileConfig) -> Result<(), Box<dyn Error>> {
    let mut buf = Vec::new();
    File::open(&config.file_descriptor_set_path)?.read_to_end(&mut buf)?;
    let descriptor = FileDescriptorSet::decode(buf.as_slice())?;

    let flush = |package: &str, content: TokenStream| {
        let output_path = config.output_path.join(format!("{}.plus.rs", package));
        println!(
            "writing package `{}` to file `{}`",
            package,
            output_path.display()
        );
        // let mut output_file = File::create(output_path)?;
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
        Result::<(), Box<dyn Error>>::Ok(())
    };

    // Clear files
    for file in descriptor.file.iter() {
        let package_name = file.package.clone().unwrap();
        let output_path = config.output_path.join(format!("{}.plus.rs", package_name));
        let mut output_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(output_path)
            .unwrap();
        output_file.write_all("".as_bytes())?;
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
            config: &config.api,
            descriptor: &descriptor,
            package_name,
            path: Default::default(),
        };

        for message in file.message_type.iter() {
            write_message(&context, &mut current_content, message);
        }
        for enum_type in file.enum_type.iter() {
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
