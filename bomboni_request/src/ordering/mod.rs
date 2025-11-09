use std::{
    cmp,
    collections::BTreeSet,
    fmt::{self, Display, Formatter},
    ops::{Deref, DerefMut},
};

use crate::{
    ordering::error::{OrderingError, OrderingResult},
    schema::{Schema, SchemaMapped},
};

/// Ordering error types.
pub mod error;

/// Query ordering specification.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ordering(Vec<OrderingTerm>);

/// Ordering term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderingTerm {
    /// Field name.
    pub name: String,
    /// Sort direction.
    pub direction: OrderingDirection,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderingDirection {
    /// Ascending order.
    Ascending,
    /// Descending order.
    Descending,
}

impl Ordering {
    /// Creates a new ordering.
    pub const fn new(terms: Vec<OrderingTerm>) -> Self {
        Self(terms)
    }

    /// Parses an ordering string.
    ///
    /// # Errors
    ///
    /// Will return [`OrderingError::InvalidTermFormat`] if ordering term format is invalid.
    /// Will return [`OrderingError::DuplicateField`] if the same field appears multiple times.
    /// Will return [`OrderingError::InvalidDirection`] if ordering direction is invalid.
    pub fn parse(source: &str) -> OrderingResult<Self> {
        let mut terms = Vec::new();
        let mut term_names = BTreeSet::<&str>::new();

        for parts in source
            .split(',')
            .map(|part| part.split_whitespace().collect::<Vec<_>>())
            .filter(|parts| !parts.is_empty())
        {
            if parts.len() > 2 {
                return Err(OrderingError::InvalidTermFormat(parts.join(" ")));
            }
            match parts.as_slice() {
                [name, dir] => {
                    if !term_names.insert(*name) {
                        return Err(OrderingError::DuplicateField((*name).into()));
                    }
                    let direction = match *dir {
                        "asc" => OrderingDirection::Ascending,
                        "desc" => OrderingDirection::Descending,
                        _ => return Err(OrderingError::InvalidDirection((*dir).into())),
                    };
                    terms.push(OrderingTerm {
                        name: (*name).into(),
                        direction,
                    });
                }
                [name] => {
                    if !term_names.insert(name) {
                        return Err(OrderingError::DuplicateField((*name).into()));
                    }
                    terms.push(OrderingTerm {
                        name: (*name).into(),
                        direction: OrderingDirection::Ascending,
                    });
                }
                _ => return Err(OrderingError::InvalidTermFormat(parts.join(" "))),
            }
        }

        Ok(Self(terms))
    }

    /// Evaluates ordering between two items.
    pub fn evaluate<T>(&self, lhs: &T, rhs: &T) -> Option<cmp::Ordering>
    where
        T: SchemaMapped,
    {
        for term in self.iter() {
            let a = lhs.get_field(&term.name);
            let b = rhs.get_field(&term.name);
            match a.partial_cmp(&b)? {
                cmp::Ordering::Less => {
                    return Some(match term.direction {
                        OrderingDirection::Ascending => cmp::Ordering::Less,
                        OrderingDirection::Descending => cmp::Ordering::Greater,
                    });
                }
                cmp::Ordering::Greater => {
                    return Some(match term.direction {
                        OrderingDirection::Ascending => cmp::Ordering::Greater,
                        OrderingDirection::Descending => cmp::Ordering::Less,
                    });
                }
                cmp::Ordering::Equal => {}
            }
        }
        Some(cmp::Ordering::Equal)
    }

    /// Validates the ordering against a schema.
    ///
    /// # Errors
    ///
    /// Will return [`OrderingError::UnknownMember`] if the ordering contains an unknown field name.
    /// Will return [`OrderingError::UnorderedField`] if the ordering contains a field that cannot be ordered.
    pub fn validate(&self, schema: &Schema) -> OrderingResult<()> {
        for term in self.iter() {
            let field_schema = schema
                .get_field(&term.name)
                .ok_or_else(|| OrderingError::UnknownMember(term.name.clone()))?;
            if !field_schema.ordered {
                return Err(OrderingError::UnorderedField(term.name.clone()));
            }
        }
        Ok(())
    }
}

impl Deref for Ordering {
    type Target = Vec<OrderingTerm>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Ordering {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Ordering {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(
            &self
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

impl Display for OrderingTerm {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.direction)
    }
}

impl Display for OrderingDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ascending => f.write_str("asc"),
            Self::Descending => f.write_str("desc"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ordering::OrderingDirection::{Ascending, Descending};
    use crate::testing::schema::UserItem;

    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(
            Ordering::parse(" , user.displayName, task.userId desc").unwrap(),
            Ordering(vec![
                OrderingTerm {
                    name: "user.displayName".into(),
                    direction: Ascending,
                },
                OrderingTerm {
                    name: "task.userId".into(),
                    direction: Descending,
                },
            ])
        );

        assert!(matches!(
            Ordering::parse("user.displayName, user.displayName").unwrap_err(),
            OrderingError::DuplicateField(_)
        ));
        assert!(matches!(
            Ordering::parse("user.id ABC").unwrap_err(),
            OrderingError::InvalidDirection(_)
        ));
    }

    #[test]
    fn evaluate() {
        let a = UserItem {
            id: "1".into(),
            display_name: "a".to_string(),
            age: 30,
        };
        let b = UserItem {
            id: "2".into(),
            display_name: "b".to_string(),
            age: 30,
        };
        assert_eq!(
            Ordering::parse("displayName")
                .unwrap()
                .evaluate(&a, &b)
                .unwrap(),
            cmp::Ordering::Less
        );
        assert_eq!(
            Ordering::parse("displayName desc")
                .unwrap()
                .evaluate(&a, &b)
                .unwrap(),
            cmp::Ordering::Greater
        );
        assert_eq!(
            Ordering::parse("age").unwrap().evaluate(&a, &b).unwrap(),
            cmp::Ordering::Equal
        );
        assert_eq!(
            Ordering::parse("age, displayName")
                .unwrap()
                .evaluate(&a, &b)
                .unwrap(),
            cmp::Ordering::Less
        );
    }
}
