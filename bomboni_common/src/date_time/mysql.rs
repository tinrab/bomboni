use mysql_common::{
    value::convert::{FromValue, ParseIr},
    Value,
};
use time::PrimitiveDateTime;

use crate::date_time::UtcDateTime;

impl FromValue for UtcDateTime {
    type Intermediate = ParseIr<PrimitiveDateTime>;
}

impl From<ParseIr<PrimitiveDateTime>> for UtcDateTime {
    fn from(ir: ParseIr<PrimitiveDateTime>) -> Self {
        ir.commit().into()
    }
}

impl From<UtcDateTime> for Value {
    fn from(value: UtcDateTime) -> Self {
        PrimitiveDateTime::from(value).into()
    }
}
