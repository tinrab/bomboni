use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Response, Status, metadata::MetadataMap};

use bomboni_common::date_time::UtcDateTime;
use bomboni_request::parse::RequestParse;

use crate::client::author_client::AuthorClient;
use crate::model::author::AuthorId;
use crate::model::author_service::{
    ParsedCreateAuthorRequest, ParsedDeleteAuthorRequest, ParsedGetAuthorRequest,
    ParsedUpdateAuthorRequest,
};
use crate::v1::Author;
use crate::v1::{
    CreateAuthorRequest, DeleteAuthorRequest, GetAuthorRequest, ListAuthorsRequest,
    ListAuthorsResponse, UpdateAuthorRequest,
};

#[derive(Debug, Clone)]
pub struct MemoryAuthorClient {
    authors: Arc<RwLock<HashMap<AuthorId, Author>>>,
}

impl Default for MemoryAuthorClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryAuthorClient {
    pub fn new() -> Self {
        MemoryAuthorClient {
            authors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_data(authors: Vec<Author>) -> Self {
        let author_map = authors
            .into_iter()
            .filter_map(|author| {
                let id = AuthorId::parse_name(&author.name)?;
                Some((id, author))
            })
            .collect();
        MemoryAuthorClient {
            authors: Arc::new(RwLock::new(author_map)),
        }
    }

    pub async fn get_authors(&self) -> Vec<Author> {
        self.authors.read().await.values().cloned().collect()
    }
}

#[async_trait::async_trait]
impl AuthorClient for MemoryAuthorClient {
    async fn get_author(
        &self,
        request: GetAuthorRequest,
        _metadata: MetadataMap,
    ) -> Result<Response<Author>, Status> {
        let request = ParsedGetAuthorRequest::parse(request)?;
        let authors = self.authors.read().await;
        match authors.get(&request.id) {
            Some(author) => Ok(Response::new(author.clone())),
            None => Err(Status::not_found("Author not found")),
        }
    }

    async fn list_authors(
        &self,
        _request: ListAuthorsRequest,
        _metadata: MetadataMap,
    ) -> Result<Response<ListAuthorsResponse>, Status> {
        let authors = self.authors.read().await;
        let authors_vec: Vec<Author> = authors.values().cloned().collect();
        let total_size = authors_vec.len() as i64;
        Ok(Response::new(ListAuthorsResponse {
            authors: authors_vec,
            next_page_token: None,
            total_size,
        }))
    }

    async fn create_author(
        &self,
        request: CreateAuthorRequest,
        _metadata: MetadataMap,
    ) -> Result<Response<Author>, Status> {
        let request = ParsedCreateAuthorRequest::parse(request)?;
        let id = AuthorId::new(bomboni_common::id::Id::generate());
        let now = UtcDateTime::now();
        let timestamp: bomboni_proto::google::protobuf::Timestamp = now.into();

        let author = Author {
            name: id.to_name(),
            create_time: Some(timestamp.clone()),
            update_time: Some(timestamp.clone()),
            delete_time: None,
            deleted: false,
            etag: None,
            display_name: request.display_name,
        };

        let mut authors = self.authors.write().await;
        authors.insert(id, author.clone());
        Ok(Response::new(author))
    }

    async fn update_author(
        &self,
        request: UpdateAuthorRequest,
        _metadata: MetadataMap,
    ) -> Result<Response<Author>, Status> {
        let request = ParsedUpdateAuthorRequest::parse(request)?;
        let mut authors = self.authors.write().await;
        match authors.get_mut(&request.id) {
            Some(author) => {
                if let Some(display_name) = request.display_name {
                    author.display_name = display_name;
                }
                let now = UtcDateTime::now();
                author.update_time = Some(now.into());
                Ok(Response::new(author.clone()))
            }
            None => Err(Status::not_found("Author not found")),
        }
    }

    async fn delete_author(
        &self,
        request: DeleteAuthorRequest,
        _metadata: MetadataMap,
    ) -> Result<Response<()>, Status> {
        let request = ParsedDeleteAuthorRequest::parse(request)?;
        let mut authors = self.authors.write().await;
        match authors.remove(&request.id) {
            Some(_) => Ok(Response::new(())),
            None => Err(Status::not_found("Author not found")),
        }
    }
}
