/// ID conversion helpers.
pub mod id_convert {
    use bomboni_common::id::Id;

    use crate::error::{CommonError, RequestResult};

    /// Parses an ID from a string.
    ///
    /// # Errors
    ///
    /// Will return [`CommonError::InvalidId`] if the source string cannot be parsed as a valid ID.
    pub fn parse<S: AsRef<str>>(source: S) -> RequestResult<Id> {
        Ok(source
            .as_ref()
            .parse()
            .map_err(|_| CommonError::InvalidId)?)
    }

    /// Writes an ID to a string.
    pub fn write(id: Id) -> String {
        id.to_string()
    }
}
