use super::{BookRecordInsert, BookRecordList, BookRecordOwned, BookRecordUpdate, BookRepository};
use async_trait::async_trait;
use bomboni_request::{query::list::ListQuery, value::Value as FilterValue};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct MemoryBookRepository {
    books: Arc<RwLock<HashMap<super::BookId, BookRecordOwned>>>,
}

impl MemoryBookRepository {
    pub fn new() -> Self {
        Self {
            books: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_data(books: Vec<BookRecordOwned>) -> Self {
        Self {
            books: Arc::new(RwLock::new(
                books.into_iter().map(|book| (book.id, book)).collect(),
            )),
        }
    }
}

impl Default for MemoryBookRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BookRepository for MemoryBookRepository {
    async fn insert(
        &self,
        record: BookRecordInsert<'_>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut books = self.books.write().await;
        books.insert(
            record.id,
            BookRecordOwned {
                id: record.id,
                create_time: record.create_time,
                update_time: None,
                delete_time: None,
                deleted: false,
                display_name: record.display_name.to_string(),
                author: record.author.to_string(),
                isbn: record.isbn.to_string(),
                description: record.description.to_string(),
                price_cents: record.price_cents,
                page_count: record.page_count,
            },
        );
        Ok(())
    }

    async fn update(
        &self,
        update: BookRecordUpdate<'_>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut books = self.books.write().await;
        if let Some(book) = books.get_mut(&update.id) {
            if let Some(update_time) = update.update_time {
                book.update_time = Some(update_time);
            }
            if let Some(delete_time) = update.delete_time {
                book.delete_time = Some(delete_time);
            }
            if let Some(deleted) = update.deleted {
                book.deleted = deleted;
            }
            if let Some(display_name) = update.display_name {
                book.display_name = display_name.to_string();
            }
            if let Some(author) = update.author {
                book.author = author.to_string();
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
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn select(
        &self,
        id: super::BookId,
    ) -> Result<Option<BookRecordOwned>, Box<dyn std::error::Error + Send + Sync>> {
        let books = self.books.read().await;
        Ok(books.get(&id).cloned())
    }

    async fn select_filtered(
        &self,
        query: &ListQuery,
        show_deleted: bool,
    ) -> Result<BookRecordList, Box<dyn std::error::Error + Send + Sync>> {
        let books = self.books.read().await;

        let mut matched_books: Vec<BookRecordOwned> = books
            .values()
            .cloned()
            .filter(|book| {
                (!book.deleted || show_deleted)
                    && if !query.filter.is_empty() {
                        if let Some(FilterValue::Boolean(value)) = query.filter.evaluate(book) {
                            value
                        } else {
                            false
                        }
                    } else {
                        true
                    }
            })
            .collect();

        // Sort the results
        matched_books.sort_by(|a, b| {
            query
                .ordering
                .evaluate(a, b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply pagination
        let next_item = if matched_books.len() > query.page_size as usize {
            Some(matched_books.remove(matched_books.len() - 1))
        } else {
            None
        };

        Ok(BookRecordList {
            items: matched_books,
            next_item,
        })
    }
}
