//! Book client implementations.
//!
//! This module provides the book client trait and implementations for
//! interacting with book resources in the bookstore service.

use std::fmt::Debug;
use std::sync::Arc;
use tonic::metadata::MetadataMap;
use tonic::{Response, Status};

use crate::v1::Book;
use crate::v1::{
    CreateBookRequest, DeleteBookRequest, GetBookRequest, ListBooksRequest, ListBooksResponse,
    UpdateBookRequest,
};

/// In-memory book client implementation.
pub mod memory;
/// Remote book client implementation.
pub mod remote;

/// Trait for book client implementations.
///
/// This trait defines the interface for all book client implementations,
/// providing CRUD operations for book resources.
#[async_trait::async_trait]
pub trait BookClient: Debug {
    /// Retrieves a specific book by its ID.
    ///
    /// # Arguments
    ///
    /// * `request` - The get book request containing the book ID
    /// * `metadata` - gRPC metadata to include with the request
    ///
    /// # Returns
    ///
    /// A response containing the book details if found
    async fn get_book(
        &self,
        request: GetBookRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Book>, Status>;

    /// Lists books with optional filtering and pagination.
    ///
    /// # Arguments
    ///
    /// * `request` - The list books request with query parameters
    /// * `metadata` - gRPC metadata to include with the request
    ///
    /// # Returns
    ///
    /// A response containing the list of books and pagination info
    async fn list_books(
        &self,
        request: ListBooksRequest,
        metadata: MetadataMap,
    ) -> Result<Response<ListBooksResponse>, Status>;

    /// Creates a new book.
    ///
    /// # Arguments
    ///
    /// * `request` - The create book request with book details
    /// * `metadata` - gRPC metadata to include with the request
    ///
    /// # Returns
    ///
    /// A response containing the newly created book
    async fn create_book(
        &self,
        request: CreateBookRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Book>, Status>;

    /// Updates an existing book.
    ///
    /// # Arguments
    ///
    /// * `request` - The update book request with changes to apply
    /// * `metadata` - gRPC metadata to include with the request
    ///
    /// # Returns
    ///
    /// A response containing the updated book
    async fn update_book(
        &self,
        request: UpdateBookRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Book>, Status>;

    /// Deletes a book.
    ///
    /// # Arguments
    ///
    /// * `request` - The delete book request containing the book ID
    /// * `metadata` - gRPC metadata to include with the request
    ///
    /// # Returns
    ///
    /// An empty response indicating successful deletion
    async fn delete_book(
        &self,
        request: DeleteBookRequest,
        metadata: MetadataMap,
    ) -> Result<Response<()>, Status>;
}

/// Thread-safe reference-counted book client.
pub type BookClientArc = Arc<dyn BookClient + Send + Sync>;
