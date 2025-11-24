# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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

### Error Handling

Located in `src/error.rs`:

- Uses `thiserror` for clean error type definitions with `#[error]` attributes
- Uses `rootcause` for error context chaining with `.attach()` method
- Result type: `Result<T> = std::result::Result<T, Report<LogViewerError>>`
- Error types: `FileRead`, `JsonParse`, `Database`, `InvalidLogFormat`, `TimestampError`, `Other`

### Key Components

- **JSON Parser**: Handles Pino-style JSON logs with fields like `level`, `time`, `msg`, `pid`, `hostname`, etc.
- **DuckDB Engine**: In-memory or file-based database for storing and querying log data with SQL (not yet implemented)
- **Ratatui UI**: Terminal interface with features like log browsing, filtering, searching, and syntax highlighting (not yet implemented)

## Dependencies

- **duckdb** (v1.4.2): Embedded analytical database with bundled binary and Parquet support for log storage/querying
- **rootcause** (v0.10.0): Error handling library with context chaining support
- **thiserror** (v2.0): Derive macro for error types with clean Display implementations
- **serde** (v1.0): Serialization/deserialization framework with derive macros
- **serde_json** (v1.0): JSON parsing and serialization
- **jiff** (v0.1): Date and time library for timestamp conversions
- **ratatui**: Terminal UI framework (to be added)

## Edition

Uses Rust 2024 edition.
