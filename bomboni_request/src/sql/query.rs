use crate::filter::Filter;
use crate::ordering::Ordering;
use crate::query::{error::QueryResult, list::ListQuery, search::SearchQuery};
use crate::schema::{FunctionSchemaMap, Schema};
use crate::sql::{
    SqlArgumentStyle, SqlDialect, SqlFilterBuilder, SqlOrderingBuilder, SqlRenameMap,
    utility::get_argument_parameter,
};
use crate::value::Value;

/// Builder for query SQL statements.
#[derive(Debug, Clone)]
pub struct QuerySqlBuilder {
    dialect: SqlDialect,
    argument_style: SqlArgumentStyle,
    schema: Schema,
    schema_functions: FunctionSchemaMap,
    rename_map: SqlRenameMap,
    case_insensitive_like: bool,
    query_next_page: bool,
}

/// Query SQL statement.
#[derive(Debug, Clone)]
pub struct QuerySqlStatement {
    /// WHERE clause.
    pub where_clause: Option<String>,
    /// Query arguments.
    pub arguments: Vec<Value>,
    /// Paged WHERE clause.
    pub paged_where_clause: Option<String>,
    /// Paged LIMIT clause.
    pub paged_limit_clause: String,
    /// Paged arguments.
    pub paged_arguments: Vec<Value>,
    /// ORDER BY clause.
    pub order_by_clause: Option<String>,
}

impl QuerySqlBuilder {
    /// Creates a new query SQL builder.
    pub fn new(dialect: SqlDialect, schema: Schema) -> Self {
        Self {
            dialect,
            argument_style: SqlArgumentStyle::Indexed { prefix: "$".into() },
            schema,
            schema_functions: FunctionSchemaMap::new(),
            rename_map: SqlRenameMap::default(),
            case_insensitive_like: false,
            query_next_page: false,
        }
    }

    /// Sets the schema functions.
    pub fn set_schema_functions(&mut self, schema_functions: FunctionSchemaMap) -> &mut Self {
        self.schema_functions = schema_functions;
        self
    }

    /// Sets the rename map.
    pub fn set_rename_map(&mut self, rename_map: SqlRenameMap) -> &mut Self {
        self.rename_map = rename_map;
        self
    }

    /// Enables case insensitive like.
    pub const fn case_insensitive_like(&mut self) -> &mut Self {
        self.case_insensitive_like = true;
        self
    }

    /// Enables query next page.
    pub const fn query_next_page(&mut self) -> &mut Self {
        self.query_next_page = true;
        self
    }

    /// Sets the argument style.
    pub fn set_argument_style(&mut self, argument_style: SqlArgumentStyle) -> &mut Self {
        self.argument_style = argument_style;
        self
    }

    /// Builds a list query SQL statement.
    ///
    /// # Errors
    ///
    /// Returns an error if query building fails.
    pub fn build_list(&self, query: &ListQuery) -> QueryResult<QuerySqlStatement> {
        self.build(
            query.page_size,
            query
                .page_token
                .as_ref()
                .map(|page_token| &page_token.filter),
            &query.filter,
            &query.ordering,
        )
    }

    /// Builds a search query SQL statement.
    ///
    /// # Errors
    ///
    /// Returns an error if query building fails.
    pub fn build_search(&self, query: &SearchQuery) -> QueryResult<QuerySqlStatement> {
        self.build(
            query.page_size,
            query
                .page_token
                .as_ref()
                .map(|page_token| &page_token.filter),
            &query.filter,
            &query.ordering,
        )
    }

