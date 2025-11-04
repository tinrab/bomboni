pub use convert_case::Case;
use convert_case::{Boundary, Casing};

pub fn str_to_case<S: AsRef<str>>(s: S, case: Case) -> String {
    static BOUNDARIES: &[Boundary] = &[
        Boundary::UNDERSCORE,
        Boundary::HYPHEN,
        Boundary::SPACE,
        Boundary::LOWER_UPPER,
        Boundary::ACRONYM,
        Boundary::UPPER_DIGIT,
        // Boundary::LOWER_DIGIT,
        Boundary::DIGIT_UPPER,
        Boundary::DIGIT_LOWER,
    ];
    s.as_ref().with_boundaries(BOUNDARIES).to_case(case)
}
