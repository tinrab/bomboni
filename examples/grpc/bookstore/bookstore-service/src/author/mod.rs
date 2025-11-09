//! Author management module.
//!
//! Provides complete CRUD operations for authors including:
//! - gRPC adapter for handling author service requests
//! - Command handlers for create, update, and delete operations
//! - Query manager for retrieving author data
//! - Repository abstraction for data persistence

/// gRPC service adapter for authors.
pub mod adapter;

/// Author creation command handler.
pub mod create_author_command;

/// Author deletion command handler.
pub mod delete_author_command;

/// Author query manager for data retrieval.
pub mod query_manager;

/// Author repository abstraction and implementations.
pub mod repository;

/// Author update command handler.
pub mod update_author_command;
