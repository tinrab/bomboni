use bomboni_common::date_time::UtcDateTime;
use bomboni_request::error::{CommonError, RequestError};
use bookstore_api::{
    model::author::{AuthorId, AuthorModel},
    v1::Author,
};
use grpc_common::auth::context::Context;
use tracing::info;

use crate::{
    author::repository::{AuthorRecordUpdate, AuthorRepositoryArc},
    error::AppResult,
};

/// Command for updating existing authors.
///
/// Handles updating author information with validation.
#[derive(Debug, Clone)]
pub struct UpdateAuthorCommand {
    author_repository: AuthorRepositoryArc,
}

/// Input data for updating an author.
#[derive(Debug, Clone)]
pub struct UpdateAuthorCommandInput<'a> {
    /// Author ID to update
    pub id: AuthorId,
    /// New display name (optional)
    pub display_name: Option<&'a str>,
}

/// Output data from author update.
#[derive(Debug, Clone)]
pub struct UpdateAuthorCommandOutput {
    /// The updated author model
    pub author: AuthorModel,
}

impl UpdateAuthorCommand {
    /// Creates a new `UpdateAuthorCommand`.
    ///
    /// # Arguments
    ///
    /// * `author_repository` - Repository for author data persistence
    pub fn new(author_repository: AuthorRepositoryArc) -> Self {
        Self { author_repository }
    }

    /// Executes the author update command.
    ///
    /// # Errors
    ///
    /// Returns an error if the user is unauthorized, author not found, or update fails.
    #[tracing::instrument]
    pub async fn execute(
        &self,
        context: &Context,
        input: UpdateAuthorCommandInput<'_>,
    ) -> AppResult<UpdateAuthorCommandOutput> {
        let user_id = context
            .access_token
            .as_ref()
            .and_then(|token| token.data.as_ref())
            .and_then(|data| data.identities.first())
            .ok_or(CommonError::Unauthorized)?;

        info!(
            user_id = ?user_id,
            author_id = %input.id,
            display_name = ?input.display_name,
            "Updating author"
        );

        let mut updated_author: AuthorModel = self
            .author_repository
            .select(&input.id)
            .await?
            .ok_or(CommonError::ResourceNotFound)?
            .into();

        let update_time = UtcDateTime::now();

        let mut record = AuthorRecordUpdate {
            id: input.id,
            update_time: Some(update_time),
            delete_time: None,
            deleted: None,
            display_name: None,
        };

        if let Some(display_name) = input.display_name {
            let display_name = display_name.trim();
            if display_name.is_empty() {
                return Err(RequestError::field(
                    Author::DISPLAY_NAME_FIELD_NAME,
                    CommonError::InvalidDisplayName,
                )
                .into());
            }
            record.display_name = Some(display_name);
            updated_author.display_name = display_name.to_string();
        }

        updated_author.resource.update_time = Some(update_time);

        if !self.author_repository.update(record).await? {
            return Err(CommonError::ResourceNotFound.into());
        }

        info!(
            user_id = ?user_id,
            author_id = %input.id,
            "Successfully updated author"
        );

        Ok(UpdateAuthorCommandOutput {
            author: updated_author,
        })
    }
}
