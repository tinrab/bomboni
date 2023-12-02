use proc_macro2::Ident;
use syn::{GenericArgument, Path, PathArguments, Type, TypePath};

pub fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        path.segments.len() == 1 && path.segments[0].ident == "Option"
    } else {
        false
    }
}

pub fn check_proto_type(ty: &Type) -> (bool, bool, bool) {
    let (mut is_option, mut is_nested, mut is_string) = (false, false, false);

    if let Type::Path(type_path) = ty {
        let segment = type_path.path.segments.first().unwrap();
        if segment.ident == "Option" {
            is_option = true;
            if let PathArguments::AngleBracketed(args) = &segment.arguments {
                if let GenericArgument::Type(Type::Path(TypePath {
                    path: Path { segments, .. },
                    ..
                })) = args.args.first().unwrap()
                {
                    let nested_arg = segments.first().unwrap();
                    let ident_type = check_proto_ident_type(&nested_arg.ident);
                    is_nested = ident_type.0;
                    is_string = ident_type.1;
                }
            }
        } else {
            let ident_type = check_proto_ident_type(&segment.ident);
            is_nested = ident_type.0;
            is_string = ident_type.1;
        }
    }

    (is_option, is_nested, is_string)
}

pub fn check_proto_ident_type(ident: &Ident) -> (bool, bool) {
    let (mut is_nested, mut is_string) = (false, false);
    if ident == "String" {
        is_string = true;
    } else {
        // Assume nested message begin with a capital letter
        is_nested = ident.to_string().chars().next().unwrap().is_uppercase();
    }
    (is_nested, is_string)
}
