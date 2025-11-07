use bomboni_common::date_time::UtcDateTime;
use bomboni_request::error::{CommonError, RequestError};
use bookstore_api::{
    model::{
        author::AuthorId,
        book::{BookId, BookModel},
    },
    v1::Book,
};
use grpc_common::auth::context::Context;

use crate::{
    book::repository::{BookRecordUpdate, BookRepositoryArc},
    error::AppResult,
};

#[derive(Debug, Clone)]
pub struct UpdateBookCommand {
    book_repository: BookRepositoryArc,
}

#[derive(Debug, Clone)]
pub struct UpdateBookCommandInput<'a> {
    pub id: BookId,
    pub display_name: Option<&'a str>,
    pub author_id: Option<AuthorId>,
    pub isbn: Option<&'a str>,
    pub description: Option<&'a str>,
    pub price_cents: Option<i64>,
    pub page_count: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct UpdateBookCommandOutput {
    pub book: BookModel,
}

impl UpdateBookCommand {
    pub fn new(book_repository: BookRepositoryArc) -> Self {
        UpdateBookCommand { book_repository }
    }

    #[tracing::instrument]
    pub async fn execute(
        &self,
        _context: &Context,
        input: UpdateBookCommandInput<'_>,
    ) -> AppResult<UpdateBookCommandOutput> {
        let mut updated_book: BookModel = self
            .book_repository
            .select(&input.id)
            .await?
            .ok_or(CommonError::ResourceNotFound)?
            .into();

        let update_time = UtcDateTime::now();
        let mut updated = false;

        let mut record = BookRecordUpdate {
            id: &input.id,
            update_time: Some(update_time),
            delete_time: None,
            deleted: None,
            display_name: None,
            author_id: None,
            isbn: None,
            description: None,
            price_cents: None,
            page_count: None,
        };

        if let Some(display_name) = input.display_name {
            let display_name = display_name.trim();
            if display_name.is_empty() {
                return Err(RequestError::field(
                    Book::DISPLAY_NAME_FIELD_NAME,
                    CommonError::InvalidDisplayName,
                )
                .into());
            }
            record.display_name = Some(display_name);
            updated_book.display_name = display_name.to_string();
            updated = true;
        }

        if let Some(author_id) = input.author_id {
            record.author_id = Some(author_id);
            updated_book.author_id = author_id;
            updated = true;
        }

        if let Some(isbn) = input.isbn {
            record.isbn = Some(isbn);
            updated_book.isbn = isbn.to_string();
            updated = true;
        }

        if let Some(description) = input.description {
            record.description = Some(description);
            updated_book.description = description.to_string();
            updated = true;
        }

        if let Some(price_cents) = input.price_cents {
            record.price_cents = Some(price_cents);
            updated_book.price_cents = price_cents;
            updated = true;
        }

        if let Some(page_count) = input.page_count {
            record.page_count = Some(page_count);
            updated_book.page_count = page_count;
            updated = true;
        }

        if updated {
            updated_book.resource.update_time = Some(update_time);

            if !self.book_repository.update(record).await? {
                return Err(CommonError::ResourceNotFound.into());
            }
        }

        Ok(UpdateBookCommandOutput { book: updated_book })
    }
}
