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
#[parse(source = UpdateBookRequest, request, write)]
pub struct ParsedUpdateBookRequest {
    #[parse(source = "book?.name", convert = book_id_convert)]
    pub id: BookId,
    // #[parse(derive = update_display_name_derive)]
    // pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Parse)]
#[parse(source = DeleteBookRequest, request, write)]
pub struct ParsedDeleteBookRequest {
    #[parse(source = "name", convert = book_id_convert)]
    pub id: BookId,
}

// mod update_display_name_derive {
//     use super::*;

//     pub fn parse(request: &UpdateModuleRequest) -> RequestResult<Option<String>> {
//         let module = request.module.as_ref().ok_or_else(|| {
//             RequestError::field(
//                 UpdateModuleRequest::MODULE_FIELD_NAME,
//                 CommonError::RequiredFieldMissing,
//             )
//         })?;
//         Ok(
//             if matches!(request.update_mask.as_ref(), Some(mask) if mask.masks(Module::DISPLAY_NAME_FIELD_NAME))
//             {
//                 Some(module.display_name.clone())
//             } else {
//                 None
//             },
//         )
//     }

//     pub fn write(request: &ParsedUpdateModuleRequest, source: &mut UpdateModuleRequest) {
//         if let Some(display_name) = &request.display_name {
//             source
//                 .module
//                 .get_or_insert_with(|| Module::default())
//                 .display_name = display_name.clone();
//         }
//     }
// }

// mod update_graph_derive {
//     use super::*;

//     pub fn parse(request: &UpdateModuleRequest) -> RequestResult<Option<ParsedGraph>> {
//         let module = request.module.as_ref().ok_or_else(|| {
//             RequestError::field(
//                 UpdateModuleRequest::MODULE_FIELD_NAME,
//                 CommonError::RequiredFieldMissing,
//             )
//         })?;
//         Ok(
//             if matches!(request.update_mask.as_ref(), Some(mask) if mask.masks(Module::GRAPH_FIELD_NAME))
//             {
//                 Some(
//                     module
//                         .graph
//                         .as_ref()
//                         .ok_or_else(|| {
//                             RequestError::field(
//                                 Module::GRAPH_FIELD_NAME,
//                                 CommonError::RequiredFieldMissing,
//                             )
//                             .wrap_field(UpdateModuleRequest::MODULE_FIELD_NAME)
//                         })?
//                         .clone()
//                         .parse_into()
//                         .map_err(|err: RequestError| {
//                             err.wrap_field(Module::GRAPH_FIELD_NAME)
//                                 .wrap_field(UpdateModuleRequest::MODULE_FIELD_NAME)
//                         })?,
//                 )
//             } else {
//                 None
//             },
//         )
//     }

//     pub fn write(request: &ParsedUpdateModuleRequest, source: &mut UpdateModuleRequest) {
//         if let Some(graph) = &request.graph {
//             source.module.get_or_insert_with(|| Module::default()).graph =
//                 Some(graph.clone().into());
//         }
//     }
// }
