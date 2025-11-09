use bomboni::request::{derive::Parse, query::list::ListQuery};

use crate::{
    model::{
        author::{AuthorId, author_id_convert},
        book::{BookId, book_id_convert},
    },
    v1::{
        CreateBookRequest, DeleteBookRequest, GetBookRequest, ListBooksRequest, UpdateBookRequest,
    },
};

/// Parsed get book request.
///
/// This structure represents a parsed get book request with the book ID
/// extracted from the resource name.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = GetBookRequest, request, write)]
pub struct ParsedGetBookRequest {
    /// The ID of the book to retrieve.
    #[parse(source = "name", convert = book_id_convert)]
    pub id: BookId,
}

/// Parsed list books request.
///
/// This structure represents a parsed list books request with query parameters
/// and options for including deleted books.
#[derive(Debug, Clone, PartialEq, Parse)]
#[parse(source = ListBooksRequest, request, write, )]
pub struct ParsedListBooksRequest {
    /// Query parameters for filtering and pagination.
    #[parse(list_query)]
    pub query: ListQuery,
    /// Whether to include deleted books in the results.
    #[parse(extract = [UnwrapOrDefault])]
    pub show_deleted: bool,
}

/// Parsed create book request.
///
/// This structure represents a parsed create book request with the required
/// fields for creating a new book.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = CreateBookRequest, request, write)]
pub struct ParsedCreateBookRequest {
    /// The display title of the book to create.
    pub display_name: String,
    /// The ID of the author who wrote the book.
    #[parse(source = "author", convert = author_id_convert)]
    pub author_id: AuthorId,
    /// The ISBN number of the book.
    pub isbn: String,
    /// The description of the book.
    pub description: String,
    /// The price in cents.
    pub price_cents: i64,
    /// The number of pages in the book.
    pub page_count: i32,
}

/// Parsed update book request.
///
/// This structure represents a parsed update book request with the book ID
/// and optional fields to update based on the field mask.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = UpdateBookRequest, request)]
pub struct ParsedUpdateBookRequest {
    /// The ID of the book to update.
    #[parse(source = "book?.name", convert = book_id_convert)]
    pub id: BookId,
    /// The new display title if included in the field mask.
    #[parse(source = "book?.display_name", field_mask)]
    pub display_name: Option<String>,
    /// The new author ID if included in the field mask.
    #[parse(
        source = "book?.author",
        convert = author_id_convert,
        field_mask
    )]
    pub author_id: Option<AuthorId>,
    /// The new ISBN if included in the field mask.
    #[parse(source = "book?.isbn", field_mask)]
    pub isbn: Option<String>,
    /// The new description if included in the field mask.
    #[parse(source = "book?.description", field_mask)]
    pub description: Option<String>,
    /// The new price in cents if included in the field mask.
    #[parse(source = "book?.price_cents", field_mask)]
    pub price_cents: Option<i64>,
    /// The new page count if included in the field mask.
    #[parse(source = "book?.page_count", field_mask)]
    pub page_count: Option<i32>,
}

/// Parsed delete book request.
///
/// This structure represents a parsed delete book request with the book ID
/// extracted from the resource name.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = DeleteBookRequest, request, write)]
pub struct ParsedDeleteBookRequest {
    /// The ID of the book to delete.
    #[parse(source = "name", convert = book_id_convert)]
    pub id: BookId,
}
