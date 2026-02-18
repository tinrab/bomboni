# bomboni_proto

Utilities for working with Protobuf/gRPC. Part of Bomboni library.

This crate provides enhanced implementations of Google's well-known protobuf types with additional functionality beyond the standard `prost-types`.
It includes comprehensive support for serialization, conversion utilities, and WebAssembly compatibility.

## Features

- **Enhanced Well-Known Types**: Improved implementations of Google protobuf types with additional methods
- **Any Type Support**: Easy conversion between protobuf messages and `Any` type with type URL handling
- **RPC Status Types**: Complete Google RPC error handling with status codes and details
- **Field Mask Support**: Utilities for working with protobuf field masks
- **Struct Value Support**: Enhanced struct and value type handling
- **Serde Integration**: Comprehensive serialization/deserialization support for all types
- **WASM Support**: Full WebAssembly compatibility with JavaScript bindings

## Examples

### Any Type Conversions

```rust
use bomboni_proto::google::protobuf::Any;
use bomboni_proto::google::rpc::ErrorInfo;

// Convert protobuf messages to Any type
let msg = ErrorInfo {
    reason: "test".to_string(),
    domain: "example".to_string(),
    metadata: Default::default(),
};
let any_msg = Any::from_msg(&msg).unwrap();

// Convert back from Any to original type
let decoded: ErrorInfo = any_msg.to_msg().unwrap();
assert_eq!(decoded.reason, "test");
```

### RPC Status Handling

```rust
use bomboni_proto::google::rpc::{Status, Code, BadRequest, bad_request::FieldViolation};
use bomboni_proto::google::protobuf::Any;

// Create status with error details
let bad_request = BadRequest {
    field_violations: vec![
        FieldViolation {
            field: "name".to_string(),
            description: "Required field".to_string(),
        }
    ],
};

let status = Status::new(
    Code::InvalidArgument,
    "Invalid request".to_string(),
    vec![Any::from_msg(&bad_request).unwrap()],
);

// Convert to/from tonic status (with tonic feature)
#[cfg(feature = "tonic")]
{
    use tonic::Status;
    
    let tonic_status = Status::try_from(status).unwrap();
    let converted_back = Status::try_from(tonic_status).unwrap();
}
```

### Field Mask Operations

```rust
use bomboni_proto::google::protobuf::FieldMask;

// Create field masks
let mask = FieldMask {
    paths: vec!["user.name".to_string(), "user.email".to_string()],
};

// Field masks are commonly used in update operations
// to specify which fields should be updated
```

### Error Details with Serde

```rust
use bomboni_proto::google::rpc::ErrorInfo;
use serde_json;

// Error details support serde serialization
let error_info = ErrorInfo {
    reason: "INVALID_ARGUMENT".to_string(),
    domain: "my.api".to_string(),
    metadata: Default::default(),
};

let json = serde_json::to_string(&error_info).unwrap();
let parsed: ErrorInfo = serde_json::from_str(&json).unwrap();
```

## Cargo Features

- `testing`: Enable testing utilities
- `tonic`: Enable integration with tonic gRPC library
- `chrono`: Enable compatibility with chrono datetime library
- `wasm`: Enable WebAssembly support and JavaScript bindings
- `js`: Enable JavaScript-specific type mappings

## Protocol Buffer Files

This crate includes and enhances the following Google protobuf files:

- `google/protobuf/any.proto` - Dynamic message types
- `google/protobuf/timestamp.proto` - Timestamp handling
- `google/protobuf/duration.proto` - Duration handling
- `google/protobuf/empty.proto` - Empty message type
- `google/protobuf/field_mask.proto` - Field mask operations
- `google/protobuf/struct.proto` - Struct and value types
- `google/protobuf/wrappers.proto` - Wrapper types for primitives
- `google/rpc/status.proto` - RPC status and error handling
- `google/rpc/code.proto` - RPC status codes
- `google/rpc/error_details.proto` - Detailed error information
