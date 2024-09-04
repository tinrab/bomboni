pub mod id_convert {
    use bomboni_common::id::Id;

    use crate::{
        error::{CommonError, RequestResult},
        format_string,
        string::String,
    };

    pub fn parse<S: AsRef<str>>(source: S) -> RequestResult<Id> {
        Ok(source
            .as_ref()
            .parse()
            .map_err(|_| CommonError::InvalidId)?)
    }

    pub fn write<S: From<String>>(id: Id) -> S {
        format_string!("{}", id).into()
    }
}
