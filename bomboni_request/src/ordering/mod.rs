use std::{
    cmp,
    collections::BTreeSet,
    fmt::{self, Display, Formatter},
    ops::{Deref, DerefMut},
};

use crate::{
    ordering::error::{OrderingError, OrderingResult},
    schema::{Schema, SchemaMapped},
    string::String,
};

pub mod error;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ordering(Vec<OrderingTerm>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderingTerm {
    pub name: String,
    pub direction: OrderingDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderingDirection {
    Ascending,
    Descending,
}

impl Ordering {
    pub fn new(terms: Vec<OrderingTerm>) -> Self {
        Self(terms)
    }

    pub fn parse(source: &str) -> OrderingResult<Self> {
        let mut terms = Vec::new();
        let mut term_names = BTreeSet::<&str>::new();

        for parts in source
            .split(',')
            .map(|part| part.split_whitespace().collect::<Vec<_>>())
            .filter(|parts| !parts.is_empty())
        {
            if parts.len() > 2 {
                return Err(OrderingError::InvalidTermFormat(parts.join(" ").into()));
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
                _ => return Err(OrderingError::InvalidTermFormat(parts.join(" ").into())),
            }
        }

        Ok(Self(terms))
    }

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
        for (i, term) in self.iter().enumerate() {
            f.write_str(&term.to_string())?;
            if i < self.len() - 1 {
                f.write_str(", ")?;
            }
        }
        Ok(())
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

#[cfg(feature = "testing")]
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
            display_name: "a".into(),
            age: 30,
        };
        let b = UserItem {
            id: "2".into(),
            display_name: "b".into(),
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
