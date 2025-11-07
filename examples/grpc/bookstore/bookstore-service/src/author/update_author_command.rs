use bomboni_common::date_time::UtcDateTime;
use bomboni_request::error::{CommonError, RequestError};
use bookstore_api::{
    model::author::{AuthorId, AuthorModel},
    v1::Author,
};
use grpc_common::auth::context::Context;

use crate::{
    author::repository::{AuthorRecordUpdate, AuthorRepositoryArc},
    error::AppResult,
};

#[derive(Debug, Clone)]
pub struct UpdateAuthorCommand {
    author_repository: AuthorRepositoryArc,
}

#[derive(Debug, Clone)]
pub struct UpdateAuthorCommandInput<'a> {
    pub id: AuthorId,
    pub display_name: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct UpdateAuthorCommandOutput {
    pub author: AuthorModel,
}

impl UpdateAuthorCommand {
    pub fn new(author_repository: AuthorRepositoryArc) -> Self {
        UpdateAuthorCommand { author_repository }
    }

    #[tracing::instrument]
    pub async fn execute(
        &self,
        _context: &Context,
        input: UpdateAuthorCommandInput<'_>,
    ) -> AppResult<UpdateAuthorCommandOutput> {
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
        updated_author.resource.update_time = Some(update_time);

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

        if !self.author_repository.update(record).await? {
            return Err(CommonError::ResourceNotFound.into());
        }

        Ok(UpdateAuthorCommandOutput {
            author: updated_author,
        })
    }
}
