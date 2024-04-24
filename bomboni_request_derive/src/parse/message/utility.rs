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
    Ok(FieldExtract { steps })
}

pub fn get_field_clone_set(fields: &[ParseField]) -> syn::Result<BTreeSet<String>> {
    let mut clone_set = BTreeSet::new();
    let mut visited = BTreeSet::new();

    for field in fields.iter().filter(|field| {
        field.options.derive.is_none()
            && field.resource.is_none()
            && !field.options.skip
            && !type_is_phantom(&field.ty)
            && field.list_query.is_none()
            && field.search_query.is_none()
    }) {
        let mut field_path = String::new();
        let extract = get_field_extract(field)?;
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

    // Clone non-borrowed derived fields.
    // TODO: Optimize.
    for field in fields.iter().filter(|field| !field.options.skip) {
        if let Some(ParseDerive {
            source_field,
            source_borrow,
            ..
        }) = field.options.derive.as_ref()
        {
            if *source_borrow {
                continue;
            }

            let Some(source_field_name) = source_field.as_ref().map(ToString::to_string) else {
                continue;
            };

            for other_field in fields.iter().filter(|other_field| {
                other_field.options.derive.is_none()
                    && other_field.resource.is_none()
                    && !other_field.options.skip
                    && !type_is_phantom(&other_field.ty)
                    && other_field.list_query.is_none()
                    && other_field.search_query.is_none()
            }) {
                let extract = get_field_extract(other_field)?;
                for step in extract.steps {
                    if let FieldExtractStep::Field(field_name) = step {
                        if field_name == source_field_name {
                            clone_set.insert(field_name);
                        }
                    }
                }
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
