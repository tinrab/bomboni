use bomboni_common::date_time::UtcDateTime;
use bomboni_request::error::CommonError;
use bookstore_api::model::book::BookId;
use grpc_common::auth::context::Context;

use crate::{
    book::repository::{BookRecordUpdate, BookRepositoryArc},
    error::AppResult,
};

#[derive(Debug, Clone)]
pub struct DeleteBookCommand {
    book_repository: BookRepositoryArc,
}

impl DeleteBookCommand {
    pub fn new(book_repository: BookRepositoryArc) -> Self {
        DeleteBookCommand { book_repository }
    }

    #[tracing::instrument]
    pub async fn execute(&self, _context: &Context, id: &BookId) -> AppResult<()> {
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
