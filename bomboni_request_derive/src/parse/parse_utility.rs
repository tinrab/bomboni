#![allow(clippy::option_if_let_else)]

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::collections::BTreeSet;

use crate::parse::{
    field_type_info::FieldTypeInfo,
    options::{FieldExtract, FieldExtractStep, ParseFieldOptions},
};

use super::options::ParseConvert;

pub fn expand_field_extract(
    extract: &FieldExtract,
    field_clone_set: &BTreeSet<String>,
    field_type_info: Option<&FieldTypeInfo>,
    field_path_wrapper: Option<&TokenStream>,
    borrow: bool,
) -> (TokenStream, TokenStream, String) {
    let mut extract_impl = quote!();
    let mut get_impl = quote!();
    let mut field_path = String::new();

    let mut source_steps = extract.steps.iter().enumerate().peekable();
    if let Some((_, FieldExtractStep::Field(field_name))) = source_steps.peek() {
        source_steps.next();

        field_path.clone_from(field_name);
        let source_ident = if borrow {
            quote! { &source }
        } else {
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
    } else {
        extract_impl.extend(if borrow {
            quote! {
                let target = &source;
            }
        } else {
            quote! {
                let target = source;
            }
        });
    }

    let last_unwrap_step = extract
        .steps
        .iter()
        .rposition(|step| matches!(step, FieldExtractStep::Unwrap));
    let mut target_option = if field_type_info
        .and_then(|field_type_info| field_type_info.container_ident.as_deref())
        == Some("Option")
    {
        Some(())
    } else {
        None
    };

    for (i, step) in source_steps {
        let block_impl = if let Some(last_unwrap_step) = last_unwrap_step {
            if last_unwrap_step < i {
                &mut get_impl
            } else {
                &mut extract_impl
            }
        } else {
            &mut extract_impl
        };

        match step {
            FieldExtractStep::Field(field_name) => {
                field_path = if field_path.is_empty() {
                    field_name.clone()
                } else {
                    format!("{field_path}.{field_name}")
                };
                let field_ident = format_ident!("{field_name}");

                if field_clone_set.contains(&field_path) {
                    block_impl.extend(quote! {
                        // TODO: Avoid cloning in some cases
                        let target = target.#field_ident.clone();
                    });
                } else {
                    block_impl.extend(quote! {
                        let target = target.#field_ident;
                    });
                }
            }
            FieldExtractStep::Unwrap => {
                if !(last_unwrap_step == Some(i) && target_option.take().is_some()) {
                    let error_path = make_field_error_path(&field_path, field_path_wrapper);
                    block_impl.extend(quote! {
                        let target = target.ok_or_else(|| {
                            RequestError::path(
                                #error_path,
                                CommonError::RequiredFieldMissing,
                            )
                        })?;
                    });
                }
            }
            FieldExtractStep::UnwrapOr(expr) => {
                block_impl.extend(quote! {
                    let target = target.unwrap_or_else(|| #expr);
                });
            }
            FieldExtractStep::UnwrapOrDefault => {
                block_impl.extend(quote! {
                    let target = target.unwrap_or_default();
                });
            }
            FieldExtractStep::Unbox => {
                block_impl.extend(quote! {
                    let target = *target;
                });
            }
            FieldExtractStep::StringFilterEmpty => {
                block_impl.extend(quote! {
                    let target = if target.is_empty() {
                        None
                    } else {
                        Some(target)
                    };
                });
            }
            FieldExtractStep::EnumerationFilterUnspecified => {
                block_impl.extend(quote! {
                    let target = if target == 0 {
                        None
                    } else {
                        Some(target)
                    };
                });
            }
        }
    }

    (extract_impl, get_impl, field_path)
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

pub fn expand_parse_field_type(
    field_options: &ParseFieldOptions,
    field_type_info: &FieldTypeInfo,
    field_error_path: TokenStream,
    get_impl: TokenStream,
) -> TokenStream {
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

    if let Some(try_from) = field_options.try_from.as_ref() {
        parse_impl.extend(quote! {
            let target = TryFrom::<#try_from>::try_from(target)
                .map_err(|_| RequestError::path(#field_error_path, CommonError::FailedConvertValue) #inner_wrap_err)?;
        });
    } else if let Some(ParseConvert { parse, module, .. }) = field_options.convert.as_ref() {
        let convert_impl = parse
            .as_ref()
            .map(ToTokens::to_token_stream)
            .or_else(|| module.as_ref().map(|module| quote!(#module::parse)))
            .unwrap();
        parse_impl.extend(quote! {
            let target = #convert_impl(target)
                .map_err(|err: RequestError| err.wrap_path(#field_error_path) #inner_wrap_err)?;
        });
    } else if field_options.enumeration {
        if !field_options.unspecified {
            parse_impl = quote! {
                if target == 0 {
                    return Err(RequestError::path(
                        #field_error_path,
                        CommonError::RequiredFieldMissing,
                    ) #inner_wrap_err);
                }
            };
        }
        if !field_options.keep_primitive {
            parse_impl.extend(quote! {
                let target = target.try_into()
                    .map_err(|_| RequestError::path(#field_error_path, CommonError::InvalidEnumValue) #inner_wrap_err)?;
            });
        }
    } else if field_options.timestamp {
        parse_impl.extend(quote! {
            let target = target.try_into()
                .map_err(|_| RequestError::path(#field_error_path, CommonError::InvalidDateTime) #inner_wrap_err)?;
        });
    } else if let Some(primitive_ident) = field_type_info.primitive_ident.as_ref() {
        if field_options.wrapper {
            // TODO: Verify that these `.value`s don't require `.unwrap_or_default()`
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
            if !field_options.unspecified {
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
        } else if field_type_info.primitive_message && !field_options.keep_primitive {
            parse_impl.extend(quote! {
                let target = target.parse_into()
                    .map_err(|err: RequestError| err.wrap_path(#field_error_path) #inner_wrap_err)?;
            });
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
                        #get_impl
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
                        #get_impl
                        #parse_impl
                        Box::new(target)
                    };
                };
            }
            "Vec" => {
                return quote! {
                    #get_impl
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
                    quote! {
                        #get_impl
                    }
                } else {
                    quote! {
                        #get_impl
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

    quote! {
        #get_impl
        #parse_impl
    }
}
