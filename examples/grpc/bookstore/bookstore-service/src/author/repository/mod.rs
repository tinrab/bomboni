use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use bomboni_common::date_time::UtcDateTime;
use bomboni_request::{
    parse::ParsedResource, query::list::ListQuery, schema::SchemaMapped, value::Value,
};
use bookstore_api::{
    model::author::{AuthorId, AuthorModel},
    v1::Author,
};

use crate::error::AppResult;

pub mod memory;

#[derive(Debug)]
pub struct AuthorRecordInsert {
    pub id: AuthorId,
    pub create_time: UtcDateTime,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub struct AuthorRecordOwned {
    pub id: AuthorId,
    pub create_time: UtcDateTime,
    pub update_time: Option<UtcDateTime>,
    pub delete_time: Option<UtcDateTime>,
    pub deleted: bool,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub struct AuthorRecordList {
    pub items: Vec<AuthorRecordOwned>,
    pub next_item: Option<AuthorRecordOwned>,
    pub total_size: i64,
}

pub struct AuthorRecordUpdate<'a> {
    pub id: AuthorId,
    pub update_time: Option<UtcDateTime>,
    pub delete_time: Option<UtcDateTime>,
    pub deleted: Option<bool>,
    pub display_name: Option<&'a str>,
}

#[async_trait]
pub trait AuthorRepository: Debug {
    async fn insert(&self, record: AuthorRecordInsert) -> AppResult<()>;
    async fn update(&self, update: AuthorRecordUpdate<'_>) -> AppResult<bool>;
    async fn select(&self, id: &AuthorId) -> AppResult<Option<AuthorRecordOwned>>;
    async fn select_multiple(&self, ids: &[AuthorId]) -> AppResult<Vec<AuthorRecordOwned>>;
    async fn select_filtered(
        &self,
        query: &ListQuery,
        show_deleted: bool,
    ) -> AppResult<AuthorRecordList>;
}

pub type AuthorRepositoryArc = Arc<dyn AuthorRepository + Send + Sync>;

impl SchemaMapped for AuthorRecordOwned {
    fn get_field(&self, name: &str) -> Value {
        match name {
            "id" => self.id.0.to_string().into(),
            "display_name" => self.display_name.clone().into(),
            _ => unimplemented!("SchemaMapped for AuthorRecordOwned::{}", name),
        }
    }
}

impl From<AuthorModel> for AuthorRecordOwned {
    fn from(author: AuthorModel) -> Self {
        AuthorRecordOwned {
            id: author.id,
            create_time: author
                .resource
                .create_time
                .unwrap_or_else(|| UtcDateTime::now()),
            update_time: author.resource.update_time,
            delete_time: author.resource.delete_time,
            deleted: author.resource.deleted,
            display_name: author.display_name,
        }
    }
}

impl From<AuthorRecordOwned> for AuthorModel {
    fn from(record: AuthorRecordOwned) -> Self {
        AuthorModel {
            resource: ParsedResource {
                name: record.id.to_name(),
                create_time: Some(record.create_time.into()),
                update_time: record.update_time.map(Into::into),
                delete_time: record.delete_time.map(Into::into),
                deleted: record.deleted,
                etag: None,
                revision_id: None,
                revision_create_time: None,
            },
            id: record.id,
            display_name: record.display_name,
        }
    }
}

impl From<AuthorRecordOwned> for Author {
    fn from(record: AuthorRecordOwned) -> Self {
        AuthorModel::from(record).into()
    }
}
