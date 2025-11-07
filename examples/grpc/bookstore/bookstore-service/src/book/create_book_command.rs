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

use crate::{
    book::repository::{BookRecordInsert, BookRepositoryArc},
    error::AppResult,
};

#[derive(Debug, Clone)]
pub struct CreateBookCommand {
    id_generator: Arc<Mutex<WorkerIdGenerator>>,
    book_repository: BookRepositoryArc,
}

#[derive(Debug, Clone)]
pub struct CreateBookCommandInput<'a> {
    pub display_name: &'a str,
    pub author_id: AuthorId,
    pub isbn: &'a str,
    pub description: &'a str,
    pub price_cents: i64,
    pub page_count: i32,
}

#[derive(Debug, Clone)]
pub struct CreateBookCommandOutput {
    pub book: BookModel,
}

impl CreateBookCommand {
    pub fn new(
        id_generator: Arc<Mutex<WorkerIdGenerator>>,
        book_repository: BookRepositoryArc,
    ) -> Self {
        CreateBookCommand {
            id_generator,
            book_repository,
        }
    }

    #[tracing::instrument]
    pub async fn execute(
        &self,
        _context: &Context,
        input: CreateBookCommandInput<'_>,
    ) -> AppResult<CreateBookCommandOutput> {
        let display_name = input.display_name.trim();
        if display_name.is_empty() {
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
                id: id.clone(),
                create_time,
                display_name: display_name.to_string(),
                author_id: input.author_id,
                isbn: input.isbn.to_string(),
                description: input.description.to_string(),
                price_cents: input.price_cents,
                page_count: input.page_count,
            })
            .await?;

        let book_record = self
            .book_repository
            .select(&id)
            .await?
            .ok_or_else(|| CommonError::ResourceNotFound)?;

        Ok(CreateBookCommandOutput {
            book: book_record.into(),
        })
    }
}
