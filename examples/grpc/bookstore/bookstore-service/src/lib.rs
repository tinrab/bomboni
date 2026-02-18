//! Bookstore gRPC service implementation.
//!
//! This crate provides a complete implementation of the bookstore service,
//! including author and book management, configuration handling, and error types.
//! It uses a clean architecture with adapters, repositories, and command patterns.

/// Author management module.
pub mod author;

/// Book management module.
pub mod book;

/// Configuration management.
pub mod config;

/// Error types and handling.
pub mod error;

/// Tracing and observability.
pub mod tracing;
