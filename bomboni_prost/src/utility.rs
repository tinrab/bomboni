use convert_case::{Boundary, Case, Casing};

pub fn str_to_case(s: &str, case: Case) -> String {
    static BOUNDARIES: &[Boundary] = &[
        Boundary::Underscore,
        Boundary::Hyphen,
        Boundary::Space,
        Boundary::LowerUpper,
        Boundary::Acronym,
        Boundary::UpperDigit,
        // Boundary::LowerDigit,
        Boundary::DigitUpper,
        Boundary::DigitLower,
    ];
    s.with_boundaries(BOUNDARIES).to_case(case)
}

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
