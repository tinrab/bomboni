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

/// In-memory repository implementation.
pub mod memory;

/// Book record for insertion operations.
///
/// Contains all required fields for creating a new book record.
/// Used when inserting new books into the repository.
#[derive(Debug)]
pub struct BookRecordInsert {
    /// Unique identifier for the book
    pub id: BookId,
    /// Timestamp when the book was created
    pub create_time: UtcDateTime,
    /// Display name of the book
    pub display_name: String,
    /// Unique identifier for the author
    pub author_id: AuthorId,
    /// International Standard Book Number
    pub isbn: String,
    /// Description of the book
    pub description: String,
    /// Price in cents
    pub price_cents: i64,
    /// Number of pages in the book
    pub page_count: i32,
}

/// Complete book record with ownership.
///
/// Contains all book fields including timestamps and deletion status.
/// Represents a fully owned book record stored in the repository.
#[derive(Debug, Clone)]
pub struct BookRecordOwned {
    /// Unique identifier for the book
    pub id: BookId,
    /// Timestamp when the book was created
    pub create_time: UtcDateTime,
    /// Timestamp when the book was last updated
    pub update_time: Option<UtcDateTime>,
    /// Timestamp when the book was soft deleted
    pub delete_time: Option<UtcDateTime>,
    /// Whether the book has been soft deleted
    pub deleted: bool,
    /// Display name of the book
    pub display_name: String,
    /// Unique identifier for the author
    pub author_id: AuthorId,
    /// International Standard Book Number
    pub isbn: String,
    /// Description of the book
    pub description: String,
    /// Price in cents
    pub price_cents: i64,
    /// Number of pages in the book
    pub page_count: i32,
}

/// Paginated list of book records.
///
/// Contains a page of book records along with pagination metadata
/// for retrieving subsequent pages.
#[derive(Debug, Clone)]
pub struct BookRecordList {
    /// List of book records in the current page
    pub items: Vec<BookRecordOwned>,
    /// First item of the next page, if any
    pub next_item: Option<BookRecordOwned>,
    /// Total number of records matching the query
    pub total_size: i64,
}

/// Book record for update operations.
///
/// Contains optional fields for updating an existing book record.
/// Only provided fields will be updated during the operation.
pub struct BookRecordUpdate<'a> {
    /// Unique identifier for the book to update
    pub id: &'a BookId,
    /// New update timestamp
    pub update_time: Option<UtcDateTime>,
    /// New delete timestamp
    pub delete_time: Option<UtcDateTime>,
    /// New deletion status
    pub deleted: Option<bool>,
    /// New display name
    pub display_name: Option<&'a str>,
    /// New author identifier
    pub author_id: Option<AuthorId>,
    /// New ISBN
    pub isbn: Option<&'a str>,
    /// New description
    pub description: Option<&'a str>,
    /// New price in cents
    pub price_cents: Option<i64>,
    /// New page count
    pub page_count: Option<i32>,
}

/// Repository trait for book data operations.
///
/// Defines the interface for book persistence operations including
/// CRUD operations and querying with filtering and pagination.
#[async_trait]
pub trait BookRepository: Debug {
    /// Inserts a new book record.
    ///
    /// # Errors
    ///
    /// Returns an error if the insertion fails.
    async fn insert(&self, record: BookRecordInsert) -> AppResult<()>;

    /// Updates an existing book record.
    ///
    /// # Errors
    ///
    /// Returns an error if the update fails.
    ///
    /// # Returns
    ///
    /// Returns `true` if a record was updated, `false` if not found.
    async fn update(&self, update: BookRecordUpdate<'_>) -> AppResult<bool>;

    /// Selects a book record by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the selection fails.
    ///
    /// # Returns
    ///
    /// Returns the book record if found, `None` otherwise.
    async fn select(&self, id: &BookId) -> AppResult<Option<BookRecordOwned>>;

    /// Selects multiple book records by their IDs.
    ///
    /// # Errors
    ///
    /// Returns an error if the selection fails.
    async fn select_multiple(&self, ids: &[BookId]) -> AppResult<Vec<BookRecordOwned>>;

    /// Selects book records with filtering and pagination.
    ///
    /// # Errors
    ///
    /// Returns an error if the selection fails.
    async fn select_filtered(
        &self,
        query: &ListQuery,
        show_deleted: bool,
    ) -> AppResult<BookRecordList>;
}

/// Thread-safe shared reference to a book repository.
///
/// Type alias for an Arc-wrapped book repository that can be shared
/// across threads and used in async contexts.
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
        Self {
            id: book.id,
            create_time: book.resource.create_time.unwrap_or_else(UtcDateTime::now),
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
        Self {
            resource: ParsedResource {
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
