pub mod macros {
    macro_rules! format_comment {
        ($($arg:tt)*) => {{
            let content = ::proc_macro2::Literal::string(&format!($($arg)*));
            ::quote::quote! {
                #[doc = #content]
            }
        }};
    }
    pub(crate) use format_comment;
}
