use std::fmt;
use std::fmt::{Display, Formatter, Write};
use std::ops::Deref;

use itertools::Itertools;
use parser::{FilterParser, Rule};
use pest::iterators::Pair;
use pest::Parser;

use self::error::FilterResult;

use super::schema::{MemberSchema, Schema, SchemaMapped, ValueType};
use super::value::Value;

pub mod error;

#[allow(clippy::upper_case_acronyms)]
pub(crate) mod parser {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "./request/filter/grammar.pest"]
    pub(crate) struct FilterParser;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    Conjunction(Vec<Filter>),
    Disjunction(Vec<Filter>),
    Negate(Box<Filter>),
    Restriction(Box<Filter>, FilterComparator, Box<Filter>),
    Function(String, Vec<Filter>),
    Composite(Box<Filter>),
    Name(String),
    Value(Value),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterComparator {
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
    NotEqual,
    Has,
}

impl Filter {
    pub fn parse(source: &str) -> FilterResult<Filter> {
        let filter = FilterParser::parse(Rule::filter, source)?.next().unwrap();
        Self::parse_tree(filter)
    }

    fn parse_tree(pair: Pair<Rule>) -> FilterResult<Filter> {
        match pair.as_rule() {
            Rule::filter | Rule::expression => {
                match pair
                    .into_inner()
                    .filter(|pair| pair.as_rule() != Rule::EOI)
                    .map(Self::parse_tree)
                    .exactly_one()
                {
                    Ok(inner_tree) => inner_tree,
                    Err(inner_trees) => Ok(Filter::Conjunction(inner_trees.try_collect()?)),
                }
            }
            Rule::factor => match pair.into_inner().map(Self::parse_tree).exactly_one() {
                Ok(inner_tree) => inner_tree,
                Err(inner_trees) => Ok(Filter::Disjunction(inner_trees.try_collect()?)),
            },
            Rule::term => {
                let lexeme = pair.as_str().trim();
                if lexeme.starts_with("NOT") || lexeme.starts_with('-') {
                    Ok(Filter::Negate(Box::new(Self::parse_tree(
                        pair.into_inner().next().unwrap(),
                    )?)))
                } else {
                    Self::parse_tree(pair.into_inner().next().unwrap())
                }
            }
            Rule::restriction => {
                let mut inner_pairs = pair.into_inner();
                let comparable = inner_pairs.next().unwrap();
                match inner_pairs.next() {
                    Some(inner_pair) => {
                        let comparator = match inner_pair.as_str() {
                            "<" => FilterComparator::Less,
                            "<=" => FilterComparator::LessOrEqual,
                            ">" => FilterComparator::Greater,
                            ">=" => FilterComparator::GreaterOrEqual,
                            "=" => FilterComparator::Equal,
                            "!=" => FilterComparator::NotEqual,
                            ":" => FilterComparator::Has,
                            _ => unreachable!(),
                        };
                        let arg = inner_pairs.next().unwrap();
                        Ok(Filter::Restriction(
                            Box::new(Self::parse_tree(comparable)?),
                            comparator,
                            Box::new(Self::parse_tree(arg)?),
                        ))
                    }
                    None => Self::parse_tree(comparable),
                }
            }
            Rule::comparable => Self::parse_tree(pair.into_inner().next().unwrap()),
            Rule::function => {
                let mut name = String::new();
                let mut arguments = Vec::new();
                let mut argument_list = false;
                for pair in pair.into_inner() {
                    if argument_list {
                        arguments.push(Self::parse_tree(pair)?);
                    } else if pair.as_rule() == Rule::name {
                        name = pair.as_str().into();
                    } else {
                        arguments.push(Self::parse_tree(pair)?);
                        argument_list = true;
                    }
                }
                Ok(Filter::Function(name, arguments))
            }
            Rule::composite => Ok(Filter::Composite(Box::new(Self::parse_tree(
                pair.into_inner().next().unwrap(),
            )?))),
            Rule::name => Ok(Filter::Name(
                pair.into_inner()
                    .map(|identifier| identifier.as_str())
                    .join("."),
            )),
            Rule::string | Rule::boolean | Rule::number | Rule::any => {
                Ok(Filter::Value(Value::parse(pair)?))
            }
            _ => {
                unreachable!("{:?}", pair);
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Filter::Conjunction(parts) => parts.iter().map(|part| part.len()).sum::<usize>(),
            Filter::Disjunction(parts) => parts.iter().map(|part| part.len()).sum::<usize>(),
            Filter::Negate(tree) => 1usize + tree.as_ref().len(),
            Filter::Restriction(comparable, _, arg) => {
                1usize + comparable.as_ref().len() + arg.as_ref().len()
            }
            Filter::Function(tree, args) => {
                1usize + tree.len() + args.iter().map(|arg| arg.len()).sum::<usize>()
            }
            Filter::Composite(composite) => 1usize + composite.as_ref().len(),
            _ => 1usize,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn add_conjunction(&mut self, other: Filter) {
        match self {
            Filter::Conjunction(filters) => {
                filters.push(other);
            }
            _ => {
                *self = Filter::Conjunction(vec![self.clone(), other]);
            }
        }
    }

    pub fn add_disjunction(&mut self, other: Filter) {
        match self {
            Filter::Disjunction(filters) => {
                filters.push(other);
            }
            _ => {
                *self = Filter::Disjunction(vec![self.clone(), other]);
            }
        }
    }

    pub fn evaluate<T>(&self, item: &T) -> Option<Value>
    where
        T: SchemaMapped,
    {
        match self {
            Filter::Conjunction(parts) => {
                let mut res = true;
                for part in parts {
                    if let Some(Value::Boolean(part_res)) = part.evaluate(item) {
                        if !part_res {
                            res = false;
                            break;
                        }
                    } else {
                        return None;
                    }
                }
                Some(Value::Boolean(res))
            }
            Filter::Disjunction(parts) => {
                let mut res = false;
                for part in parts {
                    if let Some(Value::Boolean(part_res)) = part.evaluate(item) {
                        if part_res {
                            res = true;
                            break;
                        }
                    } else {
                        return None;
                    }
                }
                Some(Value::Boolean(res))
            }
            Filter::Negate(composite) => {
                if let Value::Boolean(value) = composite.evaluate(item)? {
                    Some(Value::Boolean(!value))
                } else {
                    None
                }
            }
            Filter::Restriction(comparable, comparator, arg) => {
                let a = comparable.evaluate(item)?;
                match a {
                    Value::Integer(a) => {
                        if let Value::Integer(b) = arg.evaluate(item)? {
                            Some(
                                match comparator {
                                    FilterComparator::Less => a < b,
                                    FilterComparator::LessOrEqual => a <= b,
                                    FilterComparator::Greater => a > b,
                                    FilterComparator::GreaterOrEqual => a >= b,
                                    FilterComparator::Equal => a == b,
                                    FilterComparator::NotEqual => a != b,
                                    FilterComparator::Has => a == b,
                                }
                                .into(),
                            )
                        } else {
                            None
                        }
                    }
                    Value::Float(a) => {
                        if let Value::Float(b) = arg.evaluate(item)? {
                            Some(
                                match comparator {
                                    FilterComparator::Less => a < b,
                                    FilterComparator::LessOrEqual => a <= b,
                                    FilterComparator::Greater => a > b,
                                    FilterComparator::GreaterOrEqual => a >= b,
                                    FilterComparator::Equal => (a - b).abs() < f64::EPSILON,
                                    FilterComparator::NotEqual => (a - b).abs() > f64::EPSILON,
                                    FilterComparator::Has => (a - b).abs() < f64::EPSILON,
                                }
                                .into(),
                            )
                        } else {
                            None
                        }
                    }
                    Value::String(a) => {
                        if let Value::String(b) = arg.evaluate(item)? {
                            Some(
                                match comparator {
                                    FilterComparator::Less => a < b,
                                    FilterComparator::LessOrEqual => a <= b,
                                    FilterComparator::Greater => a > b,
                                    FilterComparator::GreaterOrEqual => a >= b,
                                    FilterComparator::Equal => a == b,
                                    FilterComparator::NotEqual => a != b,
                                    FilterComparator::Has => a.contains(b.as_str()),
                                }
                                .into(),
                            )
                        } else {
                            None
                        }
                    }
                    Value::Boolean(a) => {
                        if let Value::Boolean(b) = arg.evaluate(item)? {
                            match comparator {
                                FilterComparator::Equal => Some((a == b).into()),
                                FilterComparator::NotEqual => Some((a != b).into()),
                                FilterComparator::Has => Some((a == b).into()),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    }
                    Value::Timestamp(a) => {
                        if let Value::Timestamp(b) = arg.evaluate(item)? {
                            Some(
                                match comparator {
                                    FilterComparator::Less => a < b,
                                    FilterComparator::LessOrEqual => a <= b,
                                    FilterComparator::Greater => a > b,
                                    FilterComparator::GreaterOrEqual => a >= b,
                                    FilterComparator::Equal => a == b,
                                    FilterComparator::NotEqual => a != b,
                                    FilterComparator::Has => a == b,
                                }
                                .into(),
                            )
                        } else {
                            None
                        }
                    }
                    Value::Repeated(a) => match comparator {
                        FilterComparator::Equal => {
                            if let Value::Repeated(b) = arg.evaluate(item)? {
                                Some((a == b).into())
                            } else {
                                None
                            }
                        }
                        FilterComparator::NotEqual => {
                            if let Value::Repeated(b) = arg.evaluate(item)? {
                                Some((a != b).into())
                            } else {
                                None
                            }
                        }
                        FilterComparator::Has => {
                            if let Some(b) = arg.evaluate(item) {
                                Some(a.contains(&b).into())
                            } else if let Filter::Composite(composite) = arg.deref() {
                                match composite.as_ref() {
                                    Filter::Conjunction(parts) => Some(Value::Boolean(
                                        parts.iter().map(|part| part.evaluate(item)).all(|value| {
                                            if let Some(value) = value.as_ref() {
                                                a.contains(value)
                                            } else {
                                                false
                                            }
                                        }),
                                    )),
                                    Filter::Disjunction(parts) => Some(Value::Boolean(
                                        parts.iter().map(|part| part.evaluate(item)).any(|value| {
                                            if let Some(value) = value.as_ref() {
                                                a.contains(value)
                                            } else {
                                                false
                                            }
                                        }),
                                    )),
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        }
                        _ => None,
                    },
                    Value::Any => Some(Value::Boolean(true)),
                }
            }
            Filter::Composite(composite) => composite.evaluate(item),
            Filter::Value(value) => Some(value.clone()),
            Filter::Name(name) => Some(item.get_field(name)),
            _ => unimplemented!("evaluate {:?}", self),
        }
    }

    pub fn get_result_value_type(&self, schema: &Schema) -> Option<ValueType> {
        match self {
            Filter::Conjunction(_) => Some(ValueType::Boolean),
            Filter::Disjunction(_) => Some(ValueType::Boolean),
            Filter::Negate(_) => Some(ValueType::Boolean),
            Filter::Restriction(_, _, _) => Some(ValueType::Boolean),
            Filter::Function(name, _) => Some(schema.functions.get(name)?.return_value_type),
            Filter::Composite(composite) => composite.get_result_value_type(schema),
            Filter::Name(name) => schema.get_member(name).and_then(|member| {
                if let MemberSchema::Field(field) = member {
                    Some(field.value_type)
                } else {
                    None
                }
            }),
            Filter::Value(value) => value.value_type(),
        }
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Filter::Conjunction(parts) => {
                parts.iter().map(ToString::to_string).join(" AND ").fmt(f)
            }
            Filter::Disjunction(parts) => parts.iter().map(ToString::to_string).join(" OR ").fmt(f),
            Filter::Negate(tree) => {
                f.write_str("NOT ")?;
                tree.fmt(f)
            }
            Filter::Restriction(comparable, comparator, arg) => match comparator {
                FilterComparator::Has => {
                    comparable.fmt(f)?;
                    f.write_char(':')?;
                    arg.fmt(f)
                }
                _ => {
                    write!(f, "{} {} {}", comparable, comparator, arg)
                }
            },
            Filter::Function(name, args) => {
                write!(
                    f,
                    "{}({})",
                    name,
                    args.iter().map(ToString::to_string).join(", ")
                )
            }
            Filter::Composite(composite) => {
                f.write_char('(')?;
                composite.fmt(f)?;
                f.write_char(')')
            }
            Filter::Name(name) => name.fmt(f),
            Filter::Value(value) => value.fmt(f),
        }
    }
}

impl Display for FilterComparator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FilterComparator::Less => f.write_str("<"),
            FilterComparator::LessOrEqual => f.write_str("<="),
            FilterComparator::Greater => f.write_str(">"),
            FilterComparator::GreaterOrEqual => f.write_str(">="),
            FilterComparator::Equal => f.write_str("="),
            FilterComparator::NotEqual => f.write_str("!="),
            FilterComparator::Has => f.write_str(":"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::schema::{RequestItem, TaskItem, UserItem};

    use super::*;

    #[test]
    fn it_works() {
        Filter::parse(r#"  "#).unwrap();
        Filter::parse(r#""#).unwrap();
        Filter::parse(r#"x"#).unwrap();
        Filter::parse(r#"42"#).unwrap();
        Filter::parse(r#"x =  42"#).unwrap();
        Filter::parse(r#"x=42"#).unwrap();
        Filter::parse(r#"42"#).unwrap();
        Filter::parse(r#"3.14"#).unwrap();
        Filter::parse(r#"NOT a"#).unwrap();
        Filter::parse(r#"NOT    a"#).unwrap();
        Filter::parse(r#"a b AND c AND d"#).unwrap();
        Filter::parse(r#"a < 10 OR a >= 100"#).unwrap();
        Filter::parse(r#"NOT (a OR b)"#).unwrap();
        Filter::parse(r#"-30"#).unwrap();
        Filter::parse(r#"x.b:42"#).unwrap();
        Filter::parse(r#"experiment.rollout <= cohort(request.user)"#).unwrap();
        Filter::parse(r#"expr.type_map.type"#).unwrap();
        Filter::parse(r#"expr.type_map.type"#).unwrap();
        Filter::parse(r#"regex(m.key, a)"#).unwrap();
        Filter::parse(r#"math.mem("30mb")"#).unwrap();
        Filter::parse(r#"regex(m.key, "^.*prod.*$")"#).unwrap();
        Filter::parse(r#"(msg.endsWith("world") AND retries < 10)"#).unwrap();
        Filter::parse(r#"x:*"#).unwrap();

        assert!(Filter::parse(r#"x==42"#).is_err());
        assert!(Filter::parse(r#"--"#).is_err());
    }

    #[test]
    fn parse_tree() {
        use Filter::*;
        use FilterComparator::*;

        let tree = Filter::parse(r#"(a.f(42) AND c < 10) OR x AND y:z AND NOT w != true"#).unwrap();
        assert_eq!(tree.len(), 17);
        assert_eq!(
            tree,
            Conjunction(vec![
                Disjunction(vec![
                    Composite(Box::new(Conjunction(vec![
                        Function("a.f".into(), vec![Value(42.into())]),
                        Restriction(Box::new(Name("c".into())), Less, Box::new(Value(10.into()))),
                    ],),)),
                    Name("x".into()),
                ],),
                Restriction(Box::new(Name("y".into())), Has, Box::new(Name("z".into()))),
                Negate(Box::new(Restriction(
                    Box::new(Name("w".into())),
                    NotEqual,
                    Box::new(Value(true.into())),
                )),),
            ])
        );
    }

    #[test]
    fn to_string() {
        let src = r#"(a.f(42) AND c < 10) OR x AND y:z AND NOT w != true"#;
        let tree = Filter::parse(src).unwrap();
        assert_eq!(tree.to_string(), src);
    }

    #[test]
    fn modify() {
        let mut f = Filter::parse(r#"x=42"#).unwrap();
        f.add_conjunction(Filter::parse(r#"false"#).unwrap());
        assert_eq!(f.to_string(), r#"x = 42 AND false"#);
        let mut f = Filter::parse(r#"x=42"#).unwrap();
        f.add_disjunction(Filter::parse(r#"true"#).unwrap());
        assert_eq!(f.to_string(), r#"x = 42 OR true"#);
    }

    #[test]
    fn evaluate() {
        let f = Filter::parse(
            r#"
            user.age >= 18
            AND user.id:"4"
            AND NOT (task.deleted = false)
            AND task.content = user.displayName
            AND task.tags:("a" "b")
            AND task.tags:("d" OR "a")
        "#,
        )
        .unwrap();

        let res = f
            .evaluate(&RequestItem {
                user: UserItem {
                    id: "42".into(),
                    display_name: "test".into(),
                    age: 30,
                },
                task: TaskItem {
                    id: "1".into(),
                    user_id: "42".into(),
                    content: "test".into(),
                    deleted: true,
                    tags: vec!["a".into(), "b".into(), "c".into()],
                },
            })
            .unwrap();
        assert_eq!(res, Value::Boolean(true));
    }
}
