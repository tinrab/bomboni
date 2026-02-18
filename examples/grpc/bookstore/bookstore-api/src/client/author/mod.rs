//! Author client implementations.
//!
//! This module provides the author client trait and implementations for
//! interacting with author resources in the bookstore service.

use std::{fmt::Debug, sync::Arc};
use tonic::{Response, Status, metadata::MetadataMap};

use crate::v1::{
    Author, CreateAuthorRequest, DeleteAuthorRequest, GetAuthorRequest, ListAuthorsRequest,
    ListAuthorsResponse, UpdateAuthorRequest,
};

/// In-memory author client implementation.
pub mod memory;
/// Remote author client implementation.
pub mod remote;

/// Trait for author client implementations.
///
/// This trait defines the interface for all author client implementations,
/// providing CRUD operations for author resources.
#[async_trait::async_trait]
pub trait AuthorClient: Debug {
    /// Retrieves a specific author by their ID.
    ///
    /// # Arguments
    ///
    /// * `request` - The get author request containing the author ID
    /// * `metadata` - gRPC metadata to include with the request
    ///
    /// # Returns
    ///
    /// A response containing the author details if found
    async fn get_author(
        &self,
        request: GetAuthorRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Author>, Status>;

    /// Lists authors with optional filtering and pagination.
    ///
    /// # Arguments
    ///
    /// * `request` - The list authors request with query parameters
    /// * `metadata` - gRPC metadata to include with the request
    ///
    /// # Returns
    ///
    /// A response containing the list of authors and pagination info
    async fn list_authors(
        &self,
        request: ListAuthorsRequest,
        metadata: MetadataMap,
    ) -> Result<Response<ListAuthorsResponse>, Status>;

    /// Creates a new author.
    ///
    /// # Arguments
    ///
    /// * `request` - The create author request with author details
    /// * `metadata` - gRPC metadata to include with the request
    ///
    /// # Returns
    ///
    /// A response containing the newly created author
    async fn create_author(
        &self,
        request: CreateAuthorRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Author>, Status>;

    /// Updates an existing author.
    ///
    /// # Arguments
    ///
    /// * `request` - The update author request with changes to apply
    /// * `metadata` - gRPC metadata to include with the request
    ///
    /// # Returns
    ///
    /// A response containing the updated author
    async fn update_author(
        &self,
        request: UpdateAuthorRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Author>, Status>;

    /// Deletes an author.
    ///
    /// # Arguments
    ///
    /// * `request` - The delete author request containing the author ID
    /// * `metadata` - gRPC metadata to include with the request
    ///
    /// # Returns
    ///
    /// An empty response indicating successful deletion
    async fn delete_author(
        &self,
        request: DeleteAuthorRequest,
        metadata: MetadataMap,
    ) -> Result<Response<()>, Status>;
}

/// Thread-safe reference-counted author client.
pub type AuthorClientArc = Arc<dyn AuthorClient + Send + Sync>;
