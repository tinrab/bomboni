use super::{SqlDialect, SqlRenameMap};
use crate::{
    filter::{
        error::{FilterError, FilterResult},
        Filter, FilterComparator,
    },
    schema::{FunctionSchemaMap, Schema, ValueType},
    value::Value,
};

pub struct SqlFilterBuilder<'a> {
    schema: &'a Schema,
    dialect: SqlDialect,
    schema_functions: Option<&'a FunctionSchemaMap>,
    rename_map: Option<&'a SqlRenameMap>,
    argument_offset: usize,
    check_result_boolean: bool,
    arguments: Vec<Value>,
    result: String,
}

impl<'a> SqlFilterBuilder<'a> {
    pub fn new(schema: &'a Schema, dialect: SqlDialect) -> Self {
        Self {
            schema,
            dialect,
            schema_functions: None,
            rename_map: None,
            argument_offset: 0,
            check_result_boolean: false,
            arguments: Vec::new(),
            result: String::new(),
        }
    }

    pub fn set_schema_functions(&mut self, schema_functions: &'a FunctionSchemaMap) -> &mut Self {
        self.schema_functions = Some(schema_functions);
        self
    }

    pub fn set_rename_map(&mut self, rename_map: &'a SqlRenameMap) -> &mut Self {
        self.rename_map = Some(rename_map);
        self
    }

    pub fn set_document_offset(&mut self, offset: usize) -> &mut Self {
        self.argument_offset = offset;
        self
    }

    pub fn check_result_boolean(&mut self) -> &mut Self {
        self.check_result_boolean = true;
        self
    }

    pub fn build(&mut self, filter: &Filter) -> FilterResult<(String, Vec<Value>)> {
        if self.check_result_boolean
            && !matches!(
                filter.get_result_value_type(self.schema, self.schema_functions),
                Some(ValueType::Boolean)
            )
        {
            return Err(FilterError::InvalidResultValueType);
        }

        self.build_tree(filter)?;

        let result = self.result.clone();
        self.result.clear();
        let arguments = self.arguments.clone();
        self.arguments.clear();

        Ok((result, arguments))
    }

    fn build_tree(&mut self, tree: &Filter) -> FilterResult<()> {
        match tree {
            Filter::Conjunction(parts) => {
                for (i, part) in parts.iter().enumerate() {
                    let part_type = part
                        .get_result_value_type(self.schema, self.schema_functions)
                        .ok_or(FilterError::InvalidResultValueType)?;
                    if part_type != ValueType::Boolean {
                        return Err(FilterError::InvalidType {
                            actual: part_type,
                            expected: ValueType::Boolean,
                        });
                    }

                    self.build_tree(part)?;
                    if i < parts.len() - 1 {
                        self.result.push_str(" AND ");
                    }
                }
            }
            Filter::Disjunction(parts) => {
                for (i, part) in parts.iter().enumerate() {
                    let part_type = part
                        .get_result_value_type(self.schema, self.schema_functions)
                        .ok_or(FilterError::InvalidResultValueType)?;
                    if part_type != ValueType::Boolean {
                        return Err(FilterError::InvalidType {
                            actual: part_type,
                            expected: ValueType::Boolean,
                        });
                    }

                    self.build_tree(part)?;
                    if i < parts.len() - 1 {
                        self.result.push_str(" OR ");
                    }
                }
            }
            Filter::Negate(tree) => {
                self.build_negate(tree)?;
            }
            Filter::Restriction(comparable, comparator, arg) => {
                self.build_restriction(comparable, *comparator, arg)?;
            }
            Filter::Function(name, args) => {
                self.build_function(name, args)?;
            }
            Filter::Composite(tree) => {
                self.result.push('(');
                self.build_tree(tree)?;
                self.result.push(')');
            }
            Filter::Name(name) => {
                if self.schema.get_member(name).is_none() {
                    return Err(FilterError::UnknownMember(name.clone()));
                }
                if let Some(rename_map) = self.rename_map {
                    self.build_identifier(&rename_map.rename_member(name), true);
                } else {
                    self.build_identifier(name, true);
                }
            }
            Filter::Value(value) => {
                self.build_argument(value.clone());
            }
        }
        Ok(())
    }

    fn build_negate(&mut self, tree: &Filter) -> FilterResult<()> {
        let tree_type = tree
            .get_result_value_type(self.schema, self.schema_functions)
            .ok_or(FilterError::InvalidResultValueType)?;
        if tree_type != ValueType::Boolean {
            return Err(FilterError::InvalidType {
                actual: tree_type,
                expected: ValueType::Boolean,
            });
        }

        self.result.push_str("NOT (");
        self.build_tree(tree)?;
        self.result.push(')');

        Ok(())
    }

