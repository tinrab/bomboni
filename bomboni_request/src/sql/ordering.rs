use crate::{
    ordering::{
        error::{OrderingError, OrderingResult},
        Ordering, OrderingDirection,
    },
    schema::Schema,
};

use super::{utility::get_identifier, SqlDialect, SqlRenameMap};

pub struct SqlOrderingBuilder<'a> {
    dialect: SqlDialect,
    schema: &'a Schema,
    rename_map: Option<&'a SqlRenameMap>,
    result: String,
}

impl<'a> SqlOrderingBuilder<'a> {
    pub fn new(dialect: SqlDialect, schema: &'a Schema) -> Self {
        Self {
            dialect,
            schema,
            rename_map: None,
            result: String::new(),
        }
    }

    pub fn set_rename_map(&mut self, rename_map: &'a SqlRenameMap) -> &mut Self {
        self.rename_map = Some(rename_map);
        self
    }

    pub fn build(&mut self, ordering: &Ordering) -> OrderingResult<String> {
        for (i, term) in ordering.iter().enumerate() {
            if self.schema.get_member(&term.name).is_none() {
                return Err(OrderingError::UnknownMember(term.name.clone()));
            }

            if let Some(rename_map) = self.rename_map {
                self.result.push_str(&get_identifier(
                    self.dialect,
                    &rename_map.rename_function(&term.name),
                    true,
                ));
            } else {
                self.result
                    .push_str(&get_identifier(self.dialect, &term.name, true));
            }

            self.result.push(' ');

            match term.direction {
                OrderingDirection::Ascending => {
                    self.result.push_str("ASC");
                }
                OrderingDirection::Descending => {
                    self.result.push_str("DESC");
                }
            }

            if i < ordering.len() - 1 {
                self.result.push_str(", ");
            }
        }

        let result = self.result.clone();
        self.result.clear();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::schema::RequestItem;

    use super::*;

    #[test]
    fn it_works() {
        let schema = RequestItem::get_schema();

        assert_eq!(
            SqlOrderingBuilder::new(SqlDialect::Postgres, &schema)
                .build(&Ordering::parse("user.age desc, user.displayName").unwrap())
                .unwrap(),
            r#""user"."age" DESC, "user"."displayName" ASC"#
        );
    }
}
