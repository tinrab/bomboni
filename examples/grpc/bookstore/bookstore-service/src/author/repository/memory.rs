use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use bomboni_request::value::Value;
use bomboni_request::query::list::ListQuery;
use bookstore_api::model::author::AuthorId;
use itertools::Itertools;
use tokio::sync::RwLock;

use crate::{
    author::repository::{
        AuthorRecordInsert, AuthorRecordList, AuthorRecordOwned, AuthorRecordUpdate,
        AuthorRepository,
    },
    error::AppResult,
};

#[derive(Debug)]
pub struct MemoryAuthorRepository {
    authors: Arc<RwLock<HashMap<AuthorId, AuthorRecordOwned>>>,
}

impl MemoryAuthorRepository {
    pub fn new() -> Self {
        MemoryAuthorRepository {
            authors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_data(authors: Vec<AuthorRecordOwned>) -> Self {
        MemoryAuthorRepository {
            authors: Arc::new(RwLock::new(
                authors
                    .into_iter()
                    .map(|author| (author.id, author))
                    .collect(),
            )),
        }
    }
}

#[async_trait]
impl AuthorRepository for MemoryAuthorRepository {
    async fn insert(&self, record: AuthorRecordInsert) -> AppResult<()> {
        let mut authors = self.authors.write().await;
        authors.insert(
            record.id.clone(),
            AuthorRecordOwned {
                id: record.id,
                create_time: record.create_time,
                update_time: None,
                delete_time: None,
                deleted: false,
                display_name: record.display_name,
            },
        );
        Ok(())
    }

    async fn update(&self, update: AuthorRecordUpdate<'_>) -> AppResult<bool> {
        let mut authors = self.authors.write().await;
        if let Some(author) = authors.get_mut(&update.id) {
            if let Some(display_name) = update.display_name {
                author.display_name = display_name.to_string();
            }
            if let Some(update_time) = update.update_time {
                author.update_time = Some(update_time);
            }
            if let Some(delete_time) = update.delete_time {
                author.delete_time = Some(delete_time);
            }
            if let Some(deleted) = update.deleted {
                author.deleted = deleted;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn select(
        &self,
        id: &bookstore_api::model::author::AuthorId,
    ) -> AppResult<Option<AuthorRecordOwned>> {
        let authors = self.authors.read().await;
        Ok(authors.get(id).cloned())
    }

    async fn select_multiple(
        &self,
        ids: &[bookstore_api::model::author::AuthorId],
    ) -> AppResult<Vec<AuthorRecordOwned>> {
        let authors = self.authors.read().await;
        Ok(authors
            .values()
            .filter(|author| ids.contains(&author.id))
            .cloned()
            .collect())
    }

    async fn select_filtered(
        &self,
        query: &ListQuery,
        show_deleted: bool,
    ) -> AppResult<AuthorRecordList> {
        let authors = self.authors.read().await;

        let mut matched_authors: Vec<_> = authors
            .values()
            .filter(|author| {
                (!author.deleted || show_deleted)
                    && if !query.filter.is_empty() {
                        if let Some(Value::Boolean(value)) = query.filter.evaluate(*author) {
                            value
                        } else {
                            false
                        }
                    } else {
                        true
                    }
            })
            .sorted_unstable_by(|a, b| query.ordering.evaluate(*a, *b).unwrap())
            .take(query.page_size as usize + 1)
            .cloned()
            .collect();

        let next_item = if matched_authors.len() > query.page_size as usize {
            Some(matched_authors.remove(matched_authors.len() - 1))
        } else {
            None
        };

        Ok(AuthorRecordList {
            items: matched_authors,
            next_item,
            total_size: authors.len() as i64,
        })
    }
}
