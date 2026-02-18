use itertools::Itertools;
use std::collections::BTreeMap;

pub use filter::SqlFilterBuilder;
pub use ordering::SqlOrderingBuilder;
pub use query::{QuerySqlBuilder, QuerySqlStatement};

mod filter;
mod ordering;
mod query;
pub(crate) mod utility;

/// SQL dialect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlDialect {
    /// `PostgreSQL` dialect.
    Postgres,
    /// `MySQL` dialect.
    MySql,
}

/// SQL argument style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlArgumentStyle {
    /// Indexed arguments with prefix.
    Indexed {
        /// Argument prefix.
        prefix: String,
    },
    /// Positional arguments with symbol.
    Positional {
        /// Argument symbol.
        symbol: String,
    },
}

/// SQL rename map for members and functions.
#[derive(Debug, Clone, Default)]
pub struct SqlRenameMap {
    /// Member rename map.
    pub members: BTreeMap<String, String>,
    /// Function rename map.
    pub functions: BTreeMap<String, String>,
}

impl SqlRenameMap {
    /// Creates a new SQL rename map.
    pub const fn new(
        members: BTreeMap<String, String>,
        functions: BTreeMap<String, String>,
    ) -> Self {
        Self { members, functions }
    }

    /// Renames a member.
    pub fn rename_member(&self, name: &str) -> String {
        Self::rename(&self.members, name)
    }

    /// Renames a function.
    pub fn rename_function(&self, name: &str) -> String {
        Self::rename(&self.functions, name)
    }

    fn rename(rename_map: &BTreeMap<String, String>, name: &str) -> String {
        let mut original = Vec::new();
        let mut renamed = String::new();
        for name_part in name.split('.') {
            let prefix = if original.is_empty() {
                name_part.to_string()
            } else {
                format!("{}.{}", original.iter().join("."), name_part)
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
}
