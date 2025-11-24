use rootcause::prelude::Report;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogViewerError {
    #[error("Failed to read log file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] duckdb::Error),

    #[error("Invalid log format: {0}")]
    InvalidLogFormat(String),

    #[error("Timestamp conversion error: {0}")]
    TimestampError(String),

    #[error("Error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Report<LogViewerError>>;