    fn build_restriction(
        &mut self,
        comparable: &Filter,
        comparator: FilterComparator,
        arg: &Filter,
    ) -> FilterResult<()> {
        let comparable_type = comparable
            .get_result_value_type(self.schema, self.schema_functions)
            .ok_or(FilterError::InvalidResultValueType)?;
        let arg_type = arg
            .get_result_value_type(self.schema, self.schema_functions)
            .ok_or(FilterError::InvalidResultValueType)?;

        let mut composite = arg;
        if let Filter::Composite(comp) = composite {
            composite = comp;
        }

        if comparator == FilterComparator::Has {
            match composite {
                Filter::Conjunction(parts) => {
                    self.result.push('(');
                    for (i, part) in parts.iter().enumerate() {
                        self.build_restriction(comparable, FilterComparator::Equal, part)?;
                        if i < parts.len() - 1 {
                            self.result.push_str(" AND ");
                        }
                    }
                    self.result.push(')');
                }
                Filter::Disjunction(parts) => {
                    self.build_tree(comparable)?;
                    self.result.push_str(" = ANY(");

                    let mut values: Vec<Value> = Vec::new();
                    for part in parts {
                        if let Filter::Value(value) = part {
                            let value_type =
                                value.value_type().ok_or(FilterError::ExpectedValue)?;
                            if let Some(first_value) = values.first() {
                                if let Some(first_value_type) = first_value.value_type() {
                                    if first_value_type != value_type {
                                        return Err(FilterError::InvalidType {
                                            actual: value_type,
                                            expected: first_value_type,
                                        });
                                    }
                                } else {
                                    return Err(FilterError::InvalidResultValueType);
                                }
                            }
                            values.push(value.clone());
                        } else {
                            return Err(FilterError::ExpectedValue);
                        }
                    }
                    self.build_argument(Value::Repeated(values));

                    self.result.push(')');
                }
                _ => {
                    if comparable_type != arg_type && arg_type != ValueType::Any {
                        return Err(FilterError::InvalidType {
                            actual: arg_type,
                            expected: comparable_type,
                        });
                    }

                    self.build_tree(comparable)?;

                    match arg_type {
                        ValueType::Integer
                        | ValueType::Float
                        | ValueType::Boolean
                        | ValueType::Timestamp => {
                            self.result.push_str(" = ");
                            self.build_tree(composite)?;
                        }
                        ValueType::String => {
                            self.result.push_str(" LIKE ");
                            if let Filter::Value(value) = composite {
                                self.result.push_str(&format!(
                                    "${}",
                                    self.arguments.len() + 1 + self.argument_offset
                                ));
                                match value {
                                    Value::Integer(_) | Value::Float(_) | Value::Boolean(_) => {
                                        self.arguments.push(format!("%{value}%").into());
                                    }
                                    Value::String(value) => {
                                        self.arguments.push(format!("%{value}%").into());
                                    }
                                    Value::Timestamp(value) => {
                                        self.arguments.push(format!("%{value}%").into());
                                    }
                                    _ => unreachable!(),
                                }
                            } else {
                                unreachable!()
                            }
                        }
                        ValueType::Any => {
                            self.result.push_str(" IS NOT NULL");
                        }
                    }
                }
            }

            return Ok(());
        }

        if comparable_type != arg_type {
            return Err(FilterError::InvalidType {
                actual: arg_type,
                expected: comparable_type,
            });
        }

        self.build_tree(comparable)?;
        match comparator {
            FilterComparator::Less => {
                if arg_type == ValueType::Boolean {
                    return Err(FilterError::IncomparableType(arg_type));
                }
                self.result.push_str(" < ");
            }
            FilterComparator::LessOrEqual => {
                if arg_type == ValueType::Boolean {
                    return Err(FilterError::IncomparableType(arg_type));
                }
                self.result.push_str(" <= ");
            }
            FilterComparator::Greater => {
                if arg_type == ValueType::Boolean {
                    return Err(FilterError::IncomparableType(arg_type));
                }
                self.result.push_str(" > ");
            }
            FilterComparator::GreaterOrEqual => {
                if arg_type == ValueType::Boolean {
                    return Err(FilterError::IncomparableType(arg_type));
                }
                self.result.push_str(" >= ");
            }
            FilterComparator::Equal => {
                self.result.push_str(" = ");
            }
            FilterComparator::NotEqual => {
                self.result.push_str(" != ");
            }
            FilterComparator::Has => unreachable!(),
        }
        self.build_tree(arg)?;

        Ok(())
    }

