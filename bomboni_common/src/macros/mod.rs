//! # Common macros.
//!

//! A collection of common macros for Rust.

#[doc(hidden)]
pub mod collections;

/// A macro that creates a static `regex::Regex` instance from a string literal.
///
/// # Examples
///
/// ```
/// use bomboni_common::regex;
///
/// let re = regex!("\\d{4}-\\d{2}-\\d{2}");
/// assert!(re.is_match("2021-08-012"));
/// ```
#[macro_export(local_inner_macros)]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static REGEX: ::std::sync::OnceLock<::regex::Regex> = ::std::sync::OnceLock::new();
        REGEX.get_or_init(|| ::regex::Regex::new($re).unwrap())
    }};
}
