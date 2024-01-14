use crate::config::CompileConfig;
use bomboni_core::string::{str_to_case, Case};
use prost_types::{DescriptorProto, FileDescriptorSet, OneofDescriptorProto};
use syn::{parse_quote, ExprPath, PathSegment};

pub struct Context<'a> {
    pub config: &'a CompileConfig,
    pub descriptor: &'a FileDescriptorSet,
    pub package_name: String,
    pub path: Vec<String>,
}

impl<'a> Context<'a> {
    pub fn get_type_expr_path(&self, name: &str) -> ExprPath {
        let mut path = String::new();
        for parent in &self.path {
            path.push_str(&str_to_case(parent, Case::Snake));
            path.push_str("::");
        }
        path.push_str(&str_to_case(name, Case::Pascal));
        syn::parse_str::<ExprPath>(&path).unwrap()
    }

    pub fn get_type_expr_relative_path(&self, name: &str, nesting: usize) -> ExprPath {
        let mut path = self.get_type_expr_path(name);
        let super_ident: PathSegment = parse_quote!(super);
        for _ in 0..(nesting + self.path.len()) {
            path.path.segments.insert(0, super_ident.clone());
        }
        path
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
                    .collect::<Vec<_>>()
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
                self.package_name
                    .split('.')
                    .map(|_| "super")
                    .collect::<Vec<_>>()
                    .join("::"),
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
