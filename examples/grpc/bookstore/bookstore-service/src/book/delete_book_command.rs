use bomboni_common::date_time::UtcDateTime;
use bomboni_request::error::CommonError;
use bookstore_api::model::book::BookId;
use grpc_common::auth::context::Context;
use tracing::info;

use crate::{
    book::repository::{BookRecordUpdate, BookRepositoryArc},
    error::AppResult,
};

/// Command for deleting books.
///
/// Handles soft deletion of books by marking them as deleted.
#[derive(Debug, Clone)]
pub struct DeleteBookCommand {
    book_repository: BookRepositoryArc,
}

impl DeleteBookCommand {
    /// Creates a new `DeleteBookCommand`.
    ///
    /// # Arguments
    ///
    /// * `book_repository` - Repository for book data persistence
    pub fn new(book_repository: BookRepositoryArc) -> Self {
        Self { book_repository }
    }

    /// Executes the book deletion command.
    ///
    /// # Errors
    ///
    /// Returns an error if the user is unauthorized or the book is not found.
    #[tracing::instrument]
    pub async fn execute(&self, context: &Context, id: &BookId) -> AppResult<()> {
        let user_id = context
            .access_token
            .as_ref()
            .and_then(|token| token.data.as_ref())
            .and_then(|data| data.identities.first())
            .ok_or(CommonError::Unauthorized)?;

        info!(
            user_id = ?user_id,
            book_id = %id,
            "Deleting book"
        );

        let delete_time = UtcDateTime::now();
        let book_update = BookRecordUpdate {
            id,
            update_time: Some(delete_time),
            delete_time: Some(delete_time),
            deleted: Some(true),
            display_name: None,
            author_id: None,
            isbn: None,
            description: None,
            price_cents: None,
            page_count: None,
        };

        if !self.book_repository.update(book_update).await? {
            return Err(CommonError::ResourceNotFound.into());
        }

        Ok(())
    }
}
