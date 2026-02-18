use bomboni::request::{derive::Parse, query::list::ListQuery};

use crate::{
    model::author::{AuthorId, author_id_convert},
    v1::{
        CreateAuthorRequest, DeleteAuthorRequest, GetAuthorRequest, ListAuthorsRequest,
        UpdateAuthorRequest,
    },
};

/// Parsed get author request.
///
/// This structure represents a parsed get author request with the author ID
/// extracted from the resource name.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = GetAuthorRequest, request, write)]
pub struct ParsedGetAuthorRequest {
    /// The ID of the author to retrieve.
    #[parse(source = "name", convert = author_id_convert)]
    pub id: AuthorId,
}

/// Parsed list authors request.
///
/// This structure represents a parsed list authors request with query parameters
/// and options for including deleted authors.
#[derive(Debug, Clone, PartialEq, Parse)]
#[parse(source = ListAuthorsRequest, request, write, )]
pub struct ParsedListAuthorsRequest {
    /// Query parameters for filtering and pagination.
    #[parse(list_query)]
    pub query: ListQuery,
    /// Whether to include deleted authors in the results.
    #[parse(extract = [UnwrapOrDefault])]
    pub show_deleted: bool,
}

/// Parsed create author request.
///
/// This structure represents a parsed create author request with the required
/// fields for creating a new author.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = CreateAuthorRequest, request, write)]
pub struct ParsedCreateAuthorRequest {
    /// The display name of the author to create.
    pub display_name: String,
}

/// Parsed update author request.
///
/// This structure represents a parsed update author request with the author ID
/// and optional fields to update based on the field mask.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = UpdateAuthorRequest, request)]
pub struct ParsedUpdateAuthorRequest {
    /// The ID of the author to update.
    #[parse(source = "author?.name", convert = author_id_convert)]
    pub id: AuthorId,
    /// The new display name if included in the field mask.
    #[parse(source = "author?.display_name", field_mask)]
    pub display_name: Option<String>,
}

/// Parsed delete author request.
///
/// This structure represents a parsed delete author request with the author ID
/// extracted from the resource name.
#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = DeleteAuthorRequest, request, write)]
pub struct ParsedDeleteAuthorRequest {
    /// The ID of the author to delete.
    #[parse(source = "name", convert = author_id_convert)]
    pub id: AuthorId,
}
