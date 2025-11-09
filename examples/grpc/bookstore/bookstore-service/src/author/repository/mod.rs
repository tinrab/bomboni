use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use bomboni_common::date_time::UtcDateTime;
use bomboni_request::{
    parse::ParsedResource, query::list::ListQuery, schema::SchemaMapped, value::Value,
};
use bookstore_api::{
    model::author::{AuthorId, AuthorModel},
    v1::Author,
};

use crate::error::AppResult;

/// In-memory repository implementation.
pub mod memory;

/// Author record for insertion operations.
#[derive(Debug)]
pub struct AuthorRecordInsert {
    /// Unique author identifier
    pub id: AuthorId,
    /// Creation timestamp
    pub create_time: UtcDateTime,
    /// Author display name
    pub display_name: String,
}

/// Author record with full ownership.
///
/// Represents an author with all fields including timestamps and deletion status.
#[derive(Debug, Clone)]
pub struct AuthorRecordOwned {
    /// Unique author identifier
    pub id: AuthorId,
    /// Creation timestamp
    pub create_time: UtcDateTime,
    /// Last update timestamp
    pub update_time: Option<UtcDateTime>,
    /// Deletion timestamp
    pub delete_time: Option<UtcDateTime>,
    /// Whether the author is deleted
    pub deleted: bool,
    /// Author display name
    pub display_name: String,
}

/// Result of an author list query from the repository.
#[derive(Debug, Clone)]
pub struct AuthorRecordList {
    /// List of author records
    pub items: Vec<AuthorRecordOwned>,
    /// Next item for pagination
    pub next_item: Option<AuthorRecordOwned>,
    /// Total number of matching records
    pub total_size: i64,
}

/// Author record for update operations.
///
/// Contains only the fields that can be updated.
pub struct AuthorRecordUpdate<'a> {
    /// Unique author identifier
    pub id: AuthorId,
    /// Update timestamp
    pub update_time: Option<UtcDateTime>,
    /// Deletion timestamp
    pub delete_time: Option<UtcDateTime>,
    /// Deletion status
    pub deleted: Option<bool>,
    /// New display name
    pub display_name: Option<&'a str>,
}

/// Repository trait for author data persistence.
///
/// Defines the interface for author storage operations.
/// Implementations can use different backends (memory, database, etc.).
#[async_trait]
pub trait AuthorRepository: Debug {
    /// Inserts a new author record.
    ///
    /// # Errors
    ///
    /// Returns an error if the insertion fails.
    async fn insert(&self, record: AuthorRecordInsert) -> AppResult<()>;
    /// Updates an existing author record.
    ///
    /// # Errors
    ///
    /// Returns an error if the update fails.
    async fn update(&self, update: AuthorRecordUpdate<'_>) -> AppResult<bool>;
    /// Selects an author by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the selection fails.
    async fn select(&self, id: &AuthorId) -> AppResult<Option<AuthorRecordOwned>>;
    /// Selects multiple authors by their IDs.
    ///
    /// # Errors
    ///
    /// Returns an error if the selection fails.
    async fn select_multiple(&self, ids: &[AuthorId]) -> AppResult<Vec<AuthorRecordOwned>>;
    /// Selects authors with filtering and pagination.
    ///
    /// # Errors
    ///
    /// Returns an error if the selection fails.
    async fn select_filtered(
        &self,
        query: &ListQuery,
        show_deleted: bool,
    ) -> AppResult<AuthorRecordList>;
}

/// Thread-safe shared reference to an author repository.
pub type AuthorRepositoryArc = Arc<dyn AuthorRepository + Send + Sync>;

impl SchemaMapped for AuthorRecordOwned {
    fn get_field(&self, name: &str) -> Value {
        match name {
            "id" => self.id.0.to_string().into(),
            "display_name" => self.display_name.clone().into(),
            _ => unimplemented!("SchemaMapped for AuthorRecordOwned::{}", name),
        }
    }
}

impl From<AuthorModel> for AuthorRecordOwned {
    fn from(author: AuthorModel) -> Self {
        Self {
            id: author.id,
            create_time: author.resource.create_time.unwrap_or_else(UtcDateTime::now),
            update_time: author.resource.update_time,
            delete_time: author.resource.delete_time,
            deleted: author.resource.deleted,
            display_name: author.display_name,
        }
    }
}

impl From<AuthorRecordOwned> for AuthorModel {
    fn from(record: AuthorRecordOwned) -> Self {
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
        }
    }
}

impl From<AuthorRecordOwned> for Author {
    fn from(record: AuthorRecordOwned) -> Self {
        AuthorModel::from(record).into()
    }
}
