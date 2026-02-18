# bomboni: `request_derive`

Provides derive macros and procedural macros for request parsing and type conversion.

This crate offers several macros for working with data conversion and parsing:

- `Parse` derive macro for converting between different data representations
- `parse_resource_name!` macro for parsing Google-style resource names
- `derived_map!` macro for generating derived mappings

## Features

- Automatic field mapping and conversion
- Support for complex nested structures
- Custom conversion functions
- Resource field parsing (name, timestamps, etag)
- Query parameter parsing (list and search)
- Enum and oneof field handling
- Timestamp and wrapper types support
- Bidirectional conversion support
- Serde integration
- Google AIP-122 resource name parsing
