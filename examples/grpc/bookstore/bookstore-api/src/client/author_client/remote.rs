use std::fmt;
use std::fmt::{Debug, Formatter};

use tonic::{
    Request, Response, Status,
    metadata::MetadataMap,
    transport::{self, Channel},
};

use crate::client::author_client::AuthorClient;
use crate::v1::Author;
use crate::v1::{
    CreateAuthorRequest, DeleteAuthorRequest, GetAuthorRequest, ListAuthorsRequest,
    ListAuthorsResponse, UpdateAuthorRequest, author_service_client::AuthorServiceClient,
};

pub struct RemoteAuthorClient {
    client: AuthorServiceClient<Channel>,
}

impl RemoteAuthorClient {
    pub async fn connect(address: String) -> Result<Self, transport::Error> {
        Ok(RemoteAuthorClient {
            client: AuthorServiceClient::connect(address).await?,
        })
    }
}

#[async_trait::async_trait]
impl AuthorClient for RemoteAuthorClient {
    async fn get_author(
        &self,
        request: GetAuthorRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Author>, Status> {
        let mut request = Request::new(request);
        *request.metadata_mut() = metadata;
        let mut client = self.client.clone();
        client.get_author(request).await
    }

    async fn list_authors(
        &self,
        request: ListAuthorsRequest,
        metadata: MetadataMap,
    ) -> Result<Response<ListAuthorsResponse>, Status> {
        let mut request = Request::new(request);
        *request.metadata_mut() = metadata;
        let mut client = self.client.clone();
        client.list_authors(request).await
    }

    async fn create_author(
        &self,
        request: CreateAuthorRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Author>, Status> {
        let mut request = Request::new(request);
        *request.metadata_mut() = metadata;
        let mut client = self.client.clone();
        client.create_author(request).await
    }

    async fn update_author(
        &self,
        request: UpdateAuthorRequest,
        metadata: MetadataMap,
    ) -> Result<Response<Author>, Status> {
        let mut request = Request::new(request);
        *request.metadata_mut() = metadata;
        let mut client = self.client.clone();
        client.update_author(request).await
    }

    async fn delete_author(
        &self,
        request: DeleteAuthorRequest,
        metadata: MetadataMap,
    ) -> Result<Response<()>, Status> {
        let mut request = Request::new(request);
        *request.metadata_mut() = metadata;
        let mut client = self.client.clone();
        client.delete_author(request).await
    }
}

impl Debug for RemoteAuthorClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteAuthorClient").finish()
    }
}
