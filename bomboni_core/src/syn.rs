use syn::{Type, TypePath};

/// Macro for formatting documentation comments.
#[macro_export]
macro_rules! format_comment {
    ($($arg:tt)*) => {{
        let content = ::proc_macro2::Literal::string(&format!(" {}", format!($($arg)*)));
        ::quote::quote! {
            #[doc = #content]
        }
    }};
}

/// Checks if a type is `PhantomData`.
#[must_use]
pub fn type_is_phantom(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        path.segments
            .last()
            .is_some_and(|path| path.ident == "PhantomData")
    } else {
        false
    }
}

/// Checks if a type is `Option`.
#[must_use]
pub fn type_is_option(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        path.segments
            .last()
            .is_some_and(|path| path.ident == "Option")
    } else {
        false
    }
}
