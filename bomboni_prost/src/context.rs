//! Context for protobuf code generation.

use bomboni_core::string::{Case, str_to_case};
use prost_types::{DescriptorProto, OneofDescriptorProto};
use syn::{ExprPath, PathSegment, parse_quote};

use crate::config::CompileConfig;

/// Context for generating Rust code from protobuf definitions.
///
/// This struct maintains the current state during code generation,
/// including the configuration, package name, and nesting path.
/// It provides utility methods for generating type expressions and paths.
pub struct Context<'a> {
    /// The compilation configuration.
    pub config: &'a CompileConfig,

    /// The current protobuf package name.
    pub package_name: String,

    /// The current nesting path for types.
    pub path: Vec<String>,
}

impl Context<'_> {
    /// Gets the expression path for a type name.
    ///
    /// Constructs a fully-qualified Rust path for the given type name,
    /// taking into account the current nesting path and applying appropriate
    /// case conversions.
    pub(crate) fn get_type_expr_path(&self, name: &str) -> ExprPath {
        let mut path = String::new();
        for parent in &self.path {
            path.push_str(&str_to_case(parent, Case::Snake));
            path.push_str("::");
        }
        path.push_str(&str_to_case(name, Case::Pascal));
        syn::parse_str::<ExprPath>(&path).unwrap()
    }

    /// Gets the expression path for a type name with relative nesting.
    ///
    /// Similar to `get_type_expr_path`, but allows specifying the nesting level
    /// to generate relative paths. This is useful for nested types.
    ///
    /// # Arguments
    ///
    /// * `name` - The type name to generate a path for
    /// * `nesting` - The nesting level to use for the path
    ///
    /// # Returns
    ///
    /// Returns a `syn::ExprPath` representing the relative type path.
    pub(crate) fn get_type_expr_relative_path(&self, name: &str, nesting: usize) -> ExprPath {
        let mut path = self.get_type_expr_path(name);
        let super_ident: PathSegment = parse_quote!(super);
        for _ in 0..(nesting + self.path.len()) {
            path.path.segments.insert(0, super_ident.clone());
        }
        path
    }

    /// Gets the protobuf type name for a given name.
    ///
    /// Constructs the protobuf type name by joining the current path
    /// with the given name using dots as separators.
    ///
    /// # Arguments
    ///
    /// * `name` - The type name
    ///
    /// # Returns
    ///
    /// Returns the protobuf type name as a string.
    pub(crate) fn get_proto_type_name(&self, name: &str) -> String {
        if self.path.is_empty() {
            return name.to_string();
        }
        format!("{}.{}", self.path.join("."), name)
    }

    /// Gets the fully-qualified protobuf type name.
    ///
    /// Constructs the fully-qualified protobuf type name by prepending
    /// the package name to the type name.
    ///
    /// # Arguments
    ///
    /// * `name` - The type name
    ///
    /// # Returns
    ///
    /// Returns the fully-qualified protobuf type name as a string.
    pub(crate) fn get_proto_full_type_name(&self, name: &str) -> String {
        let type_name = self.get_proto_type_name(name);
        if self.package_name.is_empty() {
            return type_name;
        }
        format!("{}.{}", self.package_name, type_name)
    }

    /// Gets the identifier from a type name reference.
    ///
    /// Resolves a type name reference to an actual Rust type path,
    /// taking into account external path mappings.
    ///
    /// # Arguments
    ///
    /// * `type_name_reference` - The protobuf type name reference
    ///
    /// # Returns
    ///
    /// Returns a `syn::ExprPath` representing the resolved type.
    pub(crate) fn get_ident_from_type_name_reference(&self, type_name_reference: &str) -> ExprPath {
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

    /// Gets the identifier for a oneof field.
    ///
    /// Constructs a Rust identifier for a oneof field within a message,
    /// applying appropriate case conversions and naming conventions.
    ///
    /// # Arguments
    ///
    /// * `message` - The containing message descriptor
    /// * `oneof` - The oneof descriptor
    ///
    /// # Returns
    ///
    /// Returns a `syn::ExprPath` representing the oneof identifier.
    pub(crate) fn get_oneof_ident(
        &self,
        message: &DescriptorProto,
        oneof: &OneofDescriptorProto,
    ) -> ExprPath {
        use std::fmt::Write;
        let mut ident = String::new();
        for parent in &self.path {
            ident.push_str(&str_to_case(parent, Case::Snake));
            ident.push_str("::");
        }
        write!(
            ident,
            "{}::{}",
            str_to_case(message.name(), Case::Snake),
            str_to_case(oneof.name(), Case::Pascal)
        )
        .unwrap();
        syn::parse_str::<ExprPath>(&ident).unwrap()
    }
}
