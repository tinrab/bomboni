pub mod id_convert {
    use bomboni_common::id::Id;

    use crate::error::{CommonError, RequestResult};

    pub fn parse<S: AsRef<str>>(source: S) -> RequestResult<Id> {
        Ok(source
            .as_ref()
            .parse()
            .map_err(|_| CommonError::InvalidId)?)
    }

    pub fn write(id: Id) -> String {
        id.to_string()
    }
}
