use std::fmt::Debug;
use std::sync::Arc;
use tonic::metadata::MetadataMap;
use tonic::{Response, Status};

use crate::v1::Author;
use crate::v1::{
    CreateAuthorRequest, DeleteAuthorRequest, GetAuthorRequest, ListAuthorsRequest,
    ListAuthorsResponse, UpdateAuthorRequest,
};

pub mod memory;
pub mod remote;

#[async_trait::async_trait]
pub trait AuthorClient: Debug {
    async fn get_author(
        &self,
        request: GetAuthorRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Author>, Status>;

    async fn list_authors(
        &self,
        request: ListAuthorsRequest,
        metadata: MetadataMap,
    ) -> Result<Response<ListAuthorsResponse>, Status>;

    async fn create_author(
        &self,
        request: CreateAuthorRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Author>, Status>;

    async fn update_author(
        &self,
        request: UpdateAuthorRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Author>, Status>;

    async fn delete_author(
        &self,
        request: DeleteAuthorRequest,
        metadata: MetadataMap,
    ) -> Result<Response<()>, Status>;
}

pub type AuthorClientArc = Arc<dyn AuthorClient + Send + Sync>;
