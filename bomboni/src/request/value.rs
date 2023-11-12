use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter, Write},
};

use chrono::NaiveDateTime;
use itertools::Itertools;
use pest::iterators::Pair;

use super::{
    filter::{
        error::{FilterError, FilterResult},
        parser::Rule,
    },
    schema::ValueType,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Timestamp(NaiveDateTime),
    Repeated(Vec<Value>),
    Any,
}

impl Value {
    pub fn value_type(&self) -> Option<ValueType> {
        match self {
            Value::Integer(_) => Some(ValueType::Integer),
            Value::Float(_) => Some(ValueType::Float),
            Value::Boolean(_) => Some(ValueType::Boolean),
            Value::String(_) => Some(ValueType::String),
            Value::Timestamp(_) => Some(ValueType::Timestamp),
            Value::Repeated(_) => None,
            Value::Any => Some(ValueType::Any),
        }
    }

    pub fn parse(pair: Pair<'_, Rule>) -> FilterResult<Self> {
        match pair.as_rule() {
            Rule::string => {
                let value = pair.as_str();
                if let Ok(value) = value.parse::<NaiveDateTime>() {
                    Ok(value.into())
                } else {
                    let lexeme = pair.as_str();
                    Ok(Value::String(lexeme[1..lexeme.len() - 1].into()))
                }
            }
            Rule::boolean => Ok(Value::Boolean(pair.as_str() == "true")),
            Rule::number => {
                if let Ok(value) = pair.as_str().parse::<i64>() {
                    Ok(Value::Integer(value))
                } else if let Ok(value) = pair.as_str().parse::<f64>() {
                    Ok(Value::Float(value))
                } else {
                    Err(FilterError::InvalidNumber(pair.as_str().into()))
                }
            }
            Rule::any => Ok(Value::Any),
            _ => Err(FilterError::ExpectedValue),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(value) => value.fmt(f),
            Value::Float(value) => value.fmt(f),
            Value::Boolean(value) => value.fmt(f),
            Value::String(value) => {
                f.write_char('"')?;
                value.fmt(f)?;
                f.write_char('"')
            }
            Value::Timestamp(value) => {
                f.write_char('"')?;
                value.fmt(f)?;
                f.write_char('"')
            }
            Value::Repeated(values) => {
                write!(f, "[{}]", values.iter().join(", "))
            }
            Value::Any => f.write_char('*'),
        }
    }
}

impl PartialOrd<Value> for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        match self {
            Value::Integer(lhs) => {
                if let Value::Integer(rhs) = other {
                    lhs.partial_cmp(rhs)
                } else {
                    None
                }
            }
            Value::Float(lhs) => {
                if let Value::Float(rhs) = other {
                    lhs.partial_cmp(rhs)
                } else {
                    None
                }
            }
            Value::Boolean(lhs) => {
                if let Value::Boolean(rhs) = other {
                    lhs.partial_cmp(rhs)
                } else {
                    None
                }
            }
            Value::String(lhs) => {
                if let Value::String(rhs) = other {
                    lhs.partial_cmp(rhs)
                } else {
                    None
                }
            }
            Value::Timestamp(lhs) => {
                if let Value::Timestamp(rhs) = other {
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
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.into())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Integer(value as i64)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Float(value as f64)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<NaiveDateTime> for Value {
    fn from(value: NaiveDateTime) -> Self {
        Value::Timestamp(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(values: Vec<Value>) -> Self {
        Value::Repeated(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(Value::String("foo".into()).to_string(), "\"foo\"");
        assert_eq!(
            Value::Timestamp(NaiveDateTime::UNIX_EPOCH).to_string(),
            "\"1970-01-01 00:00:00\""
        );
        assert_eq!(
            Value::Repeated(vec![Value::Integer(1), 2.into(), 3.into()]).to_string(),
            "[1, 2, 3]"
        );
    }
}
