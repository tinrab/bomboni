use crate::config::CompileConfig;
use crate::utility::str_to_case;
use convert_case::Case;
use itertools::Itertools;
use prost_types::{DescriptorProto, FileDescriptorSet, OneofDescriptorProto};
use syn::ExprPath;

pub struct Context<'a> {
    pub config: &'a CompileConfig,
    pub descriptor: &'a FileDescriptorSet,
    pub package_name: String,
    pub path: Vec<String>,
}

impl<'a> Context<'a> {
    pub fn get_type_ident(&self, name: &str) -> ExprPath {
        let mut ident = String::new();
        for parent in &self.path {
            ident.push_str(&str_to_case(parent, Case::Snake));
            ident.push_str("::");
        }
        ident.push_str(&str_to_case(name, Case::Pascal));
        syn::parse_str::<ExprPath>(&ident).unwrap()
    }

    pub fn get_proto_type_name(&self, name: &str) -> String {
        if self.path.is_empty() {
            return name.to_string();
        }
        format!("{}.{}", self.path.join("."), name)
    }

    pub fn get_proto_full_type_name(&self, name: &str) -> String {
        let type_name = self.get_proto_type_name(name);
        if self.package_name.is_empty() {
            return type_name;
        }
        format!("{}.{}", self.package_name, type_name)
    }

    pub fn get_ident_from_type_name_reference(&self, type_name_reference: &str) -> ExprPath {
        let type_path = if let Some((matcher, external_path)) =
            self.config.external_paths.get_first(type_name_reference)
        {
            format!(
                "{}::{}",
                external_path,
                type_name_reference
                    .trim_start_matches(matcher)
                    .trim_matches('.')
                    .split('.')
                    .join("::")
            )
        } else {
            let mut type_name_reference = type_name_reference
                .trim_start_matches('.')
                .split('.')
                .peekable();
            let mut type_path = String::new();
            while let Some(part) = type_name_reference.next() {
                type_path.push_str("::");
                if type_name_reference.peek().is_none() {
                    type_path.push_str(&str_to_case(part, Case::Pascal));
                    break;
                }
                type_path.push_str(&str_to_case(part, Case::Snake));
            }
            format!(
                "{}{}",
                self.package_name.split('.').map(|_| "super").join("::"),
                type_path
            )
        };

        syn::parse_str::<ExprPath>(&type_path).unwrap()
    }

    pub fn get_oneof_ident(
        &self,
        message: &DescriptorProto,
        oneof: &OneofDescriptorProto,
    ) -> ExprPath {
        let mut ident = String::new();
        for parent in &self.path {
            ident.push_str(&str_to_case(parent, Case::Snake));
            ident.push_str("::");
        }
        ident.push_str(&format!(
            "{}::{}",
            str_to_case(message.name(), Case::Snake),
            str_to_case(oneof.name(), Case::Pascal)
        ));
        syn::parse_str::<ExprPath>(&ident).unwrap()
    }
}
