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

### Key Components

- **JSON Parser**: Handles Pino-style JSON logs with fields like `level`, `time`, `msg`, `pid`, `hostname`, etc.
- **DuckDB Engine**: In-memory or file-based database for storing and querying log data with SQL
- **Ratatui UI**: Terminal interface with features like log browsing, filtering, searching, and syntax highlighting

## Dependencies

- **duckdb** (v1.4.2): Embedded analytical database with bundled binary and Parquet support for log storage/querying
- **rootcause** (v0.10.0): Error handling library with std features
- **ratatui**: Terminal UI framework (to be added)

## Edition

Uses Rust 2024 edition.
