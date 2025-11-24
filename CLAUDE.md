# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**IMPORTANT**: This file must be kept up-to-date as the project evolves. When implementing new features, refactoring architecture, or making significant changes, update the relevant sections in this file to help future Claude Code instances understand the codebase.

## Project Overview

A terminal-based JSON log viewer specifically designed to handle structured JSON logs (e.g., from Pino logger). The application ingests JSON log data into DuckDB for powerful SQL-based filtering and analysis, with an interactive TUI (Terminal User Interface) built using Ratatui.

### Project Goals

1. **JSON Log Parsing**: Parse and ingest JSON-formatted logs (Pino-style and similar structured loggers)
2. **DuckDB Integration**: Store parsed logs in DuckDB for efficient querying and filtering
3. **Interactive TUI**: Provide a user-friendly terminal interface using Ratatui for browsing and filtering logs
4. **Easy Filtering**: Enable SQL-based filtering through DuckDB for complex log analysis

## Build and Development Commands

```bash
# Build the project
cargo build

# Build with optimizations
cargo build --release

# Run the application
cargo run

# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Run clipper (linter)
cargo clippy
```

## Architecture

The application follows a three-layer architecture:

1. **Ingestion Layer**: Parses JSON log files (Pino format and compatible JSON loggers) and extracts structured data
2. **Storage Layer**: Uses DuckDB as an embedded analytical database to store and index log entries for fast querying
3. **Presentation Layer**: Ratatui-based TUI that provides an interactive interface for viewing, searching, and filtering logs

### Ingestion Layer (Implemented)

Located in `src/ingestion/`:

- **models.rs**: Defines `JsonLog` struct that stores all log fields in a flexible `HashMap<String, serde_json::Value>` format
  - Provides helper methods: `get_timestamp_ms()`, `get_message()`, `get_level()`, `get_level_raw()`, `timestamp()`
  - Design is extensible: not tied to Pino format, can handle any JSON log structure
  - Defines `LogLevel` enum for human-friendly log levels with comparison support:
    - `Trace = 10`, `Debug = 20`, `Info = 30`, `Warn = 40`, `Error = 50`, `Fatal = 60`
    - Implements `PartialOrd` and `Ord` for filtering (e.g., `level >= LogLevel::Warn`)
    - Provides `as_str()` for display ("INFO", "ERROR", etc.) and `as_u64()` for numeric value
    - `from_u64()` converts Pino numeric levels to enum

- **parser.rs**: Contains `parse_json_line()` function
  - Parses JSON strings into `JsonLog` instances
  - Validates non-empty input and valid JSON
  - Returns `Result<JsonLog>` with proper error handling

- **reader.rs**: Provides `LogFileReader` for buffered file reading
  - Reads log files line-by-line efficiently
  - Tracks line numbers for error reporting
  - Returns `Vec<(usize, Result<JsonLog>)>` with line numbers and parse results

**Key Design Decisions:**
- Generic JSON structure (not Pino-specific) to support multiple log formats in the future
- All fields stored as JSON values for maximum flexibility
- Line-by-line processing to handle large files efficiently
- Graceful error handling: parsing errors don't stop processing of remaining lines

### Storage Layer (Implemented)

Located in `src/storage/`:

- **schema.rs**: Schema detection and table creation
  - `FieldType` enum: Represents SQL types (Text, Integer, Float, Boolean, Json)
    - `to_sql()` converts to DuckDB SQL type string
    - `merge()` handles type conflicts by promoting to more general types
  - `detect_field_type()`: Analyzes `serde_json::Value` to infer SQL type
  - `normalize_field_name()`: Maps common field name variants:
    - `msg` → `message`
    - `lvl` → `level`
    - `timestamp` → `time`
  - `SchemaBuilder`: Samples logs to detect schema
    - `analyze_log()` / `analyze_logs()`: Scan logs and track field types
    - `generate_create_table_sql()`: Generate CREATE TABLE statement
    - Merges types across samples (Integer + Float → Float, conflicts → Text)

- **database.rs**: DuckDB connection and operations
  - `LogDatabase`: Main database interface
    - `new_in_memory()`: Create in-memory database (fast, for development)
    - `new_with_file()`: Create file-based database (persistent)
    - `create_table_from_logs()`: Auto-detect schema from first N logs (default 100)
    - `insert_log()`: Insert single log entry
    - `insert_logs()`: Batch insert with transaction for efficiency
    - `count_logs()`: Get total log count
  - Automatic parameter extraction from `JsonLog` fields
  - Complex types (arrays, objects) stored as JSON strings
  - Full test coverage for core operations

**Key Design Decisions:**
- **Schema Detection**: Samples first 100 logs to infer types, adapting to any JSON structure
- **Field Normalization**: Handles common variations (msg/message, lvl/level, timestamp/time)
- **Type Merging**: Integer can promote to Float, conflicts default to Text for flexibility
- **Transaction Batching**: Batch inserts use transactions for 10-100x performance improvement
- **In-memory First**: Default to in-memory DB for speed, supports file-based for persistence
- **JSON Fallback**: Complex nested structures stored as JSON TEXT for queryability

### Error Handling

Located in `src/error.rs`:

