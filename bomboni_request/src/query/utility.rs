use crate::{filter::Filter, ordering::Ordering, schema::Schema};

use super::error::{QueryError, QueryResult};

pub fn parse_query_filter(
    filter: Option<&str>,
    schema: &Schema,
    max_filter_length: Option<usize>,
) -> QueryResult<Filter> {
    // Empty string is considered as None, because an optional string can be "", from protobuf's side.
    if let Some(filter) = filter.filter(|filter| !filter.is_empty()) {
        if matches!(max_filter_length, Some(max) if filter.len() > max) {
            return Err(QueryError::FilterTooLong);
        }
        let filter = Filter::parse(filter)?;
        if !filter.is_valid(schema) {
            return Err(QueryError::FilterSchemaMismatch);
        }
        Ok(filter)
    } else {
        Ok(Filter::default())
    }
}

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
        if !ordering.is_valid(schema) {
            return Err(QueryError::OrderingSchemaMismatch);
        }
        Ok(ordering)
    } else {
        Ok(Ordering::default())
    }
}
