# Bookstore Example

A gRPC bookstore service demonstrating the bomboni framework's capabilities.

The bookstore example showcases a complete gRPC service implementation with:

```
bookstore/
├── bookstore-api/     # Protocol definitions and client/server types
├── bookstore-service/ # gRPC server implementation
└── bookstore-cli/     # Command-line client tool
```

## Quick Start

```sh
cd examples/grpc/bookstore/bookstore-service
cargo run
```

The service will start on `127.0.0.1:9000` with the following features:
- gRPC server with reflection enabled
- JWT authentication (secret: `test_secret_key`)
- In-memory data storage
- Trace logging to stdout

The service uses JWT authentication for write operations (create, update, delete).
Read operations (list, get) work without authentication.

The service requires JWT authentication for write operations.
Generate a token first:

```sh
cd examples/grpc/bookstore/bookstore-cli

# Generate a JWT token (uses default test secret)
TOKEN=$(cargo run -- auth generate-token --quiet)

# Or generate with custom values
cargo run -- auth generate-token --user-id 12345 --email "user@example.com" --expires-in-hours 24

# Export the token for subsequent commands
export BOOKSTORE_TOKEN=$TOKEN
```

The CLI provides an easy way to interact with the service:

```sh
# List all authors (read operation - no auth required)
cargo run -- author list

# Create a new author (write operation - requires auth)
cargo run -- author create "J.R.R. Tolkien" "tolkien@example.com"

# Create a book for the author (write operation - requires auth)
cargo run -- book create "The Hobbit" "authors/1" "A fantasy adventure novel"

# List all books (read operation - no auth required)
cargo run -- book list

# Get specific book details (read operation - no auth required)
cargo run -- book get books/1

# Or use the token explicitly
cargo run -- --token "$TOKEN" author create "George Orwell" "george@example.com"
```
