use std::{
    cmp,
    collections::BTreeSet,
    fmt::{self, Display, Formatter},
    ops::{Deref, DerefMut},
};

use crate::schema::Schema;

use self::error::{OrderingError, OrderingResult};

use super::schema::SchemaMapped;

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
        let mut term_names = BTreeSet::new();
        for parts in source
            .split(',')
            .map(|part| part.split_whitespace().collect::<Vec<_>>())
            .filter(|parts| !parts.is_empty())
        {
            let mut direction = OrderingDirection::Ascending;
            let mut name = String::new();
            let parts_len = parts.len();
            for (i, part) in parts.into_iter().enumerate() {
                if i < parts_len - 1 {
                    name.push_str(part);
                } else if part == "asc" {
                    direction = OrderingDirection::Ascending;
                } else if part == "desc" {
                    direction = OrderingDirection::Descending;
                } else {
                    name.push_str(part);
                }
            }

            if !term_names.insert(name.clone()) {
                return Err(OrderingError::DuplicateField(name.clone()));
            }

            terms.push(OrderingTerm { name, direction });
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

    pub fn is_valid(&self, schema: &Schema) -> bool {
        for term in self.iter() {
            if let Some(field) = schema.get_field(&term.name) {
                if !field.ordered {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
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
    fn basic() {
        let ordering = Ordering::parse(" , user.displayName, task .userId desc").unwrap();
        assert_eq!(
            ordering,
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
