use bomboni_request::{derive::Parse, query::list::ListQuery};

use crate::{
    model::{
        author::{AuthorId, author_id_convert},
        book::{BookId, book_id_convert},
    },
    v1::{
        CreateBookRequest, DeleteBookRequest, GetBookRequest, ListBooksRequest, UpdateBookRequest,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Parse)]
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
    #[parse(extract = [UnwrapOrDefault])]
    pub show_deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = CreateBookRequest, request, write)]
pub struct ParsedCreateBookRequest {
    pub display_name: String,
    #[parse(source = "author", convert = author_id_convert)]
    pub author_id: AuthorId,
    pub isbn: String,
    pub description: String,
    pub price_cents: i64,
    pub page_count: i32,
}

#[derive(Debug, Clone, PartialEq, Parse)]
#[parse(source = UpdateBookRequest, request)]
pub struct ParsedUpdateBookRequest {
    #[parse(source = "book?.name", convert = book_id_convert)]
    pub id: BookId,
    #[parse(source = "book?.display_name", field_mask)]
    pub display_name: Option<String>,
    #[parse(
        source = "book?.author",
        convert = author_id_convert,
        field_mask
    )]
    pub author_id: Option<AuthorId>,
    #[parse(source = "book?.isbn", field_mask)]
    pub isbn: Option<String>,
    #[parse(source = "book?.description", field_mask)]
    pub description: Option<String>,
    #[parse(source = "book?.price_cents", field_mask)]
    pub price_cents: Option<i64>,
    #[parse(source = "book?.page_count", field_mask)]
    pub page_count: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = DeleteBookRequest, request, write)]
pub struct ParsedDeleteBookRequest {
    #[parse(source = "name", convert = book_id_convert)]
    pub id: BookId,
}
