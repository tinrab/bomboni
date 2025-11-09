use std::sync::Arc;

use bomboni_common::{date_time::UtcDateTime, id::worker::WorkerIdGenerator};
use bomboni_request::error::{CommonError, RequestError};
use bookstore_api::{
    model::{
        author::AuthorId,
        book::{BookId, BookModel},
    },
    v1::Book,
};
use grpc_common::auth::context::Context;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::{
    book::repository::{BookRecordInsert, BookRepositoryArc},
    error::AppResult,
};

/// Command for creating new books.
///
/// Handles the business logic for book creation including ID generation
/// and validation.
#[derive(Debug, Clone)]
pub struct CreateBookCommand {
    id_generator: Arc<Mutex<WorkerIdGenerator>>,
    book_repository: BookRepositoryArc,
}

/// Input data for creating a book.
#[derive(Debug, Clone)]
pub struct CreateBookCommandInput<'a> {
    /// Book title
    pub display_name: &'a str,
    /// Author ID
    pub author_id: AuthorId,
    /// ISBN number
    pub isbn: &'a str,
    /// Book description
    pub description: &'a str,
    /// Price in cents
    pub price_cents: i64,
    /// Number of pages
    pub page_count: i32,
}

/// Output data from book creation.
#[derive(Debug, Clone)]
pub struct CreateBookCommandOutput {
    /// The created book model
    pub book: BookModel,
}

impl CreateBookCommand {
    /// Creates a new `CreateBookCommand`.
    ///
    /// # Arguments
    ///
    /// * `id_generator` - Generator for creating unique book IDs
    /// * `book_repository` - Repository for persisting book data
    pub fn new(
        id_generator: Arc<Mutex<WorkerIdGenerator>>,
        book_repository: BookRepositoryArc,
    ) -> Self {
        Self {
            id_generator,
            book_repository,
        }
    }

    /// Executes the book creation command.
    ///
    /// # Errors
    ///
    /// Returns an error if user is unauthorized or book creation fails.
    #[tracing::instrument]
    pub async fn execute(
        &self,
        context: &Context,
        input: CreateBookCommandInput<'_>,
    ) -> AppResult<CreateBookCommandOutput> {
        let user_id = context
            .access_token
            .as_ref()
            .and_then(|token| token.data.as_ref())
            .and_then(|data| data.identities.first())
            .ok_or(CommonError::Unauthorized)?;

        info!(
            user_id = ?user_id,
            "Creating book with input: display_name={}, author_id={}, isbn={}, description={}, price_cents={}, page_count={}",
            input.display_name,
            input.author_id,
            input.isbn,
            input.description,
            input.price_cents,
            input.page_count
        );

        let display_name = input.display_name.trim();
        if display_name.is_empty() {
            error!(
                user_id = ?user_id,
                "Invalid display name: empty string"
            );
            return Err(RequestError::field(
                Book::DISPLAY_NAME_FIELD_NAME,
                CommonError::InvalidDisplayName,
            )
            .into());
        }

        let id = BookId::new(self.id_generator.lock().await.generate());
        let create_time = UtcDateTime::now();

        self.book_repository
            .insert(BookRecordInsert {
                id,
                create_time,
                display_name: display_name.to_string(),
                author_id: input.author_id,
                isbn: input.isbn.to_string(),
                description: input.description.to_string(),
                price_cents: input.price_cents,
                page_count: input.page_count,
            })
            .await?;

        info!(
            user_id = ?user_id,
            "Inserting book record with id: {}",
            id
        );

        let book_record = self
            .book_repository
            .select(&id)
            .await?
            .ok_or(CommonError::ResourceNotFound)?;

        info!(
            user_id = ?user_id,
            "Successfully created book: {} ({})",
            book_record.display_name,
            book_record.id
        );

        Ok(CreateBookCommandOutput {
            book: book_record.into(),
        })
    }
}
