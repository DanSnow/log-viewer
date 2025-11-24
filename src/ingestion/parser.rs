use crate::error::{LogViewerError, Result};
use crate::ingestion::models::JsonLog;
use rootcause::prelude::{Report, ResultExt};

pub fn parse_json_line(line: &str) -> Result<JsonLog> {
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return Err(Report::new(LogViewerError::InvalidLogFormat(
            "Empty line".to_string(),
        )));
    }

    let fields: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(trimmed)
            .map_err(|e| LogViewerError::from(e))
            .attach("Failed to parse JSON line")?;

    if fields.is_empty() {
        return Err(Report::new(LogViewerError::InvalidLogFormat(
            "Empty JSON object".to_string(),
        )));
    }

    Ok(JsonLog::new(fields))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pino_log() {
        let line = r#"{"level":30,"time":1531171074631,"msg":"hello world","pid":657,"hostname":"Davids-MBP-3.fritz.box"}"#;
        let result = parse_json_line(line);
        assert!(result.is_ok());

        let log = result.unwrap();
        assert_eq!(log.get_level(), Some(crate::ingestion::LogLevel::Info));
        assert_eq!(log.get_level_raw(), Some(30));
        assert_eq!(log.get_message(), Some("hello world"));
        assert_eq!(log.get_timestamp_ms(), Some(1531171074631));
    }

    #[test]
    fn test_parse_with_extra_fields() {
        let line = r#"{"level":30,"time":1531171082399,"msg":"hello child!","pid":657,"hostname":"Davids-MBP-3.fritz.box","a":"property"}"#;
        let result = parse_json_line(line);
        assert!(result.is_ok());

        let log = result.unwrap();
        assert_eq!(
            log.get_field("a").and_then(|v| v.as_str()),
            Some("property")
        );
    }

    #[test]
    fn test_parse_empty_line() {
        let line = "";
        let result = parse_json_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_json() {
        let line = "not valid json";
        let result = parse_json_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_log_level_comparison() {
        use crate::ingestion::LogLevel;

        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
        assert!(LogLevel::Debug > LogLevel::Trace);
        assert!(LogLevel::Fatal > LogLevel::Error);

        assert_eq!(LogLevel::Info.as_u64(), 30);
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }
}
