use syn::{GenericArgument, Path, PathArguments, PathSegment, Type, TypePath};

pub fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        path.segments.len() == 1 && path.segments[0].ident == "Option"
    } else {
        false
    }
}

#[derive(Debug, Default)]
pub struct ProtoTypeInfo {
    pub is_option: bool,
    pub is_nested: bool,
    pub is_string: bool,
    pub is_box: bool,
    pub is_vec: bool,
}

pub fn get_proto_type_info(ty: &Type) -> ProtoTypeInfo {
    let mut info = ProtoTypeInfo::default();

    if let Type::Path(type_path) = ty {
        let segment = type_path.path.segments.first().unwrap();
        if segment.ident == "Option" {
            info.is_option = true;
            get_proto_nested_type(&mut info, segment);
        } else if segment.ident == "Box" {
            info.is_box = true;
            get_proto_nested_type(&mut info, segment);
        } else if segment.ident == "Vec" {
            info.is_vec = true;
            get_proto_nested_type(&mut info, segment);
        } else if segment.ident == "String" {
            info.is_string = true;
        } else {
            // Assume nested message begin with a capital letter
            info.is_nested = segment
                .ident
                .to_string()
                .chars()
                .next()
                .unwrap()
                .is_uppercase();
        }
    }

    info
}

fn get_proto_nested_type(info: &mut ProtoTypeInfo, segment: &PathSegment) {
    if let PathArguments::AngleBracketed(args) = &segment.arguments {
        if let GenericArgument::Type(Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        })) = args.args.first().unwrap()
        {
            let nested_arg = segments.first().unwrap();
            if nested_arg.ident == "String" {
                info.is_string = true;
            } else {
                // Assume nested message begin with a capital letter
                info.is_nested = nested_arg
                    .ident
                    .to_string()
                    .chars()
                    .next()
                    .unwrap()
                    .is_uppercase();
            }
        }
    }
}
