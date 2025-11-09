use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use bomboni_request::{query::list::ListQuery, value::Value};
use bookstore_api::model::book::BookId;
use itertools::Itertools;
use tokio::sync::RwLock;

use crate::{
    book::repository::{
        BookRecordInsert, BookRecordList, BookRecordOwned, BookRecordUpdate, BookRepository,
    },
    error::AppResult,
};

/// In-memory implementation of the book repository.
#[derive(Debug)]
pub struct MemoryBookRepository {
    books: Arc<RwLock<HashMap<BookId, BookRecordOwned>>>,
}

impl Default for MemoryBookRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBookRepository {
    /// Creates a new empty memory book repository.
    pub fn new() -> Self {
        Self {
            books: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new memory book repository with initial data.
    ///
    /// # Arguments
    ///
    /// * `books` - Initial books to populate the repository with
    pub fn with_data(books: Vec<BookRecordOwned>) -> Self {
        Self {
            books: Arc::new(RwLock::new(
                books.into_iter().map(|book| (book.id, book)).collect(),
            )),
        }
    }
}

#[async_trait]
impl BookRepository for MemoryBookRepository {
    async fn insert(&self, record: BookRecordInsert) -> AppResult<()> {
        self.books.write().await.insert(
            record.id,
            BookRecordOwned {
                id: record.id,
                create_time: record.create_time,
                update_time: None,
                delete_time: None,
                deleted: false,
                display_name: record.display_name,
                author_id: record.author_id,
                isbn: record.isbn,
                description: record.description,
                price_cents: record.price_cents,
                page_count: record.page_count,
            },
        );
        Ok(())
    }

    async fn update(&self, update: BookRecordUpdate<'_>) -> AppResult<bool> {
        let mut books = self.books.write().await;
        if let Some(book) = books.get_mut(update.id) {
            if let Some(display_name) = update.display_name {
                book.display_name = display_name.to_string();
            }
            if let Some(author_id) = update.author_id {
                book.author_id = author_id;
            }
            if let Some(isbn) = update.isbn {
                book.isbn = isbn.to_string();
            }
            if let Some(description) = update.description {
                book.description = description.to_string();
            }
            if let Some(price_cents) = update.price_cents {
                book.price_cents = price_cents;
            }
            if let Some(page_count) = update.page_count {
                book.page_count = page_count;
            }
            if let Some(update_time) = update.update_time {
                book.update_time = Some(update_time);
            }
            if let Some(delete_time) = update.delete_time {
                book.delete_time = Some(delete_time);
            }
            if let Some(deleted) = update.deleted {
                book.deleted = deleted;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn select(&self, id: &BookId) -> AppResult<Option<BookRecordOwned>> {
        let books = self.books.read().await;
        Ok(books.get(id).cloned())
    }

    async fn select_multiple(&self, ids: &[BookId]) -> AppResult<Vec<BookRecordOwned>> {
        let books = self.books.read().await;
        Ok(books
            .values()
            .filter(|book| ids.contains(&book.id))
            .cloned()
            .collect())
    }

    async fn select_filtered(
        &self,
        query: &ListQuery,
        show_deleted: bool,
    ) -> AppResult<BookRecordList> {
        let books = self.books.read().await;

        let mut matched_books: Vec<_> = books
            .values()
            .filter(|book| {
                (!book.deleted || show_deleted)
                    && if query.filter.is_empty() {
                        true
                    } else if let Some(Value::Boolean(value)) = query.filter.evaluate(*book) {
                        value
                    } else {
                        false
                    }
            })
            .sorted_unstable_by(|a, b| query.ordering.evaluate(*a, *b).unwrap())
            .take(query.page_size as usize + 1)
            .cloned()
            .collect();

        let next_item = if matched_books.len() > query.page_size as usize {
            Some(matched_books.remove(matched_books.len() - 1))
        } else {
            None
        };

        Ok(BookRecordList {
            items: matched_books,
            next_item,
            total_size: books.len() as i64,
        })
    }
}