    fn build_function(&mut self, name: &str, args: &[Filter]) -> FilterResult<()> {
        let function = self
            .schema_functions
            .as_ref()
            .and_then(|schema_functions| schema_functions.get(name))
            .ok_or_else(|| FilterError::UnknownFunction(name.into()))?;

        if let Some(rename_map) = self.rename_map {
            self.build_identifier(&rename_map.rename_function(name), false);
        } else {
            self.build_identifier(name, false);
        }

        self.result.push('(');
        for (i, arg) in args.iter().enumerate() {
            let expected_type = function.argument_value_types[i];
            let arg_type = arg
                .get_result_value_type(self.schema, self.schema_functions)
                .ok_or(FilterError::InvalidResultValueType)?;
            if arg_type != expected_type {
                return Err(FilterError::InvalidType {
                    actual: arg_type,
                    expected: expected_type,
                });
            }

            self.build_tree(arg)?;
            if i < args.len() - 1 {
                self.result.push_str(", ");
            }
        }
        self.result.push(')');

        Ok(())
    }

    fn build_identifier(&mut self, name: &str, escape: bool) {
        match self.dialect {
            SqlDialect::Postgres => {
                if escape {
                    let mut parts = name.split('.');
                    if let Some(first) = parts.next() {
                        self.result.push_str(&format!("\"{first}\""));
                    }
                    for part in parts {
                        self.result.push('.');
                        self.result.push_str(&format!("\"{part}\""));
                    }
                } else {
                    self.result.push_str(name);
                }
            }
        }
    }

    fn build_argument(&mut self, value: Value) {
        self.result.push_str(&format!(
            "${}",
            self.arguments.len() + 1 + self.argument_offset
        ));
        self.arguments.push(value);
    }
}

#[cfg(feature = "testing")]
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{schema::FunctionSchema, testing::schema::RequestItem};
    use bomboni_common::btree_map_into;

    use super::*;

    #[test]
    fn it_works() {
        let schema = RequestItem::get_schema();

        let (sql, args) = SqlFilterBuilder::new(&schema, SqlDialect::Postgres)
            .set_rename_map(&SqlRenameMap {
                members: btree_map_into! {
                    "user" => "u",
                    "task.userId" => "user_id",
                },
                functions: BTreeMap::new(),
            })
            .build(
                &Filter::parse(
                    r#"NOT task.deleted AND task.userId:"2" OR user.age >= 30 OR task.deleted = true"#,
                )
                .unwrap(),
            )
            .unwrap();

        assert_eq!(
            sql,
            r#"NOT ("task"."deleted") AND "task"."user_id" LIKE $1 OR "u"."age" >= $2 OR "task"."deleted" = $3"#
        );
        assert_eq!(args[0], "%2%".into());
        assert_eq!(args[1], 30.into());
        assert_eq!(args[2], true.into());

        let (sql, args) = SqlFilterBuilder::new(&schema, SqlDialect::Postgres)
            .set_schema_functions(&btree_map_into! {
                "regex" => FunctionSchema {
                    argument_value_types: vec![ValueType::String, ValueType::String],
                    return_value_type: ValueType::Boolean,
                }
            })
            .set_rename_map(&SqlRenameMap {
                functions: btree_map_into! {
                  "regex" => "REGEX",
                },
                members: BTreeMap::new(),
            })
            .build(&Filter::parse(r#"regex(user.displayName, "a")"#).unwrap())
            .unwrap();
        assert_eq!(sql, r#"REGEX("user"."displayName", $1)"#);
        assert_eq!(args[0], Value::String("a".into()));

        assert!(SqlFilterBuilder::new(&schema, SqlDialect::Postgres)
            .build(&Filter::parse("logs").unwrap())
            .is_err());
        assert!(SqlFilterBuilder::new(&schema, SqlDialect::Postgres)
            .build(&Filter::parse("user.logs").unwrap())
            .is_err());
        assert!(SqlFilterBuilder::new(&schema, SqlDialect::Postgres)
            .build(&Filter::parse("user.id = 42").unwrap())
            .is_err());
        assert!(SqlFilterBuilder::new(&schema, SqlDialect::Postgres)
            .build(&Filter::parse("task.deleted < false").unwrap())
            .is_err());

        assert!(SqlFilterBuilder::new(&schema, SqlDialect::Postgres)
            .check_result_boolean()
            .build(&Filter::parse("user.id").unwrap())
            .is_err());
    }
}