use super::{SqlArgumentStyle, SqlDialect};

pub fn get_identifier(dialect: SqlDialect, name: &str, escape: bool) -> String {
    match dialect {
        SqlDialect::Postgres => {
            if escape {
                let mut parts = name.split('.');
                let mut result = String::new();
                if let Some(first) = parts.next() {
                    result.push_str(&format!("\"{first}\""));
                }
                for part in parts {
                    result.push('.');
                    result.push_str(&format!("\"{part}\""));
                }
                result
            } else {
                name.into()
            }
        }
        SqlDialect::MySql => {
            if escape {
                let mut parts = name.split('.');
                let mut result = String::new();
                if let Some(first) = parts.next() {
                    result.push_str(&format!("`{first}`"));
                }
                for part in parts {
                    result.push('.');
                    result.push_str(&format!("`{part}`"));
                }
                result
            } else {
                name.into()
            }
        }
    }
}

pub fn get_argument_parameter(style: &SqlArgumentStyle, argument: usize) -> String {
    match style {
        SqlArgumentStyle::Indexed { prefix } => {
            format!("{prefix}{argument}")
        }
        SqlArgumentStyle::Positional { symbol } => symbol.clone(),
    }
}
