use bomboni_common::date_time::UtcDateTime;
use bookstore_api::model::book::{BookId, BookModel};
use bookstore_api::v1::Book;

use super::repository::{BookRecordUpdate, BookRepositoryArc};

#[derive(Debug, Clone)]
pub struct UpdateBookCommand {
    book_repository: BookRepositoryArc,
}

#[derive(Debug)]
pub struct UpdateBookCommandInput<'a> {
    pub id: BookId,
    pub display_name: Option<&'a str>,
    pub author: Option<&'a str>,
    pub isbn: Option<&'a str>,
    pub description: Option<&'a str>,
    pub price_cents: Option<i64>,
    pub page_count: Option<i32>,
}

#[derive(Debug)]
pub struct UpdateBookCommandResult {
    pub book: Book,
}

impl UpdateBookCommand {
    pub fn new(book_repository: BookRepositoryArc) -> Self {
        Self { book_repository }
    }

    pub async fn execute(
        &self,
        input: UpdateBookCommandInput<'_>,
    ) -> Result<UpdateBookCommandResult, Box<dyn std::error::Error + Send + Sync>> {
        let now = UtcDateTime::now();

        let update = BookRecordUpdate {
            id: input.id,
            update_time: Some(now),
            delete_time: None,
            deleted: None,
            display_name: input.display_name.as_deref(),
            author: input.author.as_deref(),
            isbn: input.isbn.as_deref(),
            description: input.description.as_deref(),
            price_cents: input.price_cents,
            page_count: input.page_count,
        };

        let updated = self.book_repository.update(update).await?;

        if !updated {
            return Err("Book not found".into());
        }

        let book_record = self.book_repository.select(input.id).await?.unwrap();
        let book = BookModel::from(book_record).into();

        Ok(UpdateBookCommandResult { book })
    }
}
