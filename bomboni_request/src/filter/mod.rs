//! # Filter
//!
//! Utility for specifying filters on queries, as described in Google AIP standard [1].
//!
//! [1]: https://google.aip.dev/160

use std::fmt;
use std::fmt::{Display, Formatter, Write};

use itertools::Itertools;
use parser::{FilterParser, Rule};
use pest::iterators::Pair;
use pest::Parser;

use crate::filter::error::FilterError;
use crate::schema::{FunctionSchemaMap, MemberSchema, Schema, SchemaMapped, ValueType};
use crate::value::Value;
use error::FilterResult;

pub mod error;

#[allow(clippy::upper_case_acronyms)]
pub(crate) mod parser {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "./filter/grammar.pest"]
    pub struct FilterParser;
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
    pub fn parse(source: &str) -> FilterResult<Self> {
        let filter = FilterParser::parse(Rule::Filter, source)?.next().unwrap();
        Self::parse_tree(filter)
    }

    fn parse_tree(pair: Pair<Rule>) -> FilterResult<Self> {
        match pair.as_rule() {
            Rule::Filter | Rule::Expression => {
                match pair
                    .into_inner()
                    .filter(|pair| pair.as_rule() != Rule::EOI)
                    .map(Self::parse_tree)
                    .exactly_one()
                {
                    Ok(inner_tree) => inner_tree,
                    Err(inner_trees) => Ok(Self::Conjunction(inner_trees.try_collect()?)),
                }
            }
            Rule::Factor => match pair.into_inner().map(Self::parse_tree).exactly_one() {
                Ok(inner_tree) => inner_tree,
                Err(inner_trees) => Ok(Self::Disjunction(inner_trees.try_collect()?)),
            },
            Rule::Term => {
                let lexeme = pair.as_str().trim();
                if lexeme.starts_with("NOT") || lexeme.starts_with('-') {
                    Ok(Self::Negate(Box::new(Self::parse_tree(
                        pair.into_inner().next().unwrap(),
                    )?)))
                } else {
                    Self::parse_tree(pair.into_inner().next().unwrap())
                }
            }
            Rule::Restriction => {
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
                        Ok(Self::Restriction(
                            Box::new(Self::parse_tree(comparable)?),
                            comparator,
                            Box::new(Self::parse_tree(arg)?),
                        ))
                    }
                    None => Self::parse_tree(comparable),
                }
            }
            Rule::Comparable => Self::parse_tree(pair.into_inner().next().unwrap()),
            Rule::Function => {
                let mut name = String::new();
                let mut arguments = Vec::new();
                let mut argument_list = false;
                for pair in pair.into_inner() {
                    if argument_list {
                        arguments.push(Self::parse_tree(pair)?);
                    } else if pair.as_rule() == Rule::Name {
                        name = pair.as_str().into();
                    } else {
                        arguments.push(Self::parse_tree(pair)?);
                        argument_list = true;
                    }
                }
                Ok(Self::Function(name, arguments))
            }
            Rule::Composite => Ok(Self::Composite(Box::new(Self::parse_tree(
                pair.into_inner().next().unwrap(),
            )?))),
            Rule::Name => Ok(Self::Name(
                pair.into_inner()
                    .map(|identifier| identifier.as_str())
                    .join("."),
            )),
            Rule::String | Rule::Boolean | Rule::Number | Rule::Any => {
                Ok(Self::Value(Value::parse(&pair)?))
            }
            _ => {
                unreachable!("{:?}", pair);
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Conjunction(parts) | Self::Disjunction(parts) => {
                parts.iter().map(Filter::len).sum::<usize>()
            }
            Self::Negate(tree) => 1usize + tree.as_ref().len(),
            Self::Restriction(comparable, _, arg) => {
                1usize + comparable.as_ref().len() + arg.as_ref().len()
            }
            Self::Function(tree, arguments) => {
                1usize + tree.len() + arguments.iter().map(Filter::len).sum::<usize>()
            }
            Self::Composite(composite) => 1usize + composite.as_ref().len(),
            _ => 1usize,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn add_conjunction(&mut self, other: Self) {
        match self {
            Self::Conjunction(filters) => {
                filters.push(other);
            }
            _ => {
                *self = Self::Conjunction(vec![self.clone(), other]);
            }
        }
    }

    pub fn add_disjunction(&mut self, other: Self) {
        match self {
            Self::Disjunction(filters) => {
                filters.push(other);
            }
            _ => {
                *self = Self::Disjunction(vec![self.clone(), other]);
            }
        }
    }

    pub fn evaluate<T>(&self, item: &T) -> Option<Value>
    where
        T: SchemaMapped,
    {
        match self {
            Self::Conjunction(parts) => {
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
            Self::Disjunction(parts) => {
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
            Self::Negate(composite) => {
                if let Value::Boolean(value) = composite.evaluate(item)? {
                    Some(Value::Boolean(!value))
                } else {
                    None
                }
            }
            Self::Restriction(comparable, comparator, arg) => {
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
                                    FilterComparator::Equal | FilterComparator::Has => a == b,
                                    FilterComparator::NotEqual => a != b,
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
                                    FilterComparator::Equal | FilterComparator::Has => {
                                        (a - b).abs() < f64::EPSILON
                                    }
                                    FilterComparator::NotEqual => (a - b).abs() > f64::EPSILON,
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
                                FilterComparator::Equal | FilterComparator::Has => {
                                    Some((a == b).into())
                                }
                                FilterComparator::NotEqual => Some((a != b).into()),
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
                                    FilterComparator::Equal | FilterComparator::Has => a == b,
                                    FilterComparator::NotEqual => a != b,
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
                            } else if let Self::Composite(composite) = &**arg {
                                match composite.as_ref() {
                                    Self::Conjunction(parts) => Some(Value::Boolean(
                                        parts.iter().map(|part| part.evaluate(item)).all(|value| {
                                            if let Some(value) = value.as_ref() {
                                                a.contains(value)
                                            } else {
                                                false
                                            }
                                        }),
                                    )),
                                    Self::Disjunction(parts) => Some(Value::Boolean(
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
            Self::Composite(composite) => composite.evaluate(item),
            Self::Value(value) => Some(value.clone()),
            Self::Name(name) => Some(item.get_field(name)),
            Self::Function(_, _) => unimplemented!("evaluate {:?}", self),
        }
    }

    pub fn get_result_value_type(
        &self,
        schema: &Schema,
        schema_functions: Option<&FunctionSchemaMap>,
    ) -> FilterResult<ValueType> {
        match self {
            Self::Conjunction(_)
            | Self::Disjunction(_)
            | Self::Negate(_)
            | Self::Restriction(_, _, _) => Ok(ValueType::Boolean),
            Self::Function(name, _) => Ok(schema_functions
                .and_then(|schema_functions| schema_functions.get(name))
                .ok_or_else(|| FilterError::UnknownFunction(name.clone()))?
                .return_value_type),
            Self::Composite(composite) => composite.get_result_value_type(schema, schema_functions),
            Self::Name(name) => {
                let member_schema = schema
                    .get_member(name)
                    .ok_or_else(|| FilterError::UnknownMember(name.clone()))?;
                if let MemberSchema::Field(field) = member_schema {
                    Ok(field.value_type)
                } else {
                    Err(FilterError::InvalidResultValueType)
                }
            }
            Self::Value(value) => value
                .value_type()
                .ok_or(FilterError::InvalidResultValueType),
        }
    }

    pub fn validate(
        &self,
        schema: &Schema,
        schema_functions: Option<&FunctionSchemaMap>,
    ) -> FilterResult<()> {
        match self {
            Self::Conjunction(parts) | Self::Disjunction(parts) => {
                for part in parts {
                    part.validate(schema, schema_functions)?;
                    let part_value_type = part.get_result_value_type(schema, schema_functions)?;
                    if part_value_type != ValueType::Boolean {
                        return Err(FilterError::InvalidType {
                            actual: part_value_type,
                            expected: ValueType::Boolean,
                        });
                    }
                }
            }
            Self::Negate(tree) => {
                let result_value_type = tree.get_result_value_type(schema, schema_functions)?;
                if result_value_type != ValueType::Boolean {
                    return Err(FilterError::InvalidType {
                        actual: result_value_type,
                        expected: ValueType::Boolean,
                    });
                }
            }
            Self::Restriction(comparable, comparator, argument) => {
                comparable.validate(schema, schema_functions)?;
                let comparable_type = comparable.get_result_value_type(schema, schema_functions)?;
                argument.validate(schema, schema_functions)?;
                let argument_type = argument.get_result_value_type(schema, schema_functions)?;

                if comparator == &FilterComparator::Has {
                    return Err(FilterError::UnsuitableComparator(*comparator));
                    //     if !matches!(**argument, Filter::Conjunction(_) | Filter::Disjunction(_)) {
                    //         if comparable_type != argument_type && argument_type != ValueType::Any {
                    //             return Err(FilterError::InvalidType {
                    //                 actual: argument_type,
                    //                 expected: comparable_type,
                    //             });
                    //         }
                    //     }
                }

                if comparable_type != argument_type {
                    return Err(FilterError::InvalidType {
                        actual: argument_type,
                        expected: comparable_type,
                    });
                }
            }
            Self::Function(name, arguments) => {
                let function = schema_functions
                    .and_then(|schema_functions| schema_functions.get(name))
                    .ok_or_else(|| FilterError::UnknownFunction(name.clone()))?;
                if function.argument_value_types.len() != arguments.len() {
                    return Err(FilterError::FunctionInvalidArgumentCount {
                        name: name.clone(),
                        expected: function.argument_value_types.len(),
                    });
                }
                for (argument, argument_schema) in
                    arguments.iter().zip(&function.argument_value_types)
                {
                    argument.validate(schema, schema_functions)?;
                    let argument_value_type =
                        argument.get_result_value_type(schema, schema_functions)?;
                    if argument_value_type != *argument_schema {
                        return Err(FilterError::InvalidType {
                            actual: argument_value_type,
                            expected: *argument_schema,
                        });
                    }
                }
            }
            Self::Composite(composite) => composite.validate(schema, schema_functions)?,
            Self::Name(name) => {
                if schema.get_member(name).is_none() {
                    return Err(FilterError::UnknownMember(name.clone()));
                }
            }
            Self::Value(_) => {}
        }
        Ok(())
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::Conjunction(Vec::new())
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conjunction(parts) => parts.iter().map(ToString::to_string).join(" AND ").fmt(f),
            Self::Disjunction(parts) => parts.iter().map(ToString::to_string).join(" OR ").fmt(f),
            Self::Negate(tree) => {
                f.write_str("NOT ")?;
                tree.fmt(f)
            }
            Self::Restriction(comparable, comparator, arg) => match comparator {
                FilterComparator::Has => {
                    comparable.fmt(f)?;
                    f.write_char(':')?;
                    arg.fmt(f)
                }
                _ => {
                    write!(f, "{comparable} {comparator} {arg}")
                }
            },
            Self::Function(name, arguments) => {
                write!(
                    f,
                    "{}({})",
                    name,
                    arguments.iter().map(ToString::to_string).join(", ")
                )
            }
            Self::Composite(composite) => {
                f.write_char('(')?;
                composite.fmt(f)?;
                f.write_char(')')
            }
            Self::Name(name) => name.fmt(f),
            Self::Value(value) => value.fmt(f),
        }
    }
}

impl Display for FilterComparator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Less => f.write_str("<"),
            Self::LessOrEqual => f.write_str("<="),
            Self::Greater => f.write_str(">"),
            Self::GreaterOrEqual => f.write_str(">="),
            Self::Equal => f.write_str("="),
            Self::NotEqual => f.write_str("!="),
            Self::Has => f.write_str(":"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::schema::{RequestItem, TaskItem, UserItem};

    use super::*;

    #[test]
    fn validate_schema() {
        let schema = UserItem::get_schema();
        macro_rules! check {
            (@ok $filter:expr) => {
                match check!($filter) {
                    Ok(_) => {}
                    Err(err) => {
                        panic!("Expected Ok, got Err: {:#?}", err);
                    }
                }
            };
            (@error $filter:expr) => {
                match check!($filter) {
                    Ok(_) => {
                        panic!("Expected Err, got Ok");
                    }
                    Err(_) => {}
                }
            };
            ($filter:expr) => {
                Filter::parse($filter).unwrap().validate(&schema, None)
            };
        }

        check!(@ok "42");
        check!(@ok "false");
        check!(@ok r#"id="42""#);

        check!(@error "a");
        check!(@error "a.b");
        check!(@error "f()");
        check!(@error "id=42");
    }

    #[test]
    fn it_works() {
        Filter::parse("  ").unwrap();
        Filter::parse("").unwrap();
        Filter::parse("x").unwrap();
        Filter::parse("42").unwrap();
        Filter::parse("x =  42").unwrap();
        Filter::parse("x=42").unwrap();
        Filter::parse("42").unwrap();
        Filter::parse("3.14").unwrap();
        Filter::parse("NOT a").unwrap();
        Filter::parse("NOT    a").unwrap();
        Filter::parse("a b AND c AND d").unwrap();
        Filter::parse("a < 10 OR a >= 100").unwrap();
        Filter::parse("NOT (a OR b)").unwrap();
        Filter::parse("-30").unwrap();
        Filter::parse("x.b:42").unwrap();
        Filter::parse("experiment.rollout <= cohort(request.user)").unwrap();
        Filter::parse("expr.type_map.type").unwrap();
        Filter::parse("expr.type_map.type").unwrap();
        Filter::parse("regex(m.key, a)").unwrap();
        Filter::parse(r#"math.mem("30mb")"#).unwrap();
        Filter::parse(r#"regex(m.key, "^.*prod.*$")"#).unwrap();
        Filter::parse(r#"(msg.endsWith("world") AND retries < 10)"#).unwrap();
        Filter::parse("x:*").unwrap();

        assert!(Filter::parse("x==42").is_err());
        assert!(Filter::parse("--").is_err());
    }

    #[test]
    fn parse_tree() {
        use Filter::*;
        use FilterComparator::*;

        let tree = Filter::parse("(a.f(42) AND c < 10) OR x AND y:z AND NOT w != true").unwrap();
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
        let src = "(a.f(42) AND c < 10) OR x AND y:z AND NOT w != true";
        let tree = Filter::parse(src).unwrap();
        assert_eq!(tree.to_string(), src);
    }

    #[test]
    fn modify() {
        let mut f = Filter::parse("x=42").unwrap();
        f.add_conjunction(Filter::parse("false").unwrap());
        assert_eq!(f.to_string(), "x = 42 AND false");
        let mut f = Filter::parse("x=42").unwrap();
        f.add_disjunction(Filter::parse("true").unwrap());
        assert_eq!(f.to_string(), "x = 42 OR true");
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
