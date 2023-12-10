use proc_macro2::Ident;
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
    pub map_ident: Option<Ident>,
}

pub fn get_proto_type_info(ty: &Type) -> ProtoTypeInfo {
    let mut info = ProtoTypeInfo::default();
    if let Type::Path(type_path) = ty {
        let segment = type_path.path.segments.first().unwrap();
        update_proto_type_segment(&mut info, segment);
    }
    info
}

fn update_proto_type_segment(info: &mut ProtoTypeInfo, segment: &PathSegment) {
    if segment.ident == "Option" {
        info.is_option = true;
    } else if segment.ident == "Box" {
        info.is_box = true;
    } else if segment.ident == "Vec" {
        info.is_vec = true;
    } else if segment.ident == "HashMap" || segment.ident == "BTreeMap" {
        info.map_ident = Some(segment.ident.clone());
    } else if segment.ident == "String" {
        info.is_string = true;
    } else {
        // Assume nested message begin with a capital letter
        info.is_nested = !info.is_nested
            && segment
                .ident
                .to_string()
                .chars()
                .next()
                .unwrap()
                .is_uppercase();
    }

    if let PathArguments::AngleBracketed(args) = &segment.arguments {
        if let GenericArgument::Type(Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        })) = match segment.ident.to_string().as_str() {
            "HashMap" | "BTreeMap" => args.args.iter().nth(1).unwrap(),
            _ => args.args.first().unwrap(),
        } {
            let nested_segment = segments.first().unwrap();
            update_proto_type_segment(info, nested_segment);
        }
    }
}
