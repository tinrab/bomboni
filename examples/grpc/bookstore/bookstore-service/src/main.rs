//! Bookstore gRPC Service
//!
//! A gRPC service implementation for managing books and authors.
//! Provides RESTful-like operations through gRPC with support for
//! authentication, pagination, filtering, and ordering.
//!
//! ## Features
//!
//! - Book and author management
//! - JWT and memory-based authentication
//! - Pagination and filtering
//! - gRPC reflection support
//! - Structured logging and tracing

use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;
use tracing::info;

use bomboni_common::id::worker::WorkerIdGenerator;
use bookstore_api::v1::{
    FILE_DESCRIPTOR_SET, author_service_server::AuthorServiceServer,
    bookstore_service_server::BookstoreServiceServer,
};
use bookstore_service::{
    author::{
        adapter::AuthorAdapter,
        query_manager::AuthorQueryManager,
        repository::{AuthorRepositoryArc, memory::MemoryAuthorRepository},
    },
    book::{
        adapter::BookAdapter,
        query_manager::BookQueryManager,
        repository::{BookRepositoryArc, memory::MemoryBookRepository},
    },
    config::{AppConfig, AuthConfig},
    error::AppResult,
    tracing::tracer::Tracer,
};
use grpc_common::auth::{
    authenticator::AuthenticatorArc, context::ContextBuilder, jwt_authenticator::JwtAuthenticator,
    memory::MemoryAuthenticator,
};

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = AppConfig::get();
    info!(
        "Starting {} v{}",
        config.distribution.name,
        config.distribution.version.as_ref().unwrap(),
    );

    Tracer::install_stdout()?;

    start(config).await?;

    Ok(())
}

async fn start(config: &AppConfig) -> AppResult<()> {
    let authenticator: AuthenticatorArc = match &config.auth {
        AuthConfig::Memory => Arc::new(MemoryAuthenticator::default()),
        AuthConfig::Jwt(jwt_config) => {
            if jwt_config.validate_expiration.unwrap_or(true) {
                Arc::new(JwtAuthenticator::new(&jwt_config.secret))
            } else {
                Arc::new(JwtAuthenticator::new_no_validation(&jwt_config.secret))
            }
        }
    };

    let context_builder = ContextBuilder::new(authenticator);

    let id_generator = Arc::new(Mutex::new(WorkerIdGenerator::new(
        config.node.worker_number,
    )));

    let author_repository: AuthorRepositoryArc = Arc::new(MemoryAuthorRepository::new());
    let book_repository: BookRepositoryArc = Arc::new(MemoryBookRepository::new());

    let author_query_manager = AuthorQueryManager::new(Arc::clone(&author_repository));
    let book_query_manager = BookQueryManager::new(Arc::clone(&book_repository));

    let author_adapter = AuthorAdapter::new(
        Arc::clone(&id_generator),
        context_builder.clone(),
        author_query_manager,
        Arc::clone(&author_repository),
    );

    let book_adapter = BookAdapter::new(
        context_builder.clone(),
        book_query_manager,
        Arc::clone(&book_repository),
        Arc::clone(&id_generator),
    );

    let grpc_server = Server::builder()
        .add_service(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                // Some clients only work with v1alpha
                .build_v1alpha()
                .unwrap(),
        )
        .add_service(AuthorServiceServer::new(author_adapter))
        .add_service(BookstoreServiceServer::new(book_adapter));

    info!("gRPC server started at {}", config.server.grpc_address);

    grpc_server.serve(config.server.grpc_address).await?;

    Ok(())
}
