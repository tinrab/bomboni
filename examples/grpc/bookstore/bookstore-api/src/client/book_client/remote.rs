use std::fmt;
use std::fmt::{Debug, Formatter};

use tonic::{
    Request, Response, Status,
    metadata::MetadataMap,
    transport::{self, Channel},
};

use crate::client::book_client::BookClient;
use crate::v1::{
    Book, CreateBookRequest, DeleteBookRequest, GetBookRequest, ListBooksRequest,
    ListBooksResponse, UpdateBookRequest, bookstore_service_client::BookstoreServiceClient,
};

#[derive(Clone)]
pub struct RemoteBookClient {
    client: BookstoreServiceClient<Channel>,
}

impl RemoteBookClient {
    pub async fn connect(address: String) -> Result<Self, transport::Error> {
        Ok(RemoteBookClient {
            client: BookstoreServiceClient::connect(address).await?,
        })
    }
}

#[async_trait::async_trait]
impl BookClient for RemoteBookClient {
    async fn get_book(
        &self,
        request: GetBookRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Book>, Status> {
        let mut request = Request::new(request);
        *request.metadata_mut() = metadata;
        let mut client = self.client.clone();
        client.get_book(request).await
    }

    async fn list_books(
        &self,
        request: ListBooksRequest,
        metadata: MetadataMap,
    ) -> Result<Response<ListBooksResponse>, Status> {
        let mut request = Request::new(request);
        *request.metadata_mut() = metadata;
        let mut client = self.client.clone();
        client.list_books(request).await
    }

    async fn create_book(
        &self,
        request: CreateBookRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Book>, Status> {
        let mut request = Request::new(request);
        *request.metadata_mut() = metadata;
        let mut client = self.client.clone();
        client.create_book(request).await
    }

    async fn update_book(
        &self,
        request: UpdateBookRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Book>, Status> {
        let mut request = Request::new(request);
        *request.metadata_mut() = metadata;
        let mut client = self.client.clone();
        client.update_book(request).await
    }

    async fn delete_book(
        &self,
        request: DeleteBookRequest,
        metadata: MetadataMap,
    ) -> Result<Response<()>, Status> {
        let mut request = Request::new(request);
        *request.metadata_mut() = metadata;
        let mut client = self.client.clone();
        client.delete_book(request).await
    }
}

impl Debug for RemoteBookClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteBookClient").finish()
    }
}
