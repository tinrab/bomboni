use bomboni_request::{derive::Parse, query::list::ListQuery};

use crate::{
    model::author::{AuthorId, author_id_convert},
    v1::{CreateAuthorRequest, DeleteAuthorRequest, GetAuthorRequest, ListAuthorsRequest},
};

#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = GetAuthorRequest, request, write)]
pub struct ParsedGetAuthorRequest {
    #[parse(source = "name", convert = author_id_convert)]
    pub id: AuthorId,
}

#[derive(Debug, Clone, PartialEq, Parse)]
#[parse(source = ListAuthorsRequest, request, write, )]
pub struct ParsedListAuthorsRequest {
    #[parse(list_query)]
    pub query: ListQuery,
    #[parse(extract = [UnwrapOrDefault])]
    pub show_deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = CreateAuthorRequest, request, write)]
pub struct ParsedCreateAuthorRequest {
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = DeleteAuthorRequest, request, write)]
pub struct ParsedDeleteAuthorRequest {
    #[parse(source = "name", convert = author_id_convert)]
    pub id: AuthorId,
}
