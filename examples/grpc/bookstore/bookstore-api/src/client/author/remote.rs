use std::fmt::{self, Debug, Formatter};

use tonic::{
    Request, Response, Status,
    metadata::MetadataMap,
    transport::{self, Channel},
};

use crate::{
    client::author::AuthorClient,
    v1::{
        Author, CreateAuthorRequest, DeleteAuthorRequest, GetAuthorRequest, ListAuthorsRequest,
        ListAuthorsResponse, UpdateAuthorRequest, author_service_client::AuthorServiceClient,
    },
};

/// Remote implementation of the author client.
///
/// This implementation connects to a remote gRPC author service,
/// allowing interaction with authors over the network.
pub struct RemoteAuthorClient {
    client: AuthorServiceClient<Channel>,
}

impl RemoteAuthorClient {
    /// Connects to a remote author service.
    ///
    /// # Errors
    ///
    /// Will return [`transport::Error`] if the connection to the remote service fails.
    pub async fn connect(address: String) -> Result<Self, transport::Error> {
        Ok(Self {
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
