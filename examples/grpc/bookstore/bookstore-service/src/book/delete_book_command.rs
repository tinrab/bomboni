use bomboni_common::date_time::UtcDateTime;

use super::repository::{BookRecordUpdate, BookRepositoryArc};

#[derive(Debug, Clone)]
pub struct DeleteBookCommand {
    book_repository: BookRepositoryArc,
}

impl DeleteBookCommand {
    pub fn new(book_repository: BookRepositoryArc) -> Self {
        Self { book_repository }
    }

    pub async fn execute(
        &self,
        id: bookstore_api::model::book::BookId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = UtcDateTime::now();

        let update = BookRecordUpdate {
            id,
            update_time: Some(now),
            delete_time: Some(now),
            deleted: Some(true),
            display_name: None,
            author: None,
            isbn: None,
            description: None,
            price_cents: None,
            page_count: None,
        };

        let updated = self.book_repository.update(update).await?;

        if !updated {
            return Err("Book not found".into());
        }

        Ok(())
    }
}
