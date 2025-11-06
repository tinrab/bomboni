use tonic::transport::Server;

use bookstore_api::v1::bookstore_service_server::BookstoreServiceServer;
use bookstore_service::create_book_service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;

    let book_service = create_book_service();

    println!("Bookstore service listening on {}", addr);

    Server::builder()
        .add_service(BookstoreServiceServer::new(book_service))
        .serve(addr)
        .await?;

    Ok(())
}
