use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use bomboni_common::date_time::UtcDateTime;
use bomboni_request::{
    parse::ParsedResource, query::list::ListQuery, schema::SchemaMapped, value::Value,
};
use bookstore_api::{
    model::{
        author::AuthorId,
        book::{BookId, BookModel},
    },
    v1::Book,
};

use crate::error::AppResult;

pub mod memory;

#[derive(Debug)]
pub struct BookRecordInsert {
    pub id: BookId,
    pub create_time: UtcDateTime,
    pub display_name: String,
    pub author_id: AuthorId,
    pub isbn: String,
    pub description: String,
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
    pub author_id: AuthorId,
    pub isbn: String,
    pub description: String,
    pub price_cents: i64,
    pub page_count: i32,
}

#[derive(Debug, Clone)]
pub struct BookRecordList {
    pub items: Vec<BookRecordOwned>,
    pub next_item: Option<BookRecordOwned>,
    pub total_size: i64,
}

pub struct BookRecordUpdate<'a> {
    pub id: &'a BookId,
    pub update_time: Option<UtcDateTime>,
    pub delete_time: Option<UtcDateTime>,
    pub deleted: Option<bool>,
    pub display_name: Option<&'a str>,
    pub author_id: Option<AuthorId>,
    pub isbn: Option<&'a str>,
    pub description: Option<&'a str>,
    pub price_cents: Option<i64>,
    pub page_count: Option<i32>,
}

#[async_trait]
pub trait BookRepository: Debug {
    async fn insert(&self, record: BookRecordInsert) -> AppResult<()>;
    async fn update(&self, update: BookRecordUpdate<'_>) -> AppResult<bool>;
    async fn select(&self, id: &BookId) -> AppResult<Option<BookRecordOwned>>;
    async fn select_multiple(&self, ids: &[BookId]) -> AppResult<Vec<BookRecordOwned>>;
    async fn select_filtered(
        &self,
        query: &ListQuery,
        show_deleted: bool,
    ) -> AppResult<BookRecordList>;
}

pub type BookRepositoryArc = Arc<dyn BookRepository + Send + Sync>;

impl SchemaMapped for BookRecordOwned {
    fn get_field(&self, name: &str) -> Value {
        match name {
            "id" => self.id.0.to_string().into(),
            "display_name" => self.display_name.clone().into(),
            "author" => self.author_id.to_string().into(),
            "isbn" => self.isbn.clone().into(),
            "description" => self.description.clone().into(),
            "price_cents" => self.price_cents.into(),
            "page_count" => self.page_count.into(),
            _ => unimplemented!("SchemaMapped for BookRecordOwned::{}", name),
        }
    }
}

impl From<BookModel> for BookRecordOwned {
    fn from(book: BookModel) -> Self {
        BookRecordOwned {
            id: book.id,
            create_time: book
                .resource
                .create_time
                .unwrap_or_else(|| UtcDateTime::now()),
            update_time: book.resource.update_time,
            delete_time: book.resource.delete_time,
            deleted: book.resource.deleted,
            display_name: book.display_name,
            author_id: book.author_id,
            isbn: book.isbn,
            description: book.description,
            price_cents: book.price_cents,
            page_count: book.page_count,
        }
    }
}

impl From<BookRecordOwned> for BookModel {
    fn from(record: BookRecordOwned) -> Self {
        BookModel {
            resource: ParsedResource {
                name: record.id.to_name(),
                create_time: Some(record.create_time.into()),
                update_time: record.update_time.map(Into::into),
                delete_time: record.delete_time.map(Into::into),
                deleted: record.deleted,
                etag: None,
                revision_id: None,
                revision_create_time: None,
            },
            id: record.id,
            display_name: record.display_name,
            author_id: record.author_id,
            isbn: record.isbn,
            description: record.description,
            price_cents: record.price_cents,
            page_count: record.page_count,
        }
    }
}

impl From<BookRecordOwned> for Book {
    fn from(record: BookRecordOwned) -> Self {
        BookModel::from(record).into()
    }
}