- Uses `thiserror` for clean error type definitions with `#[error]` attributes
- Uses `rootcause` for error context chaining with `.attach()` method
- Result type: `Result<T> = std::result::Result<T, Report<LogViewerError>>`
- Error types: `FileRead`, `JsonParse`, `Database`, `InvalidLogFormat`, `TimestampError`, `Other`

#### Rootcause Error Handling Pattern

**Rootcause** (v0.10.0) is an error context library that wraps errors in a `Report<E>` type, allowing you to add contextual information as errors propagate up the call stack. This creates rich, traceable error messages that help with debugging.

**Key Concepts:**

1. **Import Pattern:**
   ```rust
   use rootcause::prelude::{Report, ResultExt};
   // Or for all exports:
   use rootcause::prelude::*;
   ```

2. **Result Type:**
   - Custom Result type: `pub type Result<T> = std::result::Result<T, Report<LogViewerError>>`
   - This wraps all errors in `Report<LogViewerError>` for context chaining

3. **Error Conversion Pattern:**
   - **CRITICAL**: Must convert foreign errors (like `std::io::Error`, `duckdb::Error`) to `LogViewerError` BEFORE using `.attach()`
   - Pattern: `.map_err(LogViewerError::from).attach("context")`
   - Example:
     ```rust
     Connection::open_in_memory()
         .map_err(LogViewerError::from)  // Convert duckdb::Error to LogViewerError
         .attach("Failed to create in-memory DuckDB connection")?;
     ```

4. **Creating New Errors:**
   - Use `Report::new()` to wrap custom errors:
     ```rust
     return Err(Report::new(LogViewerError::InvalidLogFormat(
         "Empty line".to_string(),
     )));
     ```

5. **Adding Context:**
   - `.attach(message)` - Adds static string context
   - `.attach_with(|| message)` - Adds lazy-evaluated context (useful for expensive operations like format!())
   - Examples:
     ```rust
     // Static context
     file.read().map_err(LogViewerError::from).attach("Failed to read file")?;

     // Lazy context (closure evaluated only on error)
     conn.execute(&sql, [])
         .map_err(LogViewerError::from)
         .attach_with(|| format!("Failed to execute SQL: {}", sql))?;
     ```

6. **Error Propagation:**
   - Errors automatically accumulate context as they propagate up the stack
   - Each `.attach()` call adds a new layer of context
   - Final error message shows full trace with all attached contexts

**Common Patterns in Codebase:**

```rust
// Pattern 1: Converting and attaching context to foreign errors
let conn = Connection::open_in_memory()
    .map_err(LogViewerError::from)
    .attach("Failed to create in-memory DuckDB connection")?;

// Pattern 2: Lazy context with formatted strings
self.conn.execute(&insert_sql, params)
    .map_err(LogViewerError::from)
    .attach_with(|| format!("Failed to insert log with SQL: {}", insert_sql))?;

// Pattern 3: Creating custom error reports
if trimmed.is_empty() {
    return Err(Report::new(LogViewerError::InvalidLogFormat(
        "Empty line".to_string(),
    )));
}

// Pattern 4: Manual error conversion in loops (avoiding ?)
Err(e) => {
    logs.push((line_number, Err(Report::new(LogViewerError::from(e)))));
}
```

**Why This Pattern?**

- Rootcause's `.attach()` method works on `Result<T, E>` where `E` implements a special trait
- Foreign errors (like `duckdb::Error`) don't implement this trait for `LogViewerError`
- Must convert first with `.map_err(LogViewerError::from)` to get `Result<T, LogViewerError>`
- Then `.attach()` wraps it in `Report<LogViewerError>` with added context

**Benefits:**

- Rich error messages with full context chain
- No boilerplate error handling code
- Lazy evaluation of expensive context (with `.attach_with()`)
- Automatic error conversion via `#[from]` in `thiserror`
- Beautiful error formatting with color support

### Key Components

- **JSON Parser**: Handles Pino-style JSON logs with fields like `level`, `time`, `msg`, `pid`, `hostname`, etc.
- **DuckDB Engine**: In-memory or file-based database for storing and querying log data with SQL (implemented)
- **Ratatui UI**: Terminal interface with features like log browsing, filtering, searching, and syntax highlighting (not yet implemented)

## Dependencies

### Production Dependencies
- **duckdb** (v1.4.2): Embedded analytical database with bundled binary and Parquet support for log storage/querying
- **rootcause** (v0.10.0): Error handling library with context chaining support
- **thiserror** (v2.0): Derive macro for error types with clean Display implementations
- **serde** (v1.0): Serialization/deserialization framework with derive macros
- **serde_json** (v1.0): JSON parsing and serialization
- **jiff** (v0.1): Date and time library for timestamp conversions
- **ratatui**: Terminal UI framework (to be added)

### Development Dependencies
- **insta** (v1.41): Snapshot testing library for testing SQL generation and other text output
  - Snapshots stored in `src/storage/snapshots/`
  - Use `cargo insta review` to review new/changed snapshots
  - Use `cargo insta test` to run all snapshot tests
  - Use `cargo insta review --accept` to accept all pending snapshots
  - Used for testing SQL CREATE TABLE statements to ensure schema generation is correct

## Edition

Uses Rust 2024 edition.
