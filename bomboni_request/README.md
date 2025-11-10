# bomboni_request

Utilities for working with API requests.

This crate provides comprehensive utilities for building and processing API requests following Google AIP standards, with support for filtering, ordering, pagination, and SQL generation.

A [bookstore](../examples/grpc/bookstore/README.md) is an example service written using these utilities.

## Features

- **Parse Derive**: Derive macros for automatic request parsing and validation
- **Filter Expressions**: Google AIP-160 compliant filter parsing and evaluation with logical operators (AND, OR, NOT) and comparison operators (=, !=, <, <=, >, >=, :)
- **Query Ordering**: Sort specification with ascending/descending directions and multi-field ordering support
- **List Queries**: Google AIP-132 compliant list method builders with pagination, filtering, and ordering
- **Search Queries**: Fuzzy text search with filtering, ordering, and pagination support
- **Page Tokens**: Secure pagination token implementations (Plain, Base64, AES256, RSA)
- **SQL Generation**: Convert filters and ordering to SQL queries for PostgreSQL and MySQL
- **Schema Validation**: Type-safe validation against defined schemas with field types and constraints
- **WASM Support**: Full WebAssembly compatibility for frontend applications

## Examples

### Parse Derive Macro

The `Parse` derive macro provides powerful options for converting between different data representations:

```rust,ignore
use bomboni_request_derive::Parse;
use bomboni_request::parse::RequestParse;

#[derive(Debug, Clone, PartialEq, Default)]
struct UserProto {
    user_name: String,
    user_age: i32,
    user_email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Parse)]
#[parse(source = UserProto, write, bomboni_request_crate = crate)]
struct User {
    #[parse(source = "user_name")]
    name: String,
    #[parse(source = "user_age")]
    age: i32,
    #[parse(source = "user_email?")]
    email: Option<String>,
}

let proto = UserProto {
    user_name: "Alice".to_string(),
    user_age: 30,
    user_email: Some("alice@example.com".to_string()),
};
let user = User::parse(proto).unwrap();
assert_eq!(user.name, "Alice");
assert_eq!(user.age, 30);
assert_eq!(user.email, Some("alice@example.com".to_string()));
```

See more examples in [src/parse/mod.rs](./src/parse/mod.rs).

### Filter and query

This implements CEL filtering language used in Google APIs.

```rust
use bomboni_request::filter::Filter;
use bomboni_request::testing::schema::{RequestItem, UserItem, TaskItem};

// Parse complex filter expressions
let filter = Filter::parse(r#"
    user.age >= 18
    AND user.id:"4" 
    AND NOT (task.deleted = false)
    AND task.content = user.displayName
    AND task.tags:("a" "b")
"#).unwrap();

// Evaluate against data
let result = filter.evaluate(&RequestItem {
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
}).unwrap();

assert_eq!(result, bomboni_request::value::Value::Boolean(true));
```

Query ordering.

```rust
use bomboni_request::ordering::{Ordering, OrderingDirection};
use bomboni_request::testing::schema::UserItem;

// Parse ordering specification
let ordering = Ordering::parse("displayName desc, age asc").unwrap();
assert_eq!(ordering.to_string(), "displayName desc, age asc");

// Compare items
let a = UserItem {
    id: "1".into(),
    display_name: "Alice".into(),
    age: 30,
};
let b = UserItem {
    id: "2".into(), 
    display_name: "Bob".into(),
    age: 25,
};

let comparison = ordering.evaluate(&a, &b).unwrap();
assert_eq!(comparison, std::cmp::Ordering::Greater); // Alice > Bob by displayName desc
```

You can implement `SchemaMapped` trait on a item and then filter over it.

```rust,ignore
impl SchemaMapped for RequestItem {
    fn get_field(&self, name: &str) -> Value {
        let parts: Vec<_> = name.split('.').collect();
        match *parts.first().unwrap() {
            "user" => self.user.get_field(parts[1]),
            "task" => self.task.get_field(parts[1]),
            _ => unimplemented!("SchemaMapped: SchemaItem::{}", name),
        }
    }
}
```

```rust,ignore
impl SchemaMapped for BookModel {
    fn get_field(&self, name: &str) -> Value {
        match name {
            "id" => self.id.0.to_string().into(),
            "display_name" => self.display_name.clone().into(),
            "author" => self.author_id.0.to_string().into(),
            "isbn" => self.isbn.clone().into(),
            "description" => self.description.clone().into(),
            "price_cents" => self.price_cents.into(),
            "page_count" => self.page_count.into(),
            _ => unimplemented!("SchemaMapped for BookModel::{name}"),
        }
    }
}
```

