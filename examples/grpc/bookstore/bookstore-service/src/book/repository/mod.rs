pub mod memory;

use async_trait::async_trait;
use bomboni_common::date_time::UtcDateTime;
use bomboni_request::{query::list::ListQuery, schema::SchemaMapped, value::Value as FilterValue};
use bookstore_api::model::book::{BookId, BookModel};
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct BookRecordInsert<'a> {
    pub id: BookId,
    pub create_time: UtcDateTime,
    pub display_name: &'a str,
    pub author: &'a str,
    pub isbn: &'a str,
    pub description: &'a str,
    pub price_cents: i64,
    pub page_count: i32,
}

#[derive(Debug, Clone)]
pub struct BookRecordOwned {
    pub id: BookId,
    pub create_time: UtcDateTime,
    pub update_time: Option<UtcDateTime>,
    pub delete_time: Option<UtcDateTime>,
    pub deleted: bool,
    pub display_name: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub price_cents: i64,
    pub page_count: i32,
}

#[derive(Debug, Clone)]
pub struct BookRecordList {
    pub items: Vec<BookRecordOwned>,
    pub next_item: Option<BookRecordOwned>,
}

#[derive(Debug)]
pub struct BookRecordUpdate<'a> {
    pub id: BookId,
    pub update_time: Option<UtcDateTime>,
    pub delete_time: Option<UtcDateTime>,
    pub deleted: Option<bool>,
    pub display_name: Option<&'a str>,
    pub author: Option<&'a str>,
    pub isbn: Option<&'a str>,
    pub description: Option<&'a str>,
    pub price_cents: Option<i64>,
    pub page_count: Option<i32>,
}

#[async_trait]
pub trait BookRepository: Debug {
    async fn insert(
        &self,
        record: BookRecordInsert<'_>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn update(
        &self,
        update: BookRecordUpdate<'_>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn select(
        &self,
        id: BookId,
    ) -> Result<Option<BookRecordOwned>, Box<dyn std::error::Error + Send + Sync>>;
    async fn select_filtered(
        &self,
        query: &ListQuery,
        show_deleted: bool,
    ) -> Result<BookRecordList, Box<dyn std::error::Error + Send + Sync>>;
}

pub type BookRepositoryArc = Arc<dyn BookRepository + Send + Sync>;

impl SchemaMapped for BookRecordOwned {
    fn get_field(&self, name: &str) -> FilterValue {
        match name {
            "id" => self.id.0.to_string().into(),
            "display_name" => self.display_name.clone().into(),
            "author" => self.author.clone().into(),
            "isbn" => self.isbn.clone().into(),
            "description" => self.description.clone().into(),
            "price_cents" => self.price_cents.into(),
            "page_count" => self.page_count.into(),
            _ => unimplemented!("SchemaMapped for BookRecordOwned::{}", name),
        }
    }
}

impl From<BookModel> for BookRecordOwned {
    fn from(model: BookModel) -> Self {
        BookRecordOwned {
            id: model.id,
            create_time: model.resource.create_time.unwrap(),
            update_time: model.resource.update_time,
            delete_time: model.resource.delete_time,
            deleted: model.resource.deleted,
            display_name: model.display_name,
            author: model.author_id.0.to_string(),
            isbn: model.isbn,
            description: model.description,
            price_cents: model.price_cents,
            page_count: model.page_count,
        }
    }
}

impl From<BookRecordOwned> for BookModel {
    fn from(record: BookRecordOwned) -> Self {
        BookModel {
            resource: bomboni_request::parse::ParsedResource {
                name: record.id.to_name(),
                create_time: Some(record.create_time),
                update_time: record.update_time,
                delete_time: record.delete_time,
                deleted: record.deleted,
                etag: None,
                revision_id: None,
                revision_create_time: None,
            },
            id: record.id,
            display_name: record.display_name,
            author_id: bookstore_api::model::author::AuthorId(
                bomboni_common::id::Id::from_str(&record.author).unwrap_or_default(),
            ),
            isbn: record.isbn,
            description: record.description,
            price_cents: record.price_cents,
            page_count: record.page_count,
        }
    }
}
