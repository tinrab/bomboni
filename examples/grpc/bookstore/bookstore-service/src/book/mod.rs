//! Book management module.
//!
//! Provides complete CRUD operations for books including:
//! - gRPC adapter for handling book service requests
//! - Command handlers for create, update, and delete operations
//! - Query manager for retrieving book data
//! - Repository abstraction for data persistence

/// gRPC service adapter for books.
pub mod adapter;

/// Book creation command handler.
pub mod create_book_command;

/// Book deletion command handler.
pub mod delete_book_command;

/// Book query manager for data retrieval.
pub mod query_manager;

/// Book repository abstraction and implementations.
pub mod repository;

/// Book update command handler.
pub mod update_book_command;
