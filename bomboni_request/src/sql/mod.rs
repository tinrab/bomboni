use std::collections::BTreeMap;

pub use filter::SqlFilterBuilder;
pub use ordering::SqlOrderingBuilder;
pub use query::{QuerySqlBuilder, QuerySqlStatement};

use crate::string::String;

mod filter;
mod ordering;
mod query;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlDialect {
    Postgres,
}

#[derive(Debug, Clone, Default)]
pub struct SqlRenameMap {
    pub members: BTreeMap<String, String>,
    pub functions: BTreeMap<String, String>,
}

impl SqlRenameMap {
    pub fn new(members: BTreeMap<String, String>, functions: BTreeMap<String, String>) -> Self {
        Self { members, functions }
    }

    pub fn rename_member(&self, name: &str) -> String {
        Self::rename(&self.members, name)
    }

    pub fn rename_function(&self, name: &str) -> String {
        Self::rename(&self.functions, name)
    }

    fn rename(rename_map: &BTreeMap<String, String>, name: &str) -> String {
        #[cfg(not(feature = "compact-str"))]
        {
            let mut original = Vec::<&str>::new();
            let mut renamed = String::default();
            for name_part in name.split('.') {
                let prefix = if original.is_empty() {
                    name_part.to_string()
                } else {
                    let mut s = String::new();
                    for part in original.iter() {
                        s.push_str(part);
                        s.push('.');
                    }
                    s.push_str(name_part);
                    s
                };

                let mut renamed_part = name_part.to_string();
                for (source, target) in rename_map {
                    if prefix.ends_with(source) {
                        renamed_part.clone_from(target);
                        break;
                    }
                }

                if !original.is_empty() {
                    renamed.push('.');
                }
                renamed.push_str(&renamed_part);
                original.push(name_part);
            }
            renamed
        }

        #[cfg(feature = "compact-str")]
        {
            use compact_str::ToCompactString;

            let mut original = Vec::<&str>::new();
            let mut renamed = String::default();
            for name_part in name.split('.') {
                let prefix = if original.is_empty() {
                    name_part.to_compact_string()
                } else {
                    let mut s = String::default();
                    for part in original.iter() {
                        s.push_str(part);
                        s.push('.');
                    }
                    s.push_str(name_part);
                    s
                };

                let mut renamed_part = name_part.to_compact_string();
                for (source, target) in rename_map {
                    if prefix.ends_with(source.as_str()) {
                        renamed_part.clone_from(target);
                        break;
                    }
                }

                if !original.is_empty() {
                    renamed.push('.');
                }
                renamed.push_str(&renamed_part);
                original.push(name_part);
            }
            renamed
        }
    }
}
