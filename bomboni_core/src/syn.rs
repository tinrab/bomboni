use syn::{Type, TypePath};

#[macro_export]
macro_rules! format_comment {
    ($($arg:tt)*) => {{
        let content = ::proc_macro2::Literal::string(&format!(" {}", format!($($arg)*)));
        ::quote::quote! {
            #[doc = #content]
        }
    }};
}

pub fn type_is_phantom(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        path.segments
            .last()
            .is_some_and(|path| path.ident == "PhantomData")
    } else {
        false
    }
}

pub fn type_is_option(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        path.segments
            .last()
            .is_some_and(|path| path.ident == "Option")
    } else {
        false
    }
}
