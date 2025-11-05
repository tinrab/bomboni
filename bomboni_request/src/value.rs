use pest::iterators::Pair;
use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter, Write},
};

use bomboni_common::date_time::UtcDateTime;

use crate::{
    filter::{
        error::{FilterError, FilterResult},
        parser::Rule,
    },
    schema::ValueType,
};

/// Query value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Integer value.
    Integer(i64),
    /// Float value.
    Float(f64),
    /// Boolean value.
    Boolean(bool),
    /// String value.
    String(String),
    /// Timestamp value.
    Timestamp(UtcDateTime),
    /// Repeated value.
    Repeated(Vec<Value>),
    /// Any value.
    Any,
}

impl Value {
    /// Gets the value type.
    pub const fn value_type(&self) -> Option<ValueType> {
        match self {
            Self::Integer(_) => Some(ValueType::Integer),
            Self::Float(_) => Some(ValueType::Float),
            Self::Boolean(_) => Some(ValueType::Boolean),
            Self::String(_) => Some(ValueType::String),
            Self::Timestamp(_) => Some(ValueType::Timestamp),
            Self::Repeated(_) => None,
            Self::Any => Some(ValueType::Any),
        }
    }

    /// Parses a value from a pest pair.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn parse(pair: &Pair<'_, Rule>) -> FilterResult<Self> {
        match pair.as_rule() {
            Rule::String => {
                let value = pair.as_str();
                UtcDateTime::parse_rfc3339(value).map_or_else(
                    |_| {
                        let lexeme = pair.as_str();
                        Ok(Self::String(lexeme[1..lexeme.len() - 1].into()))
                    },
                    |value| Ok(value.into()),
                )
            }
            Rule::Boolean => Ok(Self::Boolean(pair.as_str() == "true")),
            Rule::Number => pair.as_str().parse::<i64>().map_or_else(
                |_| {
                    pair.as_str().parse::<f64>().map_or_else(
                        |_| Err(FilterError::InvalidNumber(pair.as_str().into())),
                        |value| Ok(Self::Float(value)),
                    )
                },
                |value| Ok(Self::Integer(value)),
            ),
            Rule::Any => Ok(Self::Any),
            _ => Err(FilterError::ExpectedValue),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => value.fmt(f),
            Self::Float(value) => value.fmt(f),
            Self::Boolean(value) => value.fmt(f),
            Self::String(value) => {
                f.write_char('"')?;
                value.fmt(f)?;
                f.write_char('"')
            }
            Self::Timestamp(value) => {
                f.write_char('"')?;
                value.format_rfc3339().unwrap().fmt(f)?;
                f.write_char('"')
            }
            Self::Repeated(values) => {
                write!(
                    f,
                    "[{}]",
                    values
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Self::Any => f.write_char('*'),
        }
    }
}

impl PartialOrd<Self> for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Self::Integer(lhs) => {
                if let Self::Integer(rhs) = other {
                    lhs.partial_cmp(rhs)
                } else {
                    None
                }
            }
            Self::Float(lhs) => {
                if let Self::Float(rhs) = other {
                    lhs.partial_cmp(rhs)
                } else {
                    None
                }
            }
            Self::Boolean(lhs) => {
                if let Self::Boolean(rhs) = other {
                    lhs.partial_cmp(rhs)
                } else {
                    None
                }
            }
            Self::String(lhs) => {
                if let Self::String(rhs) = other {
                    lhs.partial_cmp(rhs)
                } else {
                    None
                }
            }
            Self::Timestamp(lhs) => {
                if let Self::Timestamp(rhs) = other {
                    lhs.partial_cmp(rhs)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Integer(i64::from(value))
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Float(f64::from(value))
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<UtcDateTime> for Value {
    fn from(value: UtcDateTime) -> Self {
        Self::Timestamp(value)
    }
}

impl From<Vec<Self>> for Value {
    fn from(values: Vec<Self>) -> Self {
        Self::Repeated(values)
    }
}

#[cfg(feature = "postgres")]
const _: () = {
    use bytes::BytesMut;
    use postgres_types::{IsNull, ToSql, Type, to_sql_checked};

    impl ToSql for Value {
        fn to_sql(
            &self,
            ty: &Type,
            out: &mut BytesMut,
        ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>>
        where
            Self: Sized,
        {
            match self {
                Self::Integer(value) => value.to_sql(ty, out),
                Self::Float(value) => value.to_sql(ty, out),
                Self::Boolean(value) => value.to_sql(ty, out),
                Self::String(value) => value.to_sql(ty, out),
                Self::Timestamp(value) => value.to_sql(ty, out),
                Self::Repeated(values) => values.to_sql(ty, out),
                Self::Any => Ok(IsNull::No),
            }
        }

        fn accepts(ty: &Type) -> bool {
            matches!(
                ty,
                &Type::INT8
                    | &Type::FLOAT8
                    | &Type::BOOL
                    | &Type::VARCHAR
                    | &Type::TEXT
                    | &Type::TIMESTAMPTZ
            )
        }

        to_sql_checked!();
    }

    impl<'a> FromIterator<&'a Value> for Vec<&'a (dyn ToSql + Sync)> {
        fn from_iter<T: IntoIterator<Item = &'a Value>>(iter: T) -> Self {
            let mut elements: Vec<&(dyn ToSql + Sync)> = Vec::new();
            for e in iter {
                elements.push(e);
            }
            elements
        }
    }
};

#[cfg(feature = "mysql")]
const _: () = {
    use mysql_common::Value as MySqlValue;
    use time::PrimitiveDateTime;

    impl From<Value> for MySqlValue {
        fn from(value: Value) -> Self {
            match value {
                Value::Integer(value) => value.into(),
                Value::Float(value) => value.into(),
                Value::Boolean(value) => value.into(),
                Value::String(value) => value.into(),
                Value::Timestamp(value) => PrimitiveDateTime::from(value).into(),
                Value::Repeated(_values) => {
                    todo!("repeated values are not yet supported for mysql")
                }
                Value::Any => Self::NULL,
            }
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(Value::String("foo".into()).to_string(), "\"foo\"");
        assert_eq!(
            Value::Timestamp(UtcDateTime::UNIX_EPOCH).to_string(),
            "\"1970-01-01T00:00:00Z\""
        );
        assert_eq!(
            Value::Repeated(vec![Value::Integer(1), 2.into(), 3.into()]).to_string(),
            "[1, 2, 3]"
        );
    }
}
