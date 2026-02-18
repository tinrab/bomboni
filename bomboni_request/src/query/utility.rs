use crate::{
    filter::Filter,
    ordering::Ordering,
    query::error::{QueryError, QueryResult},
    schema::{FunctionSchemaMap, Schema},
};

/// Parses a query filter.
///
/// # Errors
///
/// Will return [`QueryError::FilterTooLong`] if filter exceeds maximum length.
/// Will return [`QueryError::FilterError`] if filter cannot be parsed or validated.
pub fn parse_query_filter(
    filter: Option<&str>,
    schema: &Schema,
    schema_functions: Option<&FunctionSchemaMap>,
    max_filter_length: Option<usize>,
) -> QueryResult<Filter> {
    // Empty string is considered as None, because an optional string can be "", from protobuf's side.
    if let Some(filter) = filter.filter(|filter| !filter.is_empty()) {
        if matches!(max_filter_length, Some(max) if filter.len() > max) {
            return Err(QueryError::FilterTooLong);
        }
        let filter = Filter::parse(filter)?;
        filter.validate(schema, schema_functions)?;
        Ok(filter)
    } else {
        Ok(Filter::default())
    }
}

/// Parses a query ordering.
///
/// # Errors
///
/// Will return [`QueryError::OrderingTooLong`] if ordering exceeds maximum length.
/// Will return [`QueryError::OrderingError`] if ordering cannot be parsed or validated.
pub fn parse_query_ordering(
    ordering: Option<&str>,
    schema: &Schema,
    max_ordering_length: Option<usize>,
) -> QueryResult<Ordering> {
    if let Some(ordering) = ordering.filter(|ordering| !ordering.is_empty()) {
        if matches!(max_ordering_length, Some(max) if ordering.len() > max) {
            return Err(QueryError::OrderingTooLong);
        }
        let ordering = Ordering::parse(ordering)?;
        ordering.validate(schema)?;
        Ok(ordering)
    } else {
        Ok(Ordering::default())
    }
}
