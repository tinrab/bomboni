# bomboni_fs

File system utilities for the Bomboni library.

This crate provides utilities for working with files and directories, including recursive file visiting and content reading with extension filtering.

## Examples

### Recursive File Visiting

```rust,ignore
use bomboni_fs::visit_files;
use std::error::Error;
use std::fs::DirEntry;

// Find all Rust files in a directory
let mut rust_files = Vec::new();
visit_files("src", &["rs"], &mut |entry: &DirEntry| {
    rust_files.push(entry.path().to_path_buf());
    Ok(())
})?;

println!("Found {} Rust files", rust_files.len());
```

### Reading File Contents During Traversal

```rust,ignore
use bomboni_fs::visit_files_contents;
use std::error::Error;
use std::fs::DirEntry;

// Read all Markdown files and their contents
let mut documents = Vec::new();
visit_files_contents("docs", &["md"], &mut |entry: &DirEntry, content: String| {
    documents.push((
        entry.path().to_path_buf(),
        content.lines().count().to_string()
    ));
    Ok(())
})?;

for (path, line_count) in documents {
    println!("{}: {} lines", path.display(), line_count);
}
```

### Multiple Extension Filtering

```rust,ignore
use bomboni_fs::visit_files;
use std::error::Error;
use std::fs::DirEntry;

// Find both source and header files
let mut source_files = Vec::new();
visit_files("project", &["rs", "toml", "yaml", "yml"], &mut |entry: &DirEntry| {
    source_files.push(entry.file_name().to_string_lossy().to_string());
    Ok(())
})?;

println!("Source files: {:?}", source_files);
```
