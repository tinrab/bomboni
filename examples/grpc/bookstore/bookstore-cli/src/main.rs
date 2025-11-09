//! Command-line interface for the Bookstore gRPC service.
//!
//! This CLI provides commands to interact with authors and books in the bookstore system.
//! It supports both JSON and text output formats and can authenticate using bearer tokens.

use anyhow::Result;
use clap::{Parser, Subcommand};
use tonic::metadata::MetadataMap;

use bookstore_api::client::BookstoreClient;

/// Command-line interface configuration.
///
/// Defines the main CLI structure including server address, authentication,
/// output format, and subcommands for authors and books.
#[derive(Parser)]
#[command(name = "bookstore-cli")]
#[command(about = "A CLI tool for interacting with Bookstore gRPC service")]
#[command(version)]
pub struct Cli {
    /// The gRPC server address
    #[arg(long, short, default_value = "http://127.0.0.1:50051")]
    pub address: String,

    /// Authentication token for API requests
    #[arg(long, short, env = "BOOKSTORE_TOKEN")]
    pub token: Option<String>,

    /// Output format (json or text)
    #[arg(long, short = 'J', env = "BOOKSTORE_JSON")]
    pub json: bool,

    /// The subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands.
///
/// Groups operations by resource type (authors or books).
#[derive(Subcommand)]
pub enum Commands {
    /// Author operations
    Author {
        /// The author subcommand to execute
        #[command(subcommand)]
        command: AuthorCommands,
    },
    /// Book operations
    Book {
        /// The book subcommand to execute
        #[command(subcommand)]
        command: BookCommands,
    },
}

/// Author-related operations.
#[derive(Subcommand)]
pub enum AuthorCommands {
    /// Get an author by ID
    Get {
        /// Author ID (e.g., authors/123 or just 123)
        #[arg(help = "Author ID (e.g., authors/123 or just 123)")]
        id: String,
    },
    /// List all authors
    List,
    /// Create a new author
    Create {
        /// Author display name
        name: String,
        /// Author email
        email: String,
    },
    /// Update an existing author
    Update {
        /// Author ID (e.g., authors/123 or just 123)
        #[arg(help = "Author ID (e.g., authors/123 or just 123)")]
        id: String,
        /// New display name
        name: String,
        /// New email
        email: String,
    },
    /// Delete an author
    Delete {
        /// Author ID (e.g., authors/123 or just 123)
        #[arg(help = "Author ID (e.g., authors/123 or just 123)")]
        id: String,
    },
}

/// Book-related operations.
#[derive(Subcommand)]
pub enum BookCommands {
    /// Get a book by ID
    Get {
        /// Book ID (e.g., books/123 or just 123)
        #[arg(help = "Book ID (e.g., books/123 or just 123)")]
        id: String,
    },
    /// List all books
    List,
    /// Create a new book
    Create {
        /// Book title
        title: String,
        /// Author ID (e.g., authors/123 or just 123)
        #[arg(help = "Author ID (e.g., authors/123 or just 123)")]
        author_id: String,
        /// Book description
        description: String,
    },
    /// Update an existing book
    Update {
        /// Book ID (e.g., books/123 or just 123)
        #[arg(help = "Book ID (e.g., books/123 or just 123)")]
        id: String,
        /// New title
        title: String,
        /// New description
        description: String,
    },
    /// Delete a book
    Delete {
        /// Book ID (e.g., books/123 or just 123)
        #[arg(help = "Book ID (e.g., books/123 or just 123)")]
        id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Create client
    let client = BookstoreClient::connect(&cli.address).await?;

    // Prepare metadata
    let mut metadata = MetadataMap::new();
    if let Some(token) = &cli.token {
        let auth_value = format!("Bearer {token}");
        metadata.insert("authorization", auth_value.parse()?);
    }

    // Execute command
    match cli.command {
        Commands::Author { command } => {
            handle_author_command(client, command, metadata, cli.json).await?;
        }
        Commands::Book { command } => {
            handle_book_command(client, command, metadata, cli.json).await?;
        }
    }

    Ok(())
}

/// Handles author-related CLI commands.
///
/// # Errors
///
/// Returns an error if the gRPC call fails or the response is invalid.
async fn handle_author_command(
    client: BookstoreClient,
    command: AuthorCommands,
    metadata: MetadataMap,
    json_output: bool,
) -> Result<()> {
    use bookstore_api::v1::{
        Author, CreateAuthorRequest, DeleteAuthorRequest, GetAuthorRequest, ListAuthorsRequest,
        UpdateAuthorRequest,
    };

    match command {
        AuthorCommands::Get { id } => {
            let normalized_id = normalize_id("authors", &id);
            let request = GetAuthorRequest {
                name: normalized_id,
            };

            let response = client.author.get_author(request, metadata).await?;
            let author = response.into_inner();

            output_author(&author, json_output);
        }

        AuthorCommands::List => {
            let request = ListAuthorsRequest {
                page_size: Some(100),
                page_token: None,
                filter: None,
                order_by: None,
                show_deleted: None,
            };

            let response = client.author.list_authors(request, metadata).await?;
            let list_response = response.into_inner();

            if json_output {
                println!("{{");
                println!("  \"authors\": [");
                for (i, author) in list_response.authors.iter().enumerate() {
                    if i > 0 {
                        println!("    ,");
                    }
                    print_author_json(author);
                }
                println!("  ],");
                println!("  \"total_size\": {}", list_response.total_size);
                println!("}}");
            } else {
                println!("Authors ({} total):", list_response.total_size);
                for author in &list_response.authors {
                    println!("  - {}: {}", author.name, author.display_name);
                }
            }
        }

        AuthorCommands::Create { name, email: _ } => {
            let request = CreateAuthorRequest { display_name: name };

            let response = client.author.create_author(request, metadata).await?;
            let author = response.into_inner();

            output_author(&author, json_output);
        }

        AuthorCommands::Update { id, name, email: _ } => {
            let normalized_id = normalize_id("authors", &id);
            let request = UpdateAuthorRequest {
                name: normalized_id.clone(),
                author: Some(Author {
                    name: normalized_id,
                    create_time: None,
                    update_time: None,
                    delete_time: None,
                    deleted: false,
                    etag: None,
                    display_name: name,
                }),
                update_mask: None,
            };

            let response = client.author.update_author(request, metadata).await?;
            let author = response.into_inner();

            output_author(&author, json_output);
        }

        AuthorCommands::Delete { id } => {
            let normalized_id = normalize_id("authors", &id);
            let request = DeleteAuthorRequest {
                name: normalized_id,
            };

            let _response = client.author.delete_author(request, metadata).await?;

            if json_output {
                println!("{{\"status\": \"deleted\"}}");
            } else {
                println!("Author deleted successfully");
            }
        }
    }

    Ok(())
}

/// Handles book-related CLI commands.
///
/// # Errors
///
/// Returns an error if the gRPC call fails or the response is invalid.
async fn handle_book_command(
    client: BookstoreClient,
    command: BookCommands,
    metadata: MetadataMap,
    json_output: bool,
) -> Result<()> {
    use bookstore_api::v1::{
        Book, CreateBookRequest, DeleteBookRequest, GetBookRequest, ListBooksRequest,
        UpdateBookRequest,
    };

    match command {
        BookCommands::Get { id } => {
            let normalized_id = normalize_id("books", &id);
            let request = GetBookRequest {
                name: normalized_id,
            };

            let response = client.book.get_book(request, metadata).await?;
            let book = response.into_inner();

            output_book(&book, json_output);
        }

        BookCommands::List => {
            let request = ListBooksRequest {
                page_size: Some(100),
                page_token: None,
                filter: None,
                order_by: None,
                show_deleted: None,
            };

            let response = client.book.list_books(request, metadata).await?;
            let list_response = response.into_inner();

            if json_output {
                println!("{{");
                println!("  \"books\": [");
                for (i, book) in list_response.books.iter().enumerate() {
                    if i > 0 {
                        println!("    ,");
                    }
                    print_book_json(book);
                }
                println!("  ],");
                println!("  \"total_size\": {}", list_response.total_size);
                println!("}}");
            } else {
                println!("Books ({} total):", list_response.total_size);
                for book in &list_response.books {
                    println!(
                        "  - {}: {} (Author: {})",
                        book.name, book.display_name, book.author
                    );
                }
            }
        }

        BookCommands::Create {
            title,
            author_id,
            description,
        } => {
            let normalized_author_id = normalize_id("authors", &author_id);
            let request = CreateBookRequest {
                display_name: title,
                author: normalized_author_id,
                isbn: String::new(),
                description,
                price_cents: 0,
                page_count: 0,
            };

            let response = client.book.create_book(request, metadata).await?;
            let book = response.into_inner();

            output_book(&book, json_output);
        }

        BookCommands::Update {
            id,
            title,
            description,
        } => {
            let normalized_id = normalize_id("books", &id);
            let request = UpdateBookRequest {
                name: normalized_id.clone(),
                book: Some(Book {
                    name: normalized_id,
                    create_time: None,
                    update_time: None,
                    delete_time: None,
                    deleted: false,
                    etag: None,
                    display_name: title,
                    author: String::new(),
                    isbn: String::new(),
                    description,
                    price_cents: 0,
                    page_count: 0,
                }),
                update_mask: None,
            };

            let response = client.book.update_book(request, metadata).await?;
            let book = response.into_inner();

            output_book(&book, json_output);
        }

        BookCommands::Delete { id } => {
            let normalized_id = normalize_id("books", &id);
            let request = DeleteBookRequest {
                name: normalized_id,
            };

            let _response = client.book.delete_book(request, metadata).await?;

            if json_output {
                println!("{{\"status\": \"deleted\"}}");
            } else {
                println!("Book deleted successfully");
            }
        }
    }

    Ok(())
}

/// Normalizes resource IDs to the full format.
///
/// Accepts both short form ("123") and full form ("authors/123") and
/// always returns the full form.
fn normalize_id(resource_type: &str, id: &str) -> String {
    if id.starts_with(&format!("{resource_type}/")) {
        id.to_string()
    } else {
        format!("{resource_type}/{id}")
    }
}

/// Outputs author information in the specified format.
fn output_author(author: &bookstore_api::v1::Author, json_output: bool) {
    if json_output {
        print_author_json(author);
        println!();
    } else {
        println!("Author Details:");
        println!("  Name: {}", author.name);
        println!("  Display Name: {}", author.display_name);
        println!("  Created: {:?}", author.create_time);
        println!("  Updated: {:?}", author.update_time);
        println!("  Deleted: {}", author.deleted);
    }
}

/// Outputs book information in the specified format.
fn output_book(book: &bookstore_api::v1::Book, json_output: bool) {
    if json_output {
        print_book_json(book);
        println!();
    } else {
        println!("Book Details:");
        println!("  Name: {}", book.name);
        println!("  Title: {}", book.display_name);
        println!("  Author: {}", book.author);
        println!("  Description: {}", book.description);
        println!("  ISBN: {}", book.isbn);
        println!("  Price (cents): {}", book.price_cents);
        println!("  Pages: {}", book.page_count);
        println!("  Created: {:?}", book.create_time);
        println!("  Updated: {:?}", book.update_time);
        println!("  Deleted: {}", book.deleted);
    }
}

/// Prints author information in JSON format.
fn print_author_json(author: &bookstore_api::v1::Author) {
    println!("    {{");
    println!("      \"name\": \"{}\",", author.name);
    println!("      \"display_name\": \"{}\",", author.display_name);
    println!("      \"deleted\": {},", author.deleted);
    println!("      \"create_time\": {:?},", author.create_time);
    println!("      \"update_time\": {:?}", author.update_time);
    println!("    }}");
}

/// Prints book information in JSON format.
fn print_book_json(book: &bookstore_api::v1::Book) {
    println!("    {{");
    println!("      \"name\": \"{}\",", book.name);
    println!("      \"display_name\": \"{}\",", book.display_name);
    println!("      \"author\": \"{}\",", book.author);
    println!("      \"description\": \"{}\",", book.description);
    println!("      \"isbn\": \"{}\",", book.isbn);
    println!("      \"price_cents\": {},", book.price_cents);
    println!("      \"page_count\": {},", book.page_count);
    println!("      \"deleted\": {},", book.deleted);
    println!("      \"create_time\": {:?},", book.create_time);
    println!("      \"update_time\": {:?}", book.update_time);
    println!("    }}");
}
