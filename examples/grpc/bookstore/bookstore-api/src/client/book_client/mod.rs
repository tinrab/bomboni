use std::fmt::Debug;
use std::sync::Arc;
use tonic::metadata::MetadataMap;
use tonic::{Response, Status};

use crate::v1::Book;
use crate::v1::{
    CreateBookRequest, DeleteBookRequest, GetBookRequest, ListBooksRequest, ListBooksResponse,
    UpdateBookRequest,
};

pub mod memory;
pub mod remote;

#[async_trait::async_trait]
pub trait BookClient: Debug {
    async fn get_book(
        &self,
        request: GetBookRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Book>, Status>;

    async fn list_books(
        &self,
        request: ListBooksRequest,
        metadata: MetadataMap,
    ) -> Result<Response<ListBooksResponse>, Status>;

    async fn create_book(
        &self,
        request: CreateBookRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Book>, Status>;

    async fn update_book(
        &self,
        request: UpdateBookRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Book>, Status>;

    async fn delete_book(
        &self,
        request: DeleteBookRequest,
        metadata: MetadataMap,
    ) -> Result<Response<()>, Status>;
}

pub type BookClientArc = Arc<dyn BookClient + Send + Sync>;
