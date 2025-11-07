use std::sync::Arc;

use bomboni_common::id::worker::WorkerIdGenerator;
use bomboni_request::parse::RequestParse;
use bookstore_api::{
    model::author_service::{
        ParsedCreateAuthorRequest, ParsedDeleteAuthorRequest, ParsedGetAuthorRequest,
        ParsedListAuthorsRequest,
    },
    v1::{
        Author, CreateAuthorRequest, DeleteAuthorRequest, GetAuthorRequest, ListAuthorsRequest,
        ListAuthorsResponse, SearchAuthorsRequest, SearchAuthorsResponse, UpdateAuthorRequest,
        author_service_server::AuthorService,
    },
};
use grpc_common::auth::context::ContextBuilder;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::author::{
    create_author_command::{CreateAuthorCommand, CreateAuthorCommandInput},
    delete_author_command::DeleteAuthorCommand,
    query_manager::AuthorQueryManager,
    repository::AuthorRepositoryArc,
    update_author_command::UpdateAuthorCommand,
};

#[derive(Debug)]
pub struct AuthorAdapter {
    context_builder: ContextBuilder,
    author_query_manager: AuthorQueryManager,
    create_author_command: CreateAuthorCommand,
    update_author_command: UpdateAuthorCommand,
    delete_author_command: DeleteAuthorCommand,
}

impl AuthorAdapter {
    pub fn new(
        id_generator: Arc<Mutex<WorkerIdGenerator>>,
        context_builder: ContextBuilder,
        author_query_manager: AuthorQueryManager,
        author_repository: AuthorRepositoryArc,
    ) -> Self {
        AuthorAdapter {
            context_builder,
            author_query_manager,
            create_author_command: CreateAuthorCommand::new(
                id_generator,
                author_repository.clone(),
            ),
            update_author_command: UpdateAuthorCommand::new(author_repository.clone()),
            delete_author_command: DeleteAuthorCommand::new(author_repository),
        }
    }
}

#[tonic::async_trait]
impl AuthorService for AuthorAdapter {
    #[tracing::instrument]
    async fn get_author(
        &self,
        request: Request<GetAuthorRequest>,
    ) -> Result<Response<Author>, Status> {
        let _context = self.context_builder.build_from_metadata(request.metadata());

        let request = ParsedGetAuthorRequest::parse(request.into_inner())?;

        let mut authors = self.author_query_manager.query_batch(&[request.id]).await?;

        Ok(Response::new(authors.remove(0)))
    }

    #[tracing::instrument]
    async fn list_authors(
        &self,
        request: Request<ListAuthorsRequest>,
    ) -> Result<Response<ListAuthorsResponse>, Status> {
        let _context = self.context_builder.build_from_metadata(request.metadata());

        let request = ParsedListAuthorsRequest::parse_list_query(
            request.into_inner(),
            &self.author_query_manager.list_query_builder(),
        )?;

        let author_list = self
            .author_query_manager
            .query_list(request.query, request.show_deleted)
            .await?;

        Ok(Response::new(ListAuthorsResponse {
            authors: author_list.authors,
            next_page_token: author_list.next_page_token,
            total_size: author_list.total_size,
        }))
    }

    #[tracing::instrument]
    async fn create_author(
        &self,
        request: Request<CreateAuthorRequest>,
    ) -> Result<Response<Author>, Status> {
        let context = self.context_builder.build_from_metadata(request.metadata());

        let request = ParsedCreateAuthorRequest::parse(request.into_inner())?;

        let result = self
            .create_author_command
            .execute(
                &context,
                CreateAuthorCommandInput {
                    display_name: &request.display_name,
                },
            )
            .await?;

        Ok(Response::new(result.author.into()))
    }

    #[tracing::instrument]
    async fn update_author(
        &self,
        _request: Request<UpdateAuthorRequest>,
    ) -> Result<Response<Author>, Status> {
        Err(Status::unimplemented("Update author not yet implemented"))
    }

    #[tracing::instrument]
    async fn delete_author(
        &self,
        request: Request<DeleteAuthorRequest>,
    ) -> Result<Response<()>, Status> {
        let context = self.context_builder.build_from_metadata(request.metadata());

        let request = ParsedDeleteAuthorRequest::parse(request.into_inner())?;

        self.delete_author_command
            .execute(&context, request.id)
            .await?;

        Ok(Response::new(()))
    }

    #[tracing::instrument]
    async fn search_authors(
        &self,
        _request: tonic::Request<SearchAuthorsRequest>,
    ) -> Result<tonic::Response<SearchAuthorsResponse>, Status> {
        Err(Status::unimplemented("search_authors not yet implemented"))
    }
}
