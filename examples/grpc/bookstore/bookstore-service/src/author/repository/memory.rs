use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use bomboni_request::{query::list::ListQuery, value::Value};
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

/// In-memory implementation of the author repository.
#[derive(Debug)]
pub struct MemoryAuthorRepository {
    authors: Arc<RwLock<HashMap<AuthorId, AuthorRecordOwned>>>,
}

impl Default for MemoryAuthorRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryAuthorRepository {
    /// Creates a new empty memory repository.
    pub fn new() -> Self {
        Self {
            authors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new memory repository with initial data.
    ///
    /// # Arguments
    ///
    /// * `authors` - Initial list of authors to populate the repository
    pub fn with_data(authors: Vec<AuthorRecordOwned>) -> Self {
        Self {
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
        self.authors.write().await.insert(
            record.id,
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

    async fn select(&self, id: &AuthorId) -> AppResult<Option<AuthorRecordOwned>> {
        let authors = self.authors.read().await;
        Ok(authors.get(id).cloned())
    }

    async fn select_multiple(&self, ids: &[AuthorId]) -> AppResult<Vec<AuthorRecordOwned>> {
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
                    && if query.filter.is_empty() {
                        true
                    } else if let Some(Value::Boolean(value)) = query.filter.evaluate(*author) {
                        value
                    } else {
                        false
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
