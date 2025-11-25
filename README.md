# Log Viewer

A blazingly fast terminal-based JSON log viewer with SQL-powered filtering, built in Rust.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-2024-orange.svg)
![Vibe](https://img.shields.io/badge/coded_with-✨_vibes-ff69b4.svg)

> ⚠️ **Disclaimer**: This project was created through vibe coding with AI assistance. While it works and is functional, expect some quirks and unconventional patterns. PRs welcome! 🎨✨

## Features

- 🚀 **Fast**: Loads all logs into an in-memory DuckDB database for instant filtering
- 🔍 **SQL Filtering**: Use powerful SQL WHERE clauses to filter logs
- ⌨️ **Vim Keybindings**: Navigate logs efficiently with familiar vim-style controls
- 🎨 **Color-Coded Levels**: Visual distinction between TRACE, DEBUG, INFO, WARN, ERROR, and FATAL
- 📊 **Schema Detection**: Automatically detects field types from your JSON logs
- 🔧 **Debug Panel**: Built-in tracing debug panel for troubleshooting (toggle with 'L')
- 🎯 **Preset Filters**: Quick access to common filters (Errors Only, Warnings+, Last Hour)

## Installation

### Using Cargo

```bash
# Install directly from GitHub
cargo install --git https://github.com/DanSnow/log-viewer.git
```

**Note**: The first build will take 15-20 minutes due to DuckDB compilation.

## Usage

### Basic Usage

```bash
# View a JSON log file
log-viewer /path/to/logs.json
```

### Supported Log Formats

The viewer works with any JSON-formatted logs, especially those from structured loggers like:

- **Pino** (Node.js)
- Any logger that outputs JSON lines

Example log format:
```json
{"level":30,"time":1705315425000,"msg":"Server started","hostname":"web-01","pid":12345}
{"level":50,"time":1705315426000,"msg":"Database error","hostname":"web-01","error":"Connection timeout"}
```

## Keybindings

### Navigation
- `j` / `↓` - Move down one log
- `k` / `↑` - Move up one log
- `g` - Jump to first log
- `G` - Jump to last log
- `Ctrl+d` - Scroll down half page
- `Ctrl+u` - Scroll up half page
- `Ctrl+f` - Scroll down full page
- `Ctrl+b` - Scroll up full page

### Actions
- `d` - Toggle detail panel (shows full JSON)
- `f` - Toggle filter panel
- `/` - Focus filter input
- `c` - Clear active filter
- `L` - Toggle debug logs panel
- `?` - Toggle help menu
- `q` / `Esc` - Quit application

### Filter Panel
- `1` - Apply "Errors Only" filter (`level >= 50`)
- `2` - Apply "Warnings+" filter (`level >= 40`)
- `3` - Apply "Last Hour" filter
- Any other key - Start typing custom SQL filter
- `Enter` - Apply current filter
- `Esc` - Back to presets / Close panel

## SQL Filtering

The filter panel allows you to write SQL WHERE clauses to filter logs. The viewer automatically detects your log schema and shows available fields with their types.

### Filter Examples

```sql
-- Show only errors
level >= 50

-- Search for specific text
message LIKE '%timeout%'

-- Combine conditions
level >= 40 AND hostname = 'web-01'

-- Time-based filtering (Unix timestamp in milliseconds)
time >= 1705315425000

-- Multiple conditions
level >= 30 AND message LIKE '%database%' AND hostname != 'test-server'
```

### Log Levels

The viewer uses Pino-style numeric log levels:

| Level | Number | Description |
|-------|--------|-------------|
| TRACE | 10     | Very detailed debugging information |
| DEBUG | 20     | Debugging information |
| INFO  | 30     | Informational messages |
| WARN  | 40     | Warning messages |
| ERROR | 50     | Error messages |
| FATAL | 60     | Fatal error messages |

## Field Normalization

The viewer automatically normalizes common field name variations:

- `msg` → `message`
- `lvl` → `level`
- `timestamp` → `time`

This means you can use the normalized names in your SQL filters regardless of the original field names in your logs.

## Architecture

The application consists of three main layers:

1. **Ingestion Layer**: Parses JSON log files line-by-line
2. **Storage Layer**: Uses DuckDB for efficient in-memory SQL queries
3. **Presentation Layer**: Ratatui-based TUI with vim-style navigation

## Development

### Prerequisites

- Rust 2024 edition or later
- Cargo

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Running with Debug Logging

The application includes tracing for debugging. Press `L` in the UI to toggle the debug panel, or check stderr for trace output.

## Performance

- **Fast Startup**: Logs are loaded and indexed in seconds
- **Instant Filtering**: SQL queries execute in milliseconds even on large datasets
- **Memory Efficient**: Uses DuckDB's optimized in-memory storage

## Troubleshooting

### Build Issues

If you encounter build issues with DuckDB:

```bash
# Clean and rebuild
cargo clean
cargo build --release
```

Note: DuckDB compilation takes 15-20 minutes due to the bundled C++ library.

### Log File Not Loading

Ensure your log file contains valid JSON lines (one JSON object per line):

```bash
# Check your log file format
head -n 5 your-logs.json
```

### Filter Not Working

- Check the debug panel (press `L`) for SQL error messages
- Ensure field names match your log schema (shown in filter panel)
- Use normalized field names (`message`, `level`, `time`)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the TUI
- Powered by [DuckDB](https://duckdb.org/) for fast SQL queries

## Roadmap

- [ ] Export filtered logs to file
- [ ] Support for log streaming/tail mode
- [ ] Syntax highlighting for SQL filters
- [ ] Bookmarks for interesting log entries
- [ ] Custom color schemes
- [ ] Search within log messages
- [ ] Statistics and aggregations view

## Support

If you find this tool useful, please consider giving it a ⭐ on GitHub!

For bugs and feature requests, please [open an issue](https://github.com/DanSnow/log-viewer/issues).
