use mysql_common::{value::convert::FromValue, FromValueError, Value};

use crate::id::{Id, ParseIdError};

mod ir {
    use super::{FromValue, FromValueError, Id, ParseIdError, Value};

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct ParseIdIr<T: FromValue>(T::Intermediate, Id);

    impl TryFrom<Value> for ParseIdIr<String> {
        type Error = <<String as FromValue>::Intermediate as TryFrom<Value>>::Error;

        fn try_from(value: Value) -> Result<Self, Self::Error> {
            match value {
                Value::Bytes(bytes) => match String::from_utf8(bytes) {
                    Ok(x) => {
                        let id = x
                            .parse()
                            .map_err(|err: ParseIdError| FromValueError(err.to_string().into()))?;
                        Ok(ParseIdIr(x, id))
                    }
                    Err(e) => Err(FromValueError(Value::Bytes(e.into_bytes()))),
                },
                v => Err(FromValueError(v)),
            }
        }
    }

    impl From<ParseIdIr<String>> for Id {
        fn from(ir: ParseIdIr<String>) -> Self {
            ir.1
        }
    }

    impl From<ParseIdIr<String>> for Value
    where
        <String as FromValue>::Intermediate: Into<Value>,
    {
        fn from(ir: ParseIdIr<String>) -> Self {
            ir.0.into()
        }
    }
}

pub use ir::ParseIdIr;

impl FromValue for Id {
    type Intermediate = ParseIdIr<String>;
}

impl From<Id> for Value {
    fn from(value: Id) -> Self {
        Value::Bytes(value.to_string().into_bytes())
    }
}
