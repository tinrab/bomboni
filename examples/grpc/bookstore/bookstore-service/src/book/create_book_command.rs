use bomboni_common::date_time::UtcDateTime;
use bookstore_api::model::book::{BookId, BookModel};
use bookstore_api::v1::Book;

use super::repository::{BookRecordInsert, BookRepositoryArc};

#[derive(Debug, Clone)]
pub struct CreateBookCommand {
    book_repository: BookRepositoryArc,
}

#[derive(Debug)]
pub struct CreateBookCommandInput<'a> {
    pub display_name: &'a str,
    pub author: &'a str,
    pub isbn: &'a str,
    pub description: &'a str,
    pub price_cents: i64,
    pub page_count: i32,
}

#[derive(Debug)]
pub struct CreateBookCommandResult {
    pub book: Book,
}

impl CreateBookCommand {
    pub fn new(book_repository: BookRepositoryArc) -> Self {
        Self { book_repository }
    }

    pub async fn execute(
        &self,
        input: CreateBookCommandInput<'_>,
    ) -> Result<CreateBookCommandResult, Box<dyn std::error::Error + Send + Sync>> {
        let id = BookId::new(bomboni_common::id::Id::generate());
        let now = UtcDateTime::now();

        let record = BookRecordInsert {
            id,
            create_time: now,
            display_name: input.display_name,
            author: input.author,
            isbn: input.isbn,
            description: input.description,
            price_cents: input.price_cents,
            page_count: input.page_count,
        };

        self.book_repository.insert(record).await?;

        let book_record = self.book_repository.select(id).await?.unwrap();
        let book = BookModel::from(book_record).into();

        Ok(CreateBookCommandResult { book })
    }
}
