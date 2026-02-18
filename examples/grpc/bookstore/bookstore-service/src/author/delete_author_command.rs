use bomboni_common::date_time::UtcDateTime;
use bomboni_request::error::CommonError;
use bookstore_api::model::author::AuthorId;
use grpc_common::auth::context::Context;
use tracing::info;

use crate::{
    author::repository::{AuthorRecordUpdate, AuthorRepositoryArc},
    error::AppResult,
};

/// Command for deleting authors.
///
/// Handles soft deletion of authors by marking them as deleted.
#[derive(Debug, Clone)]
pub struct DeleteAuthorCommand {
    author_repository: AuthorRepositoryArc,
}

impl DeleteAuthorCommand {
    /// Creates a new `DeleteAuthorCommand`.
    ///
    /// # Arguments
    ///
    /// * `author_repository` - Repository for author data persistence
    pub fn new(author_repository: AuthorRepositoryArc) -> Self {
        Self { author_repository }
    }

    /// Executes the author deletion command.
    ///
    /// # Errors
    ///
    /// Returns an error if the user is unauthorized or the author is not found.
    #[tracing::instrument]
    pub async fn execute(&self, context: &Context, id: AuthorId) -> AppResult<()> {
        let user_id = context
            .access_token
            .as_ref()
            .and_then(|token| token.data.as_ref())
            .and_then(|data| data.identities.first())
            .ok_or(CommonError::Unauthorized)?;

        info!(
            user_id = ?user_id,
            author_id = %id,
            "Deleting author"
        );

        let delete_time = UtcDateTime::now();
        let author_update = AuthorRecordUpdate {
            id,
            update_time: Some(delete_time),
            delete_time: Some(delete_time),
            deleted: Some(true),
            display_name: None,
        };

        if !self.author_repository.update(author_update).await? {
            return Err(CommonError::ResourceNotFound.into());
        }

        info!(
            user_id = ?user_id,
            author_id = %id,
            "Successfully deleted author"
        );

        Ok(())
    }
}
