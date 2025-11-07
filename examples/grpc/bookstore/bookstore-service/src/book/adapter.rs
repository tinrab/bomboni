use std::sync::Arc;

use bomboni_common::id::worker::WorkerIdGenerator;
use bomboni_request::parse::RequestParse;
use bookstore_api::{
    model::book_service::{
        ParsedCreateBookRequest, ParsedDeleteBookRequest, ParsedGetBookRequest,
        ParsedListBooksRequest, ParsedUpdateBookRequest,
    },
    v1::{
        Book, CreateBookRequest, DeleteBookRequest, GetBookRequest, ListBooksRequest,
        ListBooksResponse, SearchBooksRequest, SearchBooksResponse, UpdateBookRequest,
        bookstore_service_server::BookstoreService,
    },
};
use grpc_common::auth::context::ContextBuilder;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::book::{
    create_book_command::{CreateBookCommand, CreateBookCommandInput},
    delete_book_command::DeleteBookCommand,
    query_manager::BookQueryManager,
    repository::BookRepositoryArc,
    update_book_command::UpdateBookCommand,
};

#[derive(Debug)]
pub struct BookAdapter {
    context_builder: ContextBuilder,
    book_query_manager: BookQueryManager,
    create_book_command: CreateBookCommand,
    update_book_command: UpdateBookCommand,
    delete_book_command: DeleteBookCommand,
}

impl BookAdapter {
    pub fn new(
        context_builder: ContextBuilder,
        book_query_manager: BookQueryManager,
        book_repository: BookRepositoryArc,
        id_generator: Arc<Mutex<WorkerIdGenerator>>,
    ) -> Self {
        BookAdapter {
            context_builder,
            book_query_manager,
            create_book_command: CreateBookCommand::new(
                Arc::clone(&id_generator),
                Arc::clone(&book_repository),
            ),
            update_book_command: UpdateBookCommand::new(Arc::clone(&book_repository)),
            delete_book_command: DeleteBookCommand::new(Arc::clone(&book_repository)),
        }
    }
}

#[tonic::async_trait]
impl BookstoreService for BookAdapter {
    #[tracing::instrument]
    async fn get_book(&self, request: Request<GetBookRequest>) -> Result<Response<Book>, Status> {
        let _context = self.context_builder.build_from_metadata(request.metadata());

        let request = ParsedGetBookRequest::parse(request.into_inner())?;

        let mut books = self.book_query_manager.query_batch(&[request.id]).await?;

        Ok(Response::new(books.remove(0)))
    }

    #[tracing::instrument]
    async fn list_books(
        &self,
        request: Request<ListBooksRequest>,
    ) -> Result<Response<ListBooksResponse>, Status> {
        let _context = self.context_builder.build_from_metadata(request.metadata());

        let request = ParsedListBooksRequest::parse_list_query(
            request.into_inner(),
            &self.book_query_manager.list_query_builder(),
        )?;

        let book_list = self
            .book_query_manager
            .query_list(request.query, request.show_deleted)
            .await?;

        Ok(Response::new(ListBooksResponse {
            books: book_list.books,
            next_page_token: book_list.next_page_token,
            total_size: book_list.total_size,
        }))
    }

    #[tracing::instrument]
    async fn create_book(
        &self,
        request: Request<CreateBookRequest>,
    ) -> Result<Response<Book>, Status> {
        let context = self.context_builder.build_from_metadata(request.metadata());

        let request = ParsedCreateBookRequest::parse(request.into_inner())?;

        let book = self
            .create_book_command
            .execute(
                &context,
                CreateBookCommandInput {
                    display_name: &request.display_name,
                    author_id: request.author_id,
                    isbn: &request.isbn,
                    description: &request.description,
                    price_cents: request.price_cents,
                    page_count: request.page_count,
                },
            )
            .await?
            .book;

        Ok(Response::new(book.into()))
    }

    #[tracing::instrument]
    async fn update_book(
        &self,
        request: Request<UpdateBookRequest>,
    ) -> Result<Response<Book>, Status> {
        let _context = self.context_builder.build_from_metadata(request.metadata());

        let request = ParsedUpdateBookRequest::parse(request.into_inner())?;

        todo!()
        // let book = self
        //     .update_book_command
        //     .execute(
        //         &context,
        //         UpdateBookCommandInput {
        //             id: request.id,
        //             title: &request.title,
        //             author_id: request.author_id,
        //             isbn: &request.isbn,
        //             description: &request.description,
        //             price_cents: request.price_cents,
        //             page_count: request.page_count,
        //         },
        //     )
        //     .await?
        //     .book;

        // Ok(Response::new(book.into()))
    }

    #[tracing::instrument]
    async fn delete_book(
        &self,
        request: Request<DeleteBookRequest>,
    ) -> Result<Response<()>, Status> {
        let context = self.context_builder.build_from_metadata(request.metadata());

        let request = ParsedDeleteBookRequest::parse(request.into_inner())?;

        self.delete_book_command
            .execute(&context, &request.id)
            .await?;

        Ok(Response::new(()))
    }

    async fn search_books(
        &self,
        request: Request<SearchBooksRequest>,
    ) -> Result<Response<SearchBooksResponse>, Status> {
        Err(Status::unimplemented("search_books not yet implemented"))
    }
}
