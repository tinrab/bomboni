use bomboni::common::{date_time::UtcDateTime, id::Id};
use bomboni::proto::google::protobuf::Timestamp;
use bomboni::request::parse::RequestParse;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tokio::sync::RwLock;
use tonic::{Response, Status, metadata::MetadataMap};

use crate::{
    client::author::AuthorClient,
    model::{
        author::AuthorId,
        author_service::{
            ParsedCreateAuthorRequest, ParsedDeleteAuthorRequest, ParsedGetAuthorRequest,
            ParsedUpdateAuthorRequest,
        },
    },
    v1::{
        Author, CreateAuthorRequest, DeleteAuthorRequest, GetAuthorRequest, ListAuthorsRequest,
        ListAuthorsResponse, UpdateAuthorRequest,
    },
};

/// In-memory implementation of the author client.
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
    /// Creates a new empty memory author client.
    pub fn new() -> Self {
        Self {
            authors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new memory author client with initial data.
    ///
    /// # Arguments
    ///
    /// * `authors` - Vector of authors to initialize the client with
    ///
    /// # Notes
    ///
    /// Authors with invalid names will be filtered out during initialization.
    pub fn with_data(authors: Vec<Author>) -> Self {
        let author_map = authors
            .into_iter()
            .filter_map(|author| {
                let id = AuthorId::parse_name(&author.name)?;
                Some((id, author))
            })
            .collect();
        Self {
            authors: Arc::new(RwLock::new(author_map)),
        }
    }

    /// Retrieves all authors stored in memory.
    ///
    /// # Returns
    ///
    /// A vector containing all stored authors
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
        self.authors.read().await.get(&request.id).map_or_else(
            || Err(Status::not_found("Author not found")),
            |author| Ok(Response::new(author.clone())),
        )
    }

    async fn list_authors(
        &self,
        _request: ListAuthorsRequest,
        _metadata: MetadataMap,
    ) -> Result<Response<ListAuthorsResponse>, Status> {
        let authors_vec: Vec<Author> = self.authors.read().await.values().cloned().collect();
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
        let id = AuthorId::new(Id::generate());
        let timestamp: Timestamp = UtcDateTime::now().into();

        let author = Author {
            name: id.to_name(),
            create_time: Some(timestamp),
            update_time: Some(timestamp),
            delete_time: None,
            deleted: false,
            etag: None,
            display_name: request.display_name,
        };

        self.authors.write().await.insert(id, author.clone());
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
