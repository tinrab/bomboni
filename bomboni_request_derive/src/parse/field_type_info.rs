use syn::{
    GenericArgument, GenericParam, Path, PathArguments, PathSegment, Type, TypePath, TypeTuple,
};

use crate::parse::options::{ParseFieldOptions, ParseOptions};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FieldTypeInfo {
    pub container_ident: Option<String>,
    pub primitive_ident: Option<String>,
    pub primitive_message: bool,
    pub generic_param: Option<String>,
    pub unit: bool,
}

pub fn get_field_type_info(
    options: &ParseOptions,
    field_options: &ParseFieldOptions,
    ty: &Type,
) -> syn::Result<FieldTypeInfo> {
    match ty {
        Type::Path(TypePath { path, .. }) if path.segments.len() == 1 => {
            let segment = path.segments.first().unwrap();
            let mut info = FieldTypeInfo::default();
            get_segment_type_info(&mut info, options, field_options, segment, false).map_err(
                |err: syn::Error| {
                    syn::Error::new(err.span(), format!("{err}; use `derive` instead"))
                },
            )?;

            Ok(info)
        }
        Type::Tuple(TypeTuple { elems, .. }) if elems.is_empty() => Ok(FieldTypeInfo {
            unit: true,
            ..Default::default()
        }),
        _ => Err(syn::Error::new_spanned(
            ty,
            "unsupported field type, use `derive` instead",
        )),
    }
}

fn get_segment_type_info(
    info: &mut FieldTypeInfo,
    options: &ParseOptions,
    field_options: &ParseFieldOptions,
    segment: &PathSegment,
    nested: bool,
) -> syn::Result<()> {
    let ident_str = segment.ident.to_string();

    if options.generics.params.iter().any(|param| {
        matches!(param, GenericParam::Type(type_param) if type_param.ident == segment.ident)
    }) {
        info.generic_param = Some(ident_str.clone());
    } else {
        match ident_str.as_str() {
            "Option" | "Box" | "Vec" | "HashMap" | "BTreeMap" => {
                if nested {
                    return Err(syn::Error::new_spanned(segment, "nesting is too deep"));
                }
                info.container_ident = Some(ident_str.clone());
            }
            _ => {
                info.primitive_ident = Some(ident_str.clone());
                if !info.primitive_message {
                    info.primitive_message =
                        ident_str.chars().next().unwrap().is_uppercase() && ident_str != "String";
                }

                if field_options.wrapper
                    && !matches!(
                        ident_str.as_str(),
                        "String"
                            | "f32"
                            | "f64"
                            | "bool"
                            | "i8"
                            | "i16"
                            | "i32"
                            | "i64"
                            | "u8"
                            | "u16"
                            | "u32"
                            | "u64"
                            | "isize"
                            | "usize"
                    )
                {
                    return Err(syn::Error::new_spanned(
                        segment,
                        format!("invalid wrapper type: `{ident_str}`"),
                    ));
                }
            }
        }
    }

    if let PathArguments::AngleBracketed(args) = &segment.arguments {
        if nested {
            return Err(syn::Error::new_spanned(segment, "nesting is too deep"));
        }

        if info.container_ident.is_none() {
            return Err(syn::Error::new_spanned(
                segment,
                format!("invalid container `{ident_str}`"),
            ));
        }

        match match ident_str.as_str() {
            "HashMap" | "BTreeMap" => args.args.iter().nth(1).ok_or_else(|| {
                syn::Error::new_spanned(args, format!("`{ident_str}`: missing value type"))
            })?,
            _ => args.args.first().ok_or_else(|| {
                syn::Error::new_spanned(args, format!("`{ident_str}`: missing type argument"))
            })?,
        } {
            GenericArgument::Type(Type::Path(TypePath {
                path: Path { segments, .. },
                ..
            })) if segments.len() == 1 => {
                let nested_segment = segments.first().unwrap();
                get_segment_type_info(info, options, field_options, nested_segment, true)?;
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    args,
                    format!("`{ident_str}`: invalid type argument"),
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn it_works() {
        let options = ParseOptions::parse(&parse_quote! {
            #[parse(source = Item)]
            struct Item;
        })
        .unwrap();
        let field_options = ParseFieldOptions {
            source: None,
            source_field: false,
            skip: false,
            keep: false,
            keep_primitive: false,
            unspecified: false,
            extract: None,
            wrapper: false,
            oneof: false,
            enumeration: false,
            timestamp: false,
            regex: None,
            try_from: None,
            convert: None,
            derive: None,
            field_mask: None,
        };

        macro_rules! parse_type_info {
            ($source:expr) => {
                get_field_type_info(&options, &field_options, &syn::parse_str($source).unwrap())
                    .unwrap()
            };
        }

        assert_eq!(
            parse_type_info!("String"),
            FieldTypeInfo {
                primitive_ident: Some("String".into()),
                ..Default::default()
            },
        );
        assert_eq!(
            parse_type_info!("Option<i32>"),
            FieldTypeInfo {
                primitive_ident: Some("i32".into()),
                container_ident: Some("Option".into()),
                ..Default::default()
            },
        );
        assert_eq!(
            parse_type_info!("Box<i32>"),
            FieldTypeInfo {
                primitive_ident: Some("i32".into()),
                container_ident: Some("Box".into()),
                ..Default::default()
            },
        );
        assert_eq!(
            parse_type_info!("Vec<i32>"),
            FieldTypeInfo {
                primitive_ident: Some("i32".into()),
                container_ident: Some("Vec".into()),
                ..Default::default()
            },
        );
        assert_eq!(
            parse_type_info!("HashMap<i32, String>"),
            FieldTypeInfo {
                primitive_ident: Some("String".into()),
                container_ident: Some("HashMap".into()),
                ..Default::default()
            },
        );

        assert_eq!(
            get_field_type_info(
                &options,
                &field_options,
                &syn::parse_str("Option<Option<i32>>").unwrap()
            )
            .unwrap_err()
            .to_string(),
            "nesting is too deep; use `derive` instead",
        );

        assert_eq!(
            get_field_type_info(
                &ParseOptions::parse(&parse_quote! {
                    #[parse(source = Item)]
                    struct Item<T> {}
                })
                .unwrap(),
                &field_options,
                &syn::parse_str("Option<T>").unwrap()
            )
            .unwrap(),
            FieldTypeInfo {
                generic_param: Some("T".into()),
                container_ident: Some("Option".into()),
                ..Default::default()
            },
        );
    }
}
