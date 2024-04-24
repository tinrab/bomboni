use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeSet;

use crate::parse::{
    field_type_info::FieldTypeInfo,
    options::{FieldExtract, FieldExtractStep, ParseFieldOptions},
};

pub fn expand_field_extract(
    extract: &FieldExtract,
    field_clone_set: &BTreeSet<String>,
    field_path_wrapper: Option<&TokenStream>,
) -> (TokenStream, String) {
    let mut field_path = String::new();
    let mut extract_impl = quote!();
    let mut extracted_source: bool = false;

    // In case of enums, the first "field" is the enum variant
    if !matches!(extract.steps.first(), Some(FieldExtractStep::Field(_))) {
        extract_impl.extend(quote! {
            let target = source;
        });
        extracted_source = true;
    }

    for step in &extract.steps {
        match step {
            FieldExtractStep::Field(field_name) => {
                field_path = if field_path.is_empty() {
                    field_name.clone()
                } else {
                    format!("{field_path}.{field_name}")
                };

                let source_ident = if extracted_source {
                    quote! { target }
                } else {
                    extracted_source = true;
                    quote! { source }
                };
                let field_ident = format_ident!("{field_name}");

                if field_clone_set.contains(&field_path) {
                    extract_impl.extend(quote! {
                        // TODO: Avoid cloning in some cases
                        let target = #source_ident.#field_ident.clone();
                    });
                } else {
                    extract_impl.extend(quote! {
                        let target = #source_ident.#field_ident;
                    });
                }
            }
            FieldExtractStep::Unwrap => {
                let error_path = make_field_error_path(&field_path, field_path_wrapper);
                extract_impl.extend(quote! {
                    let target = target.ok_or_else(|| {
                        RequestError::path(
                            #error_path,
                            CommonError::RequiredFieldMissing,
                        )
                    })?;
                });
            }
            FieldExtractStep::UnwrapOr(expr) => {
                extract_impl.extend(quote! {
                    let target = target.unwrap_or_else(|| #expr);
                });
            }
            FieldExtractStep::UnwrapOrDefault => {
                extract_impl.extend(quote! {
                    let target = target.unwrap_or_default();
                });
            }
            FieldExtractStep::Unbox => {
                extract_impl.extend(quote! {
                    let target = *target;
                });
            }
            FieldExtractStep::StringFilterEmpty => {
                extract_impl.extend(quote! {
                    let target = if target.is_empty() {
                        None
                    } else {
                        Some(target)
                    };
                });
            }
            FieldExtractStep::EnumerationFilterUnspecified => {
                extract_impl.extend(quote! {
                    let target = if target == 0 {
                        None
                    } else {
                        Some(target)
                    };
                });
            }
        }
    }

    (extract_impl, field_path)
}

pub fn make_field_error_path(field_path: &str, before: Option<&TokenStream>) -> TokenStream {
    let mut error_path = if let Some(before) = before {
        quote!(#before,)
    } else {
        assert!(
            !field_path.is_empty(),
            "expected extract plan to begin with a field path"
        );
        quote!()
    };
    for part in field_path.split('.').filter(|part| !part.is_empty()) {
        error_path.extend(quote! {
            PathErrorStep::Field(#part.into()),
        });
    }
    quote!([#error_path])
}

pub fn parse_field_source_extract(source: &str) -> Option<FieldExtract> {
    let mut steps = Vec::new();

    for part in source.split('.') {
        if let Some(stripped) = part.strip_suffix('?') {
            steps.push(FieldExtractStep::Field(stripped.to_string()));
            steps.push(FieldExtractStep::Unwrap);
        } else {
            steps.push(FieldExtractStep::Field(part.to_string()));
        }
    }

    if steps.is_empty() {
        None
    } else {
        Some(FieldExtract { steps })
    }
}

pub fn expand_field_parse_type(
    field_options: &ParseFieldOptions,
    field_type_info: &FieldTypeInfo,
    field_error_path: TokenStream,
) -> TokenStream {
    let check_empty = true;
    let mut parse_impl = quote!();

    if let Some(regex) = field_options.regex.as_ref() {
        parse_impl.extend(quote! {
            static REGEX: ::std::sync::OnceLock<::regex::Regex> = ::std::sync::OnceLock::new();
            let re = REGEX.get_or_init(|| ::regex::Regex::new(#regex).unwrap());
        });
    }

    let inner_wrap_err = match field_type_info.container_ident.as_deref() {
        Some("Vec") => {
            quote!(.insert_path([PathErrorStep::Index(i)], 1))
        }
        Some("HashMap" | "BTreeMap") => {
            quote!(.insert_path([PathErrorStep::Key(key.to_string())], 1))
        }
        _ => quote!(),
    };

    if !field_options.keep_primitive {
        if field_options.enumeration {
            if check_empty {
                parse_impl = quote! {
                    if target == 0 {
                        return Err(RequestError::path(
                            #field_error_path,
                            CommonError::RequiredFieldMissing,
                        ) #inner_wrap_err);
                    }
                };
            }
            parse_impl.extend(quote! {
                let target = target.try_into()
                    .map_err(|_| RequestError::path(#field_error_path, CommonError::InvalidEnumValue) #inner_wrap_err)?;
            });
        } else if let Some(primitive_ident) = field_type_info.primitive_ident.as_ref() {
            if field_options.wrapper {
                if matches!(
                    primitive_ident.as_str(),
                    "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize" | "usize"
                ) {
                    let target_type_ident = format_ident!("{}", primitive_ident);
                    parse_impl = quote! {
                        let target = target.value as #target_type_ident;
                    };
                } else {
                    parse_impl = quote! {
                        let target = target.value;
                    };
                }
            }

            if primitive_ident == "String" {
                if check_empty {
                    parse_impl.extend(quote! {
                        if target.is_empty() {
                            return Err(RequestError::path(
                                #field_error_path,
                                CommonError::RequiredFieldMissing,
                            ) #inner_wrap_err);
                        }
                    });
                }
                if let Some(regex) = field_options.regex.as_ref() {
                    parse_impl.extend(quote! {
                        if !re.is_match(&target) {
                            return Err(RequestError::path(
                                #field_error_path,
                                CommonError::InvalidStringFormat {
                                    expected: #regex.into(),
                                },
                            ) #inner_wrap_err);
                        }
                    });
                }
            } else if field_type_info.primitive_message {
                parse_impl.extend(quote! {
                    let target = target.parse_into()
                        .map_err(|err: RequestError| err.wrap_path(#field_error_path) #inner_wrap_err)?;
                });
            }
        }
    }

    if field_type_info.generic_param.is_some() {
        parse_impl.extend(quote! {
            let target = target.parse_into()
                .map_err(|err: RequestError| err.wrap_path(#field_error_path) #inner_wrap_err)?;
        });
    }

    if let Some(container_ident) = field_type_info.container_ident.as_ref() {
        match container_ident.as_str() {
            "Option" => {
                return quote! {
                    let target = if let Some(target) = target {
                        #parse_impl
                        Some(target)
                    } else {
                        None
                    };
                };
            }
            "Box" => {
                return quote! {
                    let target = {
                        #parse_impl
                        Box::new(target)
                    };
                };
            }
            "Vec" => {
                return quote! {
                    let mut v = Vec::new();
                    for (i, target) in target.into_iter().enumerate() {
                        v.push({
                            #parse_impl
                            target
                        });
                    }
                    let target = v;
                };
            }
            "HashMap" | "BTreeMap" => {
                let container_ident = if container_ident == "HashMap" {
                    quote! { HashMap }
                } else {
                    quote! { BTreeMap }
                };
                let parse_kv_impl = if parse_impl.is_empty() {
                    quote!()
                } else {
                    quote! {
                        let target = {
                            #parse_impl
                            target
                        };
                    }
                };
                return if parse_kv_impl.is_empty() {
                    quote!()
                } else {
                    quote! {
                        let mut m = #container_ident::new();
                        for (key, target) in target.into_iter() {
                            #parse_kv_impl
                            m.insert(key, target);
                        }
                        let target = m;
                    }
                };
            }
            _ => unreachable!(),
        }
    }

    parse_impl
}
