use bytes::BytesMut;
use postgres_types::{FromSql, IsNull, ToSql, Type, accepts, to_sql_checked};

use crate::id::Id;

impl ToSql for Id {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        self.to_string().to_sql(ty, out)
    }

    accepts!(VARCHAR, TEXT, CSTRING);

    to_sql_checked!();
}

impl<'a> FromSql<'a> for Id {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let s = <&str as FromSql>::from_sql(ty, raw)?;
        Ok(s.parse()?)
    }

    accepts!(VARCHAR, TEXT, CSTRING);
}
