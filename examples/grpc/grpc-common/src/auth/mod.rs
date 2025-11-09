//! Authentication utilities for gRPC services.
//!
//! This module provides types and traits for handling JWT-based authentication,
//! including access token models, authenticators, and context management.

/// Access token models and related types.
pub mod access_token;

/// Authentication traits and utilities.
pub mod authenticator;

/// Context management for authentication.
pub mod context;

/// JWT-based authenticator implementation.
pub mod jwt_authenticator;

/// In-memory authenticator for testing.
pub mod memory;
