pub mod book;
pub mod config;
pub mod error;

use book::{
    adapter::BookAdapter, create_book_command::CreateBookCommand,
    delete_book_command::DeleteBookCommand, query_manager::BookQueryManager,
    repository::memory::MemoryBookRepository, update_book_command::UpdateBookCommand,
};
use std::sync::Arc;

pub fn create_book_service() -> BookAdapter {
    let book_repository = Arc::new(MemoryBookRepository::new());
    let book_query_manager = BookQueryManager::new(book_repository.clone());
    let create_book_command = CreateBookCommand::new(book_repository.clone());
    let update_book_command = UpdateBookCommand::new(book_repository.clone());
    let delete_book_command = DeleteBookCommand::new(book_repository);

    BookAdapter::new(
        book_query_manager,
        create_book_command,
        update_book_command,
        delete_book_command,
    )
}
