# bomboni_common

Common utilities for the Bomboni library.

This crate provides essential utilities for building distributed systems and applications.

## Features

- **Unique Identifiers**: ULID-based sortable and unique ID generation supporting both random and worker-based approaches
- **UTC DateTime Handling**: A UTC-focused datetime type
- **WASM Compatible**: Full support for WebAssembly targets
- **Database Support**: PostgreSQL and MySQL integration via optional features

## Examples

### Unique IDs

```rust
use bomboni_common::id::{Id, worker::WorkerIdGenerator};
use std::str::FromStr;

// Generate a random sortable ID
let id = Id::generate();
println!("ID: {}", id);

// Generate multiple IDs
let ids = Id::generate_multiple(5);
assert_eq!(ids.len(), 5);

// Parsing IDs
let id_str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
let id: Id = id_str.parse().unwrap();

// For distributed systems where each worker needs to generate unique IDs:
let mut g = WorkerIdGenerator::new(1);
let id = g.generate();
assert_ne!(g.generate(), id);

// Generating multiple IDs at once is more efficient:
let mut g = WorkerIdGenerator::new(1);
let ids = g.generate_multiple(3);
assert_eq!(ids.len(), 3);
```

### UTC DateTime

```rust
use bomboni_common::date_time::UtcDateTime;

// Get the current time
let now = UtcDateTime::now();

// Create from Unix timestamp
let dt = UtcDateTime::from_seconds(1609459200).unwrap();
assert_eq!(dt.to_string(), "2021-01-01T00:00:00Z");

// Convert from string
let dt = "1970-01-01T00:00:01Z".parse::<UtcDateTime>().unwrap();
assert_eq!(dt.timestamp(), (1, 0));
```

## Cargo Features

- `serde`: Enable serialization with serde
- `tokio`: Enable async APIs using tokio
- `chrono`: Enable conversion with the chrono crate
- `wasm`: Enable WebAssembly support
- `postgres`: Enable PostgreSQL type conversions
- `mysql`: Enable MySQL type conversions
