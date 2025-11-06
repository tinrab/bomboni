pub use convert_case::Case;
use convert_case::{Boundary, Casing};

/// Converts a string to the specified case.
pub fn str_to_case<S: AsRef<str>>(s: S, case: Case) -> String {
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
    s.as_ref().with_boundaries(BOUNDARIES).to_case(case)
}
