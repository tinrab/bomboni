//! Client implementations for the bookstore service.
//!
//! This module provides client traits and implementations for interacting with
//! both in-memory and remote bookstore services. It includes separate clients
//! for authors and books, as well as a combined client for convenience.

use std::{fmt::Debug, sync::Arc};

use tonic::transport;

pub mod author;
pub mod book;

use crate::client::author::{AuthorClientArc, remote::RemoteAuthorClient};
use crate::client::book::{BookClientArc, remote::RemoteBookClient};

/// Combined client for the bookstore service.
///
/// This struct provides access to both author and book clients through a single
/// interface, making it convenient to work with the complete bookstore API.
#[derive(Debug, Clone)]
pub struct BookstoreClient {
    /// Client for author-related operations.
    pub author: AuthorClientArc,
    /// Client for book-related operations.
    pub book: BookClientArc,
}

impl BookstoreClient {
    /// Creates a new bookstore client with the provided author and book clients.
    ///
    /// # Arguments
    ///
    /// * `author` - The author client to use for author operations
    /// * `book` - The book client to use for book operations
    pub fn new(author: AuthorClientArc, book: BookClientArc) -> Self {
        Self { author, book }
    }

    /// Connects to a remote bookstore service.
    ///
    /// # Errors
    ///
    /// Will return [`transport::Error`] if the connection to the remote service fails.
    pub async fn connect(address: &str) -> Result<Self, transport::Error> {
        let (author, book) = tokio::try_join!(
            async { RemoteAuthorClient::connect(address.into()).await },
            async { RemoteBookClient::connect(address.into()).await },
        )?;
        Ok(Self {
            author: Arc::new(author),
            book: Arc::new(book),
        })
    }
}
