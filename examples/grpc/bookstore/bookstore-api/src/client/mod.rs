use std::fmt::Debug;
use std::sync::Arc;

use crate::client::author_client::AuthorClientArc;
use crate::client::book_client::BookClientArc;
use tonic::transport;

pub mod author_client;
pub mod book_client;

#[derive(Debug, Clone)]
pub struct BookstoreClient {
    pub author: AuthorClientArc,
    pub book: BookClientArc,
}

impl BookstoreClient {
    pub fn new(author: AuthorClientArc, book: BookClientArc) -> Self {
        BookstoreClient { author, book }
    }

    pub async fn connect(address: &str) -> Result<Self, transport::Error> {
        let (author, book) = tokio::try_join!(
            async { author_client::remote::RemoteAuthorClient::connect(address.into()).await },
            async { book_client::remote::RemoteBookClient::connect(address.into()).await },
        )?;
        Ok(BookstoreClient {
            author: Arc::new(author),
            book: Arc::new(book),
        })
    }
}
