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
use tracing::{error, info};

use crate::{
    book::repository::{BookRecordUpdate, BookRepositoryArc},
    error::AppResult,
};

/// Command for updating existing books.
///
/// Handles updating book information with validation.
#[derive(Debug, Clone)]
pub struct UpdateBookCommand {
    book_repository: BookRepositoryArc,
}

/// Input data for updating a book.
#[derive(Debug, Clone)]
pub struct UpdateBookCommandInput<'a> {
    /// Book ID to update
    pub id: BookId,
    /// New title (optional)
    pub display_name: Option<&'a str>,
    /// New author ID (optional)
    pub author_id: Option<AuthorId>,
    /// New ISBN (optional)
    pub isbn: Option<&'a str>,
    /// New description (optional)
    pub description: Option<&'a str>,
    /// New price in cents (optional)
    pub price_cents: Option<i64>,
    /// New page count (optional)
    pub page_count: Option<i32>,
}

/// Output data from book update.
#[derive(Debug, Clone)]
pub struct UpdateBookCommandOutput {
    /// The updated book model
    pub book: BookModel,
}

impl UpdateBookCommand {
    /// Creates a new `UpdateBookCommand`.
    ///
    /// # Arguments
    ///
    /// * `book_repository` - Repository for book data persistence
    pub fn new(book_repository: BookRepositoryArc) -> Self {
        Self { book_repository }
    }

    /// Executes the book update command.
    ///
    /// # Errors
    ///
    /// Returns an error if user is unauthorized, book not found, or update fails.
    #[tracing::instrument]
    pub async fn execute(
        &self,
        context: &Context,
        input: UpdateBookCommandInput<'_>,
    ) -> AppResult<UpdateBookCommandOutput> {
        let user_id = context
            .access_token
            .as_ref()
            .and_then(|token| token.data.as_ref())
            .and_then(|data| data.identities.first())
            .ok_or(CommonError::Unauthorized)?;

        info!(
            user_id = ?user_id,
            book_id = %input.id,
            "Updating book with input: display_name={:?}, author_id={:?}, isbn={:?}, description={:?}, price_cents={:?}, page_count={:?}",
            input.display_name,
            input.author_id,
            input.isbn,
            input.description,
            input.price_cents,
            input.page_count
        );

        let mut updated_book: BookModel = self
            .book_repository
            .select(&input.id)
            .await?
            .ok_or(CommonError::ResourceNotFound)?
            .into();

        let update_time = UtcDateTime::now();

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
                error!(
                    user_id = ?user_id,
                    book_id = %input.id,
                    "Invalid display name: empty string"
                );
                return Err(RequestError::field(
                    Book::DISPLAY_NAME_FIELD_NAME,
                    CommonError::InvalidDisplayName,
                )
                .into());
            }
            info!(
                user_id = ?user_id,
                book_id = %input.id,
                "Updating display_name to: {}",
                display_name
            );
            record.display_name = Some(display_name);
            updated_book.display_name = display_name.to_string();
        }

        if let Some(author_id) = input.author_id {
            info!(
                user_id = ?user_id,
                book_id = %input.id,
                "Updating author_id to: {}",
                author_id
            );
            record.author_id = Some(author_id);
            updated_book.author_id = author_id;
        }

        if let Some(isbn) = input.isbn {
            info!(
                user_id = ?user_id,
                book_id = %input.id,
                "Updating isbn to: {}",
                isbn
            );
            record.isbn = Some(isbn);
            updated_book.isbn = isbn.to_string();
        }

        if let Some(description) = input.description {
            info!(
                user_id = ?user_id,
                book_id = %input.id,
                "Updating description to: {}",
                description
            );
            record.description = Some(description);
            updated_book.description = description.to_string();
        }

        if let Some(price_cents) = input.price_cents {
            info!(
                user_id = ?user_id,
                book_id = %input.id,
                "Updating price_cents to: {}",
                price_cents
            );
            record.price_cents = Some(price_cents);
            updated_book.price_cents = price_cents;
        }

        if let Some(page_count) = input.page_count {
            info!(
                user_id = ?user_id,
                book_id = %input.id,
                "Updating page_count to: {}",
                page_count
            );
            record.page_count = Some(page_count);
            updated_book.page_count = page_count;
        }

        updated_book.resource.update_time = Some(update_time);

        info!(
            user_id = ?user_id,
            book_id = %input.id,
            "Executing book update"
        );

        if !self.book_repository.update(record).await? {
            error!(
                user_id = ?user_id,
                book_id = %input.id,
                "Failed to update book: not found"
            );
            return Err(CommonError::ResourceNotFound.into());
        }

        info!(
            user_id = ?user_id,
            book_id = %input.id,
            "Successfully updated book: {}",
            updated_book.display_name
        );

        Ok(UpdateBookCommandOutput { book: updated_book })
    }
}
