use bomboni_common::{date_time::UtcDateTime, id::worker::WorkerIdGenerator};
use bomboni_request::error::{CommonError, RequestError};
use bookstore_api::model::author::{AuthorId, AuthorModel};
use grpc_common::auth::context::Context;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::{
    author::repository::{AuthorRecordInsert, AuthorRepositoryArc},
    error::AppResult,
};

/// Command for creating new authors.
///
/// Handles the business logic for author creation including ID generation
/// and validation.
#[derive(Debug, Clone)]
pub struct CreateAuthorCommand {
    id_generator: Arc<Mutex<WorkerIdGenerator>>,
    author_repository: AuthorRepositoryArc,
}

/// Input data for creating an author.
#[derive(Debug, Clone)]
pub struct CreateAuthorCommandInput<'a> {
    /// Author display name
    pub display_name: &'a str,
}

/// Output data from author creation.
#[derive(Debug, Clone)]
pub struct CreateAuthorCommandOutput {
    /// The created author model
    pub author: AuthorModel,
}

impl CreateAuthorCommand {
    /// Creates a new `CreateAuthorCommand`.
    ///
    /// # Arguments
    ///
    /// * `id_generator` - Generator for creating unique author IDs
    /// * `author_repository` - Repository for persisting author data
    pub fn new(
        id_generator: Arc<Mutex<WorkerIdGenerator>>,
        author_repository: AuthorRepositoryArc,
    ) -> Self {
        Self {
            id_generator,
            author_repository,
        }
    }

    /// Executes the author creation command.
    ///
    /// # Errors
    ///
    /// Returns an error if author creation fails.
    #[tracing::instrument]
    pub async fn execute(
        &self,
        context: &Context,
        input: CreateAuthorCommandInput<'_>,
    ) -> AppResult<CreateAuthorCommandOutput> {
        let user_id = context
            .access_token
            .as_ref()
            .and_then(|token| token.data.as_ref())
            .and_then(|data| data.identities.first())
            .ok_or(CommonError::Unauthorized)?;

        info!(
            user_id = ?user_id,
            display_name = %input.display_name,
            "Creating author"
        );

        let display_name = input.display_name.trim();
        if display_name.is_empty() {
            return Err(
                RequestError::field("display_name", CommonError::RequiredFieldMissing).into(),
            );
        }

        let id = AuthorId::new(self.id_generator.lock().await.generate());
        let create_time = UtcDateTime::now();

        self.author_repository
            .insert(AuthorRecordInsert {
                id,
                create_time,
                display_name: display_name.to_string(),
            })
            .await?;

        let author_record = self
            .author_repository
            .select(&id)
            .await?
            .ok_or(CommonError::ResourceNotFound)?;

        info!(
            user_id = ?user_id,
            author_id = %id,
            "Successfully created author"
        );

        Ok(CreateAuthorCommandOutput {
            author: author_record.into(),
        })
    }
}
