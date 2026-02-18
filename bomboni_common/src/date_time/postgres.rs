use bytes::BytesMut;
use postgres_types::{FromSql, IsNull, ToSql, Type, accepts, to_sql_checked};
use time::OffsetDateTime;

use super::UtcDateTime;

impl ToSql for UtcDateTime {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        OffsetDateTime::from(*self).to_sql(ty, out)
    }

    accepts!(TIMESTAMP, TIMESTAMPTZ);

    to_sql_checked!();
}

impl<'a> FromSql<'a> for UtcDateTime {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let offset_date_time = OffsetDateTime::from_sql(ty, raw)?;
        Ok(offset_date_time.into())
    }

    accepts!(TIMESTAMP, TIMESTAMPTZ);
}
