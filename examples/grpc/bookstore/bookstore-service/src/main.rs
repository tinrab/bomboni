use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;
use tracing::info;

use bomboni_common::id::worker::WorkerIdGenerator;
use bookstore_api::v1::author_service_server::AuthorServiceServer;
use bookstore_service::{
    author::{
        adapter::AuthorAdapter, query_manager::AuthorQueryManager, repository::AuthorRepositoryArc,
        repository::memory::MemoryAuthorRepository,
    },
    config::AppConfig,
    error::AppResult,
};
use grpc_common::auth::{context::ContextBuilder, memory::MemoryAuthenticator};

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = AppConfig::get();

    // TODO: Set up tracing based on config

    start(&config).await?;

    Ok(())
}

async fn start(config: &AppConfig) -> AppResult<()> {
    let context_builder = ContextBuilder::new(Arc::new(MemoryAuthenticator::default()));

    let id_generator = Arc::new(Mutex::new(WorkerIdGenerator::new(
        config.node.worker_number,
    )));

    // Initialize repositories
    let author_repository: AuthorRepositoryArc = Arc::new(MemoryAuthorRepository::new());

    // Initialize query managers
    let author_query_manager = AuthorQueryManager::new(author_repository.clone());

    // Initialize adapters
    let author_adapter = AuthorAdapter::new(
        id_generator.clone(),
        context_builder.clone(),
        author_query_manager,
        author_repository.clone(),
    );

    let grpc_server = Server::builder().add_service(AuthorServiceServer::new(author_adapter));

    info!("gRPC server started at {}", config.server.grpc_address);

    grpc_server.serve(config.server.grpc_address).await?;

    Ok(())
}
