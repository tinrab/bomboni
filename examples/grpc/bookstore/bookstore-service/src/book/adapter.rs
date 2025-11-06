use super::{
    create_book_command::{CreateBookCommand, CreateBookCommandInput},
    delete_book_command::DeleteBookCommand,
    query_manager::BookQueryManager,
    update_book_command::{UpdateBookCommand, UpdateBookCommandInput},
};
use bomboni_request::parse::RequestParse;

use bookstore_api::{
    model::book::book_id_convert,
    model::book_service::{
        ParsedCreateBookRequest, ParsedDeleteBookRequest, ParsedGetBookRequest,
        ParsedListBooksRequest,
    },
    v1::{
        CreateBookRequest, DeleteBookRequest, GetBookRequest, ListBooksRequest, ListBooksResponse,
        SearchBooksRequest, SearchBooksResponse, UpdateBookRequest,
        bookstore_service_server::BookstoreService,
    },
};
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct BookAdapter {
    book_query_manager: BookQueryManager,
    create_book_command: CreateBookCommand,
    update_book_command: UpdateBookCommand,
    delete_book_command: DeleteBookCommand,
}

impl BookAdapter {
    pub fn new(
        book_query_manager: BookQueryManager,
        create_book_command: CreateBookCommand,
        update_book_command: UpdateBookCommand,
        delete_book_command: DeleteBookCommand,
    ) -> Self {
        BookAdapter {
            book_query_manager,
            create_book_command,
            update_book_command,
            delete_book_command,
        }
    }
}

#[tonic::async_trait]
impl BookstoreService for BookAdapter {
    async fn create_book(
        &self,
        request: Request<CreateBookRequest>,
    ) -> Result<Response<bookstore_api::v1::Book>, Status> {
        let request = ParsedCreateBookRequest::parse(request.into_inner())
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let result = self
            .create_book_command
            .execute(CreateBookCommandInput {
                display_name: &request.title,
                author: &request.author,
                isbn: &request.isbn,
                description: &request.description,
                price_cents: request.price_cents,
                page_count: request.page_count,
            })
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(result.book))
    }

    async fn get_book(
        &self,
        request: Request<GetBookRequest>,
    ) -> Result<Response<bookstore_api::v1::Book>, Status> {
        let request = ParsedGetBookRequest::parse(request.into_inner())
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let book = self
            .book_query_manager
            .query_single(request.id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(book))
    }

    async fn update_book(
        &self,
        request: Request<UpdateBookRequest>,
    ) -> Result<Response<bookstore_api::v1::Book>, Status> {
        let request = request.into_inner();

        let id = book_id_convert::parse(request.name)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let book = request
            .book
            .ok_or_else(|| Status::invalid_argument("Book field is required"))?;

        let result = self
            .update_book_command
            .execute(UpdateBookCommandInput {
                id,
                display_name: Some(&book.display_name),
                author: Some(&book.author),
                isbn: Some(&book.isbn),
                description: Some(&book.description),
                price_cents: Some(book.price_cents),
                page_count: Some(book.page_count),
            })
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(result.book))
    }

    async fn delete_book(
        &self,
        request: Request<DeleteBookRequest>,
    ) -> Result<Response<()>, Status> {
        let request = ParsedDeleteBookRequest::parse(request.into_inner())
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        self.delete_book_command
            .execute(request.id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(()))
    }

    async fn list_books(
        &self,
        request: Request<ListBooksRequest>,
    ) -> Result<Response<ListBooksResponse>, Status> {
        let request = ParsedListBooksRequest::parse_list_query(
            request.into_inner(),
            &self.book_query_manager.list_query_builder(),
        )
        .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let result = self
            .book_query_manager
            .query_list(request.query, request.show_deleted)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(ListBooksResponse {
            books: result.books,
            next_page_token: result.next_page_token,
            total_size: result.total_size,
        }))
    }

    async fn search_books(
        &self,
        _request: Request<SearchBooksRequest>,
    ) -> Result<Response<SearchBooksResponse>, Status> {
        Err(Status::unimplemented("SearchBooks not yet implemented"))
    }
}