    /// Builds a query SQL statement.
    ///
    /// # Errors
    ///
    /// Returns an error if query building fails.
    pub fn build(
        &self,
        page_size: i32,
        page_token: Option<&Filter>,
        filter: &Filter,
        ordering: &Ordering,
    ) -> QueryResult<QuerySqlStatement> {
        let (where_clause, arguments) = if filter.is_empty() {
            (None, Vec::new())
        } else {
            let mut filter_builder = SqlFilterBuilder::new(self.dialect, &self.schema);
            filter_builder
                .set_schema_functions(&self.schema_functions)
                .set_rename_map(&self.rename_map)
                .set_argument_style(self.argument_style.clone());
            if self.case_insensitive_like {
                filter_builder.case_insensitive_like();
            }
            let (where_clause, arguments) = filter_builder.build(filter)?;
            (Some(where_clause), arguments)
        };

        let (paged_where_clause, mut paged_arguments) =
            if let Some(page_token) = page_token.filter(|page_token| !page_token.is_empty()) {
                let filter = if filter.is_empty() {
                    page_token.clone()
                } else {
                    Filter::Conjunction(vec![filter.clone(), page_token.clone()])
                };
                let mut filter_builder = SqlFilterBuilder::new(self.dialect, &self.schema);
                filter_builder
                    .set_schema_functions(&self.schema_functions)
                    .set_rename_map(&self.rename_map)
                    .set_argument_style(self.argument_style.clone());
                if self.case_insensitive_like {
                    filter_builder.case_insensitive_like();
                }
                let (paged_where_clause, paged_arguments) = filter_builder.build(&filter)?;
                (Some(paged_where_clause), paged_arguments)
            } else {
                (where_clause.clone(), arguments.clone())
            };

        let paged_limit_clause = format!(
            "LIMIT {}",
            get_argument_parameter(&self.argument_style, paged_arguments.len() + 1)
        );
        paged_arguments.push(if self.query_next_page {
            // One more than page_size to determine if there are more results
            (page_size + 1).into()
        } else {
            page_size.into()
        });

        let order_by_clause = if ordering.is_empty() {
            None
        } else {
            Some(
                SqlOrderingBuilder::new(self.dialect, &self.schema)
                    .set_rename_map(&self.rename_map)
                    .build(ordering)?,
            )
        };

        Ok(QuerySqlStatement {
            where_clause,
            arguments,
            paged_where_clause,
            paged_limit_clause,
            paged_arguments,
            order_by_clause,
        })
    }
}

#[cfg(feature = "postgres")]
const _: () = {
    use postgres_types::ToSql;

    impl QuerySqlStatement {
        /// Gets the SQL parameters for the query.
        pub fn get_sql_params(&self) -> Vec<&(dyn ToSql + Sync)> {
            self.arguments.iter().collect()
        }

        /// Gets the SQL parameters for the paged query.
        pub fn get_paged_sql_params(&self) -> Vec<&(dyn ToSql + Sync)> {
            self.paged_arguments.iter().collect()
        }
    }
};

#[cfg(test)]
mod tests {
    use crate::{
        ordering::Ordering, query::page_token::FilterPageToken, testing::schema::RequestItem,
    };

    use super::*;

    #[test]
    fn it_works() {
        let mut builder = QuerySqlBuilder::new(SqlDialect::Postgres, RequestItem::get_schema());
        builder.query_next_page();

        let s = builder
            .build_list(&ListQuery {
                filter: Filter::parse(r#"NOT task.deleted AND user.id = "42""#).unwrap(),
                ordering: Ordering::parse("task.id desc").unwrap(),
                page_size: 5,
                page_token: None,
            })
            .unwrap();
        assert_eq!(
            &s.where_clause.unwrap(),
            r#"NOT ("task"."deleted") AND "user"."id" = $1"#,
        );
        assert_eq!(s.arguments.len(), 1);
        assert_eq!(
            &s.paged_where_clause.unwrap(),
            r#"NOT ("task"."deleted") AND "user"."id" = $1"#,
        );
        assert_eq!(s.paged_limit_clause, "LIMIT $2");
        assert_eq!(s.paged_arguments.len(), 2);
        assert_eq!(s.order_by_clause.unwrap(), r#""task"."id" DESC"#);

        let s = builder
            .build_list(&ListQuery {
                filter: Filter::parse(r#"NOT task.deleted AND user.id = "42""#).unwrap(),
                ordering: Ordering::parse("task.id desc").unwrap(),
                page_size: 5,
                page_token: Some(FilterPageToken::new(
                    Filter::parse(r#"task.id < "10""#).unwrap(),
                )),
            })
            .unwrap();
        assert_eq!(
            &s.where_clause.unwrap(),
            r#"NOT ("task"."deleted") AND "user"."id" = $1"#,
        );
        assert_eq!(s.arguments.len(), 1);
        assert_eq!(
            &s.paged_where_clause.unwrap(),
            r#"NOT ("task"."deleted") AND "user"."id" = $1 AND "task"."id" < $2"#,
        );
        assert_eq!(s.paged_limit_clause, "LIMIT $3");
        assert_eq!(s.paged_arguments.len(), 3);
        assert_eq!(s.order_by_clause.unwrap(), r#""task"."id" DESC"#);
    }
}
