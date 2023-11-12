use convert_case::{Case, Casing};
use prost_types::{DescriptorProto, FileDescriptorSet, OneofDescriptorProto};
use syn::ExprPath;

use crate::config::ApiConfig;

pub struct Context<'a> {
    pub config: &'a ApiConfig,
    pub descriptor: &'a FileDescriptorSet,
    pub package_name: String,
    pub path: Vec<String>,
}

impl<'a> Context<'a> {
    pub fn get_type_ident(&self, name: &str) -> ExprPath {
        let mut ident = String::new();
        for parent in self.path.iter() {
            ident.push_str(&parent.to_case(Case::Snake));
            ident.push_str("::");
        }
        ident.push_str(&name.to_case(Case::Pascal));
        syn::parse_str::<ExprPath>(&ident).unwrap()
    }

    pub fn get_proto_type_name(&self, name: &str) -> String {
        if self.path.is_empty() {
            return name.to_string();
        }
        format!("{}.{}", self.path.join("."), name)
    }

    pub fn get_oneof_ident(
        &self,
        message: &DescriptorProto,
        oneof: &OneofDescriptorProto,
    ) -> ExprPath {
        let mut ident = String::new();
        for parent in self.path.iter() {
            ident.push_str(&parent.to_case(Case::Snake));
            ident.push_str("::");
        }
        ident.push_str(&format!(
            "{}::{}",
            message.name().to_case(Case::Snake),
            oneof.name().to_case(Case::Pascal)
        ));
        syn::parse_str::<ExprPath>(&ident).unwrap()
    }
}
