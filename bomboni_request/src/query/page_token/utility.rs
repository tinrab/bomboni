use blake2::Blake2s256;

use crate::filter::FilterComparator;
use crate::ordering::OrderingDirection;
use crate::value::Value;
use crate::{filter::Filter, schema::SchemaMapped};
use blake2::Digest;

use crate::ordering::Ordering;

/// Constructs a filter that selects items greater than `next_item` based on ordering.
/// For example, if the ordering is "age desc", then the filter will be `age <= next_item.age`.
/// "Equals" (>=, <=) is used to ensure that the next item is included in the results.
pub fn get_page_filter<T: SchemaMapped>(ordering: &Ordering, next_item: &T) -> Filter {
    let mut filters = Vec::new();

    for term in ordering.iter() {
        let term_argument = match next_item.get_field(&term.name) {
            Value::Integer(value) => Filter::Value(value.into()),
            Value::Float(value) => Filter::Value(value.into()),
            Value::Boolean(value) => Filter::Value(value.into()),
            Value::String(value) => Filter::Value(value.into()),
            Value::Timestamp(value) => Filter::Value(value.into()),
            Value::Repeated(value) => Filter::Value(value.into()),
            Value::Any => Filter::Value(Value::Any),
        };

        filters.push(Filter::Restriction(
            Box::new(Filter::Name(term.name.clone())),
            match term.direction {
                OrderingDirection::Ascending => FilterComparator::GreaterOrEqual,
                OrderingDirection::Descending => FilterComparator::LessOrEqual,
            },
            Box::new(term_argument),
        ));
    }

    // Disjunction?
    Filter::Conjunction(filters)
}

/// Constructs a page key from a filter and ordering.
/// The key should be completely different for different filters and orderings.
pub fn make_page_key<const N: usize>(filter: &Filter, ordering: &Ordering, salt: &[u8]) -> [u8; N] {
    let mut hasher = Blake2s256::new();

    hasher.update(filter.to_string().as_bytes());
    hasher.update(ordering.to_string().as_bytes());
    hasher.update(salt);

    let res = hasher.finalize();
    // TODO: other than 32 bytes?
    debug_assert_eq!(res.len(), N);
    let key: [u8; N] = res[..].try_into().unwrap();

    key
}
