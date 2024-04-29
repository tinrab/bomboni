use bomboni_core::syn::type_is_phantom;
use std::collections::BTreeSet;
use syn::{GenericArgument, PathArguments, Type, TypePath};

use crate::parse::{
    options::{FieldExtract, FieldExtractStep, ParseDerive, ParseField},
    parse_utility::parse_field_source_extract,
};

pub fn get_field_extract(field: &ParseField) -> syn::Result<FieldExtract> {
    let target_ident = field.ident.as_ref().unwrap();

    let mut steps = if let Some(source) = field.options.source.as_ref() {
        let source_extract = parse_field_source_extract(source)
            .ok_or_else(|| syn::Error::new_spanned(target_ident, "invalid source"))?;
        source_extract.steps
    } else {
        vec![FieldExtractStep::Field(target_ident.to_string())]
    };

    if let Some(extract) = field.options.extract.clone() {
        steps.extend(extract.steps);
    }

    // In proto3 generated code, primitive message fields are wrapped in an Option.
    // Insert an unwrap step for these fields by default.
    if let Some(field_type_info) = field.type_info.as_ref() {
        if field.options.extract.is_none()
            && field.options.source.is_none()
            && field.options.derive.is_none()
            && !field.options.oneof
            && !field.options.enumeration
            && matches!(
                field_type_info.container_ident.as_deref(),
                None | Some("Option")
            )
            && field_type_info.primitive_message
            && field_type_info.primitive_ident.is_some()
            && field_type_info.generic_param.is_none()
        {
            steps.insert(1, FieldExtractStep::Unwrap);
        }
    }

    Ok(FieldExtract { steps })
}

pub fn get_field_clone_set(fields: &[ParseField]) -> syn::Result<BTreeSet<String>> {
    let mut clone_set = BTreeSet::new();
    let mut visited = BTreeSet::new();

    for field in fields.iter().filter(|field| {
        !field.options.skip
            && field.resource.is_none()
            && !type_is_phantom(&field.ty)
            && field.list_query.is_none()
            && field.search_query.is_none()
            && (field.options.derive.is_none()
                || matches!(
                    field.options.derive.as_ref(),
                    Some(ParseDerive { source_borrow, .. }) if !source_borrow,
                ))
    }) {
        let extract = get_field_extract(field)?;
        let mut field_path = String::new();
        for step in extract.steps {
            if let FieldExtractStep::Field(field_name) = step {
                field_path = if field_path.is_empty() {
                    field_name
                } else {
                    format!("{field_path}.{field_name}")
                };

                let steps: Vec<_> = field_path.split('.').collect();
                let mut path = String::new();
                for step in steps {
                    path = if path.is_empty() {
                        step.to_string()
                    } else {
                        format!("{path}.{step}")
                    };
                    if visited.contains(&path) {
                        clone_set.insert(path.clone());
                    }
                    visited.insert(path.clone());
                }
            }
        }
    }

    for field in fields.iter().filter(|field| !field.options.skip) {
        if let Some(resource) = field.resource.as_ref() {
            if resource.name.parse {
                let field_path = resource.name.source.to_string();
                if visited.contains(&field_path) {
                    clone_set.insert(field_path.clone());
                }
                visited.insert(field_path);
            }
        }
    }

    Ok(clone_set)
}

pub fn get_query_field_token_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if path.segments.len() == 1
            && (path.segments[0].ident == "ListQuery" || path.segments[0].ident == "SearchQuery")
        {
            if let PathArguments::AngleBracketed(args) = &path.segments[0].arguments {
                if let GenericArgument::Type(ty) = args.args.first().unwrap() {
                    return Some(ty);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use darling::ast::Data;
    use syn::parse_quote;

    use crate::parse::options::ParseOptions;

    use super::*;

    #[test]
    fn get_resource_clone_set() {
        macro_rules! assert_clone_set {
            ($source:expr, $set:expr $(,)?) => {{
                let options = ParseOptions::parse(&$source).unwrap();
                let Data::Struct(data) = &options.data else {
                    unreachable!()
                };
                assert_eq!(
                    get_field_clone_set(&data.fields).unwrap(),
                    $set.into_iter().map(ToString::to_string).collect(),
                );
            }};
        }

        assert_clone_set!(
            parse_quote! {
                #[parse(source = Item)]
                struct Item {
                    #[parse(resource)]
                    pub resource: ParsedResource,
                    #[parse(source = "name", derive { module = derive_module })]
                    pub id: String,
                }
            },
            ["name"],
        );
        assert_clone_set!(
            parse_quote! {
                #[parse(source = Item)]
                struct Item {
                    #[parse(resource)]
                    pub resource: ParsedResource,
                    #[parse(source = "name", derive { module = derive_module, source_borrow })]
                    pub id: String,
                }
            },
            Vec::<&str>::new(),
        );
    }
}