Schema validation.

```rust
use bomboni_request::schema::{Schema, FieldMemberSchema, ValueType};
use bomboni_request::filter::Filter;
use bomboni_macros::btree_map_into;

// Define schema
let schema = Schema {
    members: btree_map_into! {
        "id" => FieldMemberSchema::new_ordered(ValueType::String),
        "age" => FieldMemberSchema::new_ordered(ValueType::Integer),
        "name" => FieldMemberSchema::new(ValueType::String),
        "tags" => FieldMemberSchema::new_repeated(ValueType::String),
    },
};

// Validate filter against schema
let filter = Filter::parse("age >= 18 AND name = \"John\"").unwrap();
filter.validate(&schema, None).unwrap(); // OK

let invalid_filter = Filter::parse("invalid_field = \"test\"").unwrap();
assert!(invalid_filter.validate(&schema, None).is_err()); // Error: unknown field
```

### List and Search Queries

```rust
use bomboni_request::query::{
    list::{ListQueryBuilder, PlainListQueryBuilder, ListQueryConfig},
    search::{SearchQueryBuilder, PlainSearchQueryBuilder, SearchQueryConfig},
    page_token::plain::PlainPageTokenBuilder,
};
use bomboni_request::testing::schema::UserItem;
use bomboni_request::ordering::{OrderingTerm, OrderingDirection};

// Create list query builder
let list_builder = PlainListQueryBuilder::new(
    UserItem::get_schema(),
    std::collections::BTreeMap::new(),
    ListQueryConfig {
        max_page_size: Some(100),
        default_page_size: 20,
        primary_ordering_term: Some(OrderingTerm {
            name: "id".into(),
            direction: OrderingDirection::Ascending,
        }),
        max_filter_length: Some(1000),
        max_ordering_length: Some(100),
    },
    PlainPageTokenBuilder {},
);

// Build list query
let list_query = list_builder.build(
    Some(50),                                    // page_size
    None,                                        // page_token
    Some(r#"displayName = "John""#),             // filter
    Some("age desc")                             // ordering
).unwrap();

assert_eq!(list_query.page_size, 50);
assert_eq!(list_query.filter.to_string(), r#"displayName = "John""#);
assert_eq!(list_query.ordering.to_string(), "id asc, age desc");

// Create search query builder
let search_builder = PlainSearchQueryBuilder::new(
    UserItem::get_schema(),
    std::collections::BTreeMap::new(),
    SearchQueryConfig {
        max_query_length: Some(100),
        max_page_size: Some(20),  // Clamp to 20
        default_page_size: 20,
        primary_ordering_term: Some(OrderingTerm {
            name: "id".into(),
            direction: OrderingDirection::Descending,
        }),
        max_filter_length: Some(1000),
        max_ordering_length: Some(100),
    },
    PlainPageTokenBuilder {},
);

// Build search query
let search_query = search_builder.build(
    "john doe",                                  // search query text
    Some(25),                                    // page_size
    None,                                        // page_token
    Some(r#"age >= 18 AND displayName = "John""#), // filter
    Some("age desc, displayName asc")             // ordering
).unwrap();

assert_eq!(search_query.query, "john doe");
assert_eq!(search_query.page_size, 20); // Clamped to max_page_size
assert_eq!(search_query.filter.to_string(), r#"age >= 18 AND displayName = "John""#);
assert_eq!(search_query.ordering.to_string(), "id desc, age desc, displayName asc");
```

The `Parse` derive macro can automatically handle list and search query parsing:

