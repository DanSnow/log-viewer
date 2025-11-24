use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Trace = 10,
    Debug = 20,
    Info = 30,
    Warn = 40,
    Error = 50,
    Fatal = 60,
}

impl LogLevel {
    pub fn from_u64(level: u64) -> Option<Self> {
        match level {
            10 => Some(LogLevel::Trace),
            20 => Some(LogLevel::Debug),
            30 => Some(LogLevel::Info),
            40 => Some(LogLevel::Warn),
            50 => Some(LogLevel::Error),
            60 => Some(LogLevel::Fatal),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }

    pub fn as_u64(&self) -> u64 {
        *self as u64
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonLog {
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
}

impl JsonLog {
    pub fn new(fields: HashMap<String, serde_json::Value>) -> Self {
        Self { fields }
    }

    pub fn get_field(&self, key: &str) -> Option<&serde_json::Value> {
        self.fields.get(key)
    }

    pub fn get_timestamp_ms(&self) -> Option<i64> {
        self.fields.get("time")?.as_i64()
    }

    pub fn get_message(&self) -> Option<&str> {
        self.fields.get("msg")?.as_str()
    }

    pub fn get_level_raw(&self) -> Option<u64> {
        self.fields.get("level")?.as_u64()
    }

    pub fn get_level(&self) -> Option<LogLevel> {
        let level = self.get_level_raw()?;
        LogLevel::from_u64(level)
    }

    pub fn timestamp(&self) -> Option<jiff::Timestamp> {
        let ms = self.get_timestamp_ms()?;
        jiff::Timestamp::from_millisecond(ms).ok()
    }
}
