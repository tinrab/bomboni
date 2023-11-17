use sqlx::{
    database::HasArguments, encode::IsNull, error::BoxDynError, mysql::MySqlValueRef, Database,
    Decode, Encode, MySql, Type,
};

use crate::id::Id;

impl Type<MySql> for Id {
    fn type_info() -> <MySql as Database>::TypeInfo {
        <Vec<u8> as Type<MySql>>::type_info()
    }

    fn compatible(ty: &<MySql as Database>::TypeInfo) -> bool {
        <&[u8] as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for Id {
    fn encode_by_ref(&self, buf: &mut <MySql as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        let encoded: Vec<_> = self
            .0
            .to_be_bytes()
            .into_iter()
            // Is this necessary?
            .skip_while(|x| *x == 0)
            .collect();
        <&[u8] as Encode<MySql>>::encode(encoded.as_slice(), buf)
    }
}

impl<'q> Decode<'q, MySql> for Id {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        const MAX_SIZE: usize = std::mem::size_of::<u128>();

        let mut bytes = <&[u8] as Decode<MySql>>::decode(value).map(ToOwned::to_owned)?;

        assert!(bytes.len() <= MAX_SIZE, "invalid bytes length for `Id`");
        let missing = MAX_SIZE - bytes.len();
        if missing != 0 {
            let mut buf = vec![0u8; MAX_SIZE];
            buf.splice(missing..MAX_SIZE, bytes);
            bytes = buf;
        }

        Ok(Self::new(u128::from_be_bytes(bytes.try_into().unwrap())))
    }
}
