use itertools::Itertools;
use std::collections::BTreeMap;

pub use filter::SqlFilterBuilder;
pub use ordering::SqlOrderingBuilder;
pub use query::{QuerySqlBuilder, QuerySqlStatement};

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
