use bomboni_common::date_time::UtcDateTime;
use bomboni_request::error::CommonError;
use bookstore_api::model::author::AuthorId;
use grpc_common::auth::context::Context;

use crate::{
    author::repository::{AuthorRecordUpdate, AuthorRepositoryArc},
    error::AppResult,
};

#[derive(Debug, Clone)]
pub struct DeleteAuthorCommand {
    author_repository: AuthorRepositoryArc,
}

impl DeleteAuthorCommand {
    pub fn new(author_repository: AuthorRepositoryArc) -> Self {
        DeleteAuthorCommand { author_repository }
    }

    #[tracing::instrument]
    pub async fn execute(&self, _context: &Context, id: AuthorId) -> AppResult<()> {
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

        Ok(())
    }
}
