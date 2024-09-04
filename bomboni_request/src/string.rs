// use compact_str::{CompactString, ToCompactString};

#[cfg(feature = "compact-str")]
pub type String = compact_str::CompactString;

#[cfg(not(feature = "compact-str"))]
pub type String = std::string::String;

// pub trait Into<String>: compact_str::ToCompactString {
//     // fn to_string(&self) -> String {
//     //     self.to_compact_string()
//     // }
// }

// impl<T: std::string::Into<String> + std::fmt::Display> Into<String> for T {
//     fn to_string(&self) -> String {
//         // Into<String>::to_string(&self).into()
//         CompactString::from(self.to_string())
//     }
// }

// impl Into<String> for &str {
//     fn to_string(&self) -> String {
//         use crate::format_string;
//         format_string!("{}", self)
//     }
// }

// impl Into<String> for CompactString {
//     fn to_string(&self) -> String {
//         self.to_compact_string().into()
//     }
// }

pub mod format_string {
    // use compact_str::format_compact;

    #[cfg(feature = "compact-str")]
    #[macro_export]
    macro_rules! format_string {
        ($($arg:tt)*) => {{
            ::compact_str::format_compact!($($arg)*)
        }}
    }

    #[cfg(not(feature = "compact-str"))]
    #[macro_export]
    macro_rules! format_string {
        ($($arg:tt)*) => {{
            format!($($arg)*)
        }}
    }
}