```rust,ignore
use bomboni_request_derive::Parse;
use bomboni_request::parse::RequestParse;
use bomboni_request::query::list::{ListQuery, ListQueryBuilder, ListQueryConfig};
use bomboni_request::query::search::{SearchQuery, SearchQueryBuilder, SearchQueryConfig};
use bomboni_request::ordering::{OrderingTerm, OrderingDirection};
use bomboni_request::query::page_token::plain::PlainPageTokenBuilder;
use bomboni_request::testing::schema::UserItem;
use std::collections::BTreeMap;

// Define request structures
#[derive(Debug, Clone, PartialEq, Default)]
struct ListUsersRequest {
    page_size: Option<u32>,
    page_token: Option<String>,
    filter: Option<String>,
    order_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct SearchUsersRequest {
    query: String,
    page_size: Option<u32>,
    page_token: Option<String>,
    filter: Option<String>,
    order_by: Option<String>,
}

// Apply Parse derive macro
#[derive(Debug, Clone, PartialEq, Parse)]
#[parse(source = ListUsersRequest, write, bomboni_request_crate = crate)]
struct ParsedListUsersRequest {
    #[parse(list_query)]
    query: ListQuery,
}

#[derive(Debug, Clone, PartialEq, Parse)]
#[parse(source = SearchUsersRequest, write, bomboni_request_crate = crate)]
struct ParsedSearchUsersRequest {
    #[parse(search_query)]
    query: SearchQuery,
}

// Create query builders (shared)
let list_builder = ListQueryBuilder::new(
    UserItem::get_schema(),
    BTreeMap::new(),
    ListQueryConfig {
        max_page_size: Some(100),
        default_page_size: 20,
        primary_ordering_term: Some(OrderingTerm {
            name: "id".into(),
            direction: OrderingDirection::Ascending,
        }),
        max_filter_length: Some(1000),
        max_ordering_length: Some(100),
    },
    PlainPageTokenBuilder {},
);

let search_builder = SearchQueryBuilder::new(
    UserItem::get_schema(),
    BTreeMap::new(),
    SearchQueryConfig {
        max_query_length: Some(100),
        max_page_size: Some(20),
        default_page_size: 10,
        primary_ordering_term: Some(OrderingTerm {
            name: "id".into(),
            direction: OrderingDirection::Descending,
        }),
        max_filter_length: Some(1000),
        max_ordering_length: Some(100),
    },
    PlainPageTokenBuilder {},
);

// Parse list request
let list_request = ListUsersRequest {
    page_size: Some(50),
    page_token: None,
    filter: Some(r#"displayName = "John""#),
    order_by: Some("age desc"),
};

let parsed_list = ParsedListUsersRequest::parse_list_query(list_request, &list_builder).unwrap();
assert_eq!(parsed_list.query.page_size, 50);
assert_eq!(parsed_list.query.filter.to_string(), r#"displayName = "John""#);
assert_eq!(parsed_list.query.ordering.to_string(), "id asc, age desc");

// Parse search request
let search_request = SearchUsersRequest {
    query: "john doe".to_string(),
    page_size: Some(25),
    page_token: None,
    filter: Some(r#"age >= 18 AND displayName = "John""#),
    order_by: Some("age desc, displayName asc"),
};

let parsed_search = ParsedSearchUsersRequest::parse_search_query(search_request, &search_builder).unwrap();
assert_eq!(parsed_search.query.query, "john doe");
assert_eq!(parsed_search.query.page_size, 20); // Clamped to max_page_size
assert_eq!(parsed_search.query.filter.to_string(), r#"age >= 18 AND displayName = "John""#);
assert_eq!(parsed_search.query.ordering.to_string(), "id desc, age desc, displayName asc");
```

### SQL Generation

```rust
use bomboni_request::sql::{SqlFilterBuilder, SqlDialect, SqlRenameMap};
use bomboni_request::filter::Filter;
use bomboni_request::testing::schema::RequestItem;
use bomboni_macros::btree_map_into;

let schema = RequestItem::get_schema();
let filter = Filter::parse(r#"NOT task.deleted AND user.age >= 30"#).unwrap();

// Generate PostgreSQL SQL
let (sql, args) = SqlFilterBuilder::new(SqlDialect::Postgres, &schema)
    .set_rename_map(&SqlRenameMap {
        members: btree_map_into! {
            "user" => "u",
            "task.userId" => "user_id",
        },
        functions: std::collections::BTreeMap::new(),
    })
    .build(&filter)
    .unwrap();

assert_eq!(sql, r#"NOT ("task"."deleted") AND "u"."age" >= $1"#);
assert_eq!(args[0], bomboni_request::value::Value::Integer(30));
```

### Resource Name Parsing

Parse structured resource names using the `parse_resource_name` macro:

```rust
use bomboni_request::derive::parse_resource_name;

// Define resource name pattern
let parse_user_resource = parse_resource_name!({
    "users": String,
    "projects": Option<String>,
});

// Parse resource names
let (user_id, project_id) = parse_user_resource("users/alice/projects/awesome").unwrap();
assert_eq!(user_id, "alice");
assert_eq!(project_id, Some("awesome".to_string()));

let (user_id, project_id) = parse_user_resource("users/bob").unwrap();
assert_eq!(user_id, "bob");
assert_eq!(project_id, None);
```

## Cargo Features

- `derive`: Enable derive macros for request parsing
- `testing`: Enable testing utilities and schemas
- `tonic`: Enable gRPC integration with tonic
- `wasm`: Enable WebAssembly support
- `postgres`: Enable PostgreSQL type conversions
- `mysql`: Enable MySQL type conversions
