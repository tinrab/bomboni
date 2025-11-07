use bomboni_common::{date_time::UtcDateTime, id::worker::WorkerIdGenerator};
use bomboni_request::error::{CommonError, RequestError};
use bookstore_api::model::author::{AuthorId, AuthorModel};
use grpc_common::auth::context::Context;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    author::repository::{AuthorRecordInsert, AuthorRepositoryArc},
    error::AppResult,
};

#[derive(Debug, Clone)]
pub struct CreateAuthorCommand {
    id_generator: Arc<Mutex<WorkerIdGenerator>>,
    author_repository: AuthorRepositoryArc,
}

#[derive(Debug, Clone)]
pub struct CreateAuthorCommandInput<'a> {
    pub display_name: &'a str,
}

#[derive(Debug, Clone)]
pub struct CreateAuthorCommandOutput {
    pub author: AuthorModel,
}

impl CreateAuthorCommand {
    pub fn new(
        id_generator: Arc<Mutex<WorkerIdGenerator>>,
        author_repository: AuthorRepositoryArc,
    ) -> Self {
        CreateAuthorCommand {
            id_generator,
            author_repository,
        }
    }

    #[tracing::instrument]
    pub async fn execute(
        &self,
        _context: &Context,
        input: CreateAuthorCommandInput<'_>,
    ) -> AppResult<CreateAuthorCommandOutput> {
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
                id: id.clone(),
                create_time,
                display_name: display_name.to_string(),
            })
            .await?;

        let author_record = self
            .author_repository
            .select(&id)
            .await?
            .ok_or_else(|| CommonError::ResourceNotFound)?;

        Ok(CreateAuthorCommandOutput {
            author: author_record.into(),
        })
    }
}
