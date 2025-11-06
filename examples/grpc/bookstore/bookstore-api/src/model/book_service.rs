use bomboni_request::{derive::Parse, query::list::ListQuery};

use crate::{
    model::book::{BookId, book_id_convert},
    v1::{CreateBookRequest, DeleteBookRequest, GetBookRequest, ListBooksRequest},
};

#[derive(Debug, Clone, PartialEq, Parse)]
#[parse(source = GetBookRequest, request, write)]
pub struct ParsedGetBookRequest {
    #[parse(source = "name", convert = book_id_convert)]
    pub id: BookId,
}

#[derive(Debug, Clone, PartialEq, Parse)]
#[parse(source = ListBooksRequest, request, write, )]
pub struct ParsedListBooksRequest {
    #[parse(list_query)]
    pub query: ListQuery,
    #[parse(source = "book", convert = book_id_convert)]
    pub book_id: BookId,
    #[parse(extract = [UnwrapOrDefault])]
    pub show_deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Parse)]
#[parse(source = CreateBookRequest, request, write)]
pub struct ParsedCreateBookRequest {
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub price_cents: i64,
    pub page_count: i32,
}

#[derive(Debug, Clone, PartialEq, Parse)]
#[parse(source = DeleteBookRequest, request, write)]
pub struct ParsedDeleteBookRequest {
    #[parse(source = "name", convert = book_id_convert)]
    pub id: BookId,
}
