use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    pub fn get_level(&self) -> Option<u64> {
        self.fields.get("level")?.as_u64()
    }

    pub fn timestamp(&self) -> Option<jiff::Timestamp> {
        let ms = self.get_timestamp_ms()?;
        jiff::Timestamp::from_millisecond(ms).ok()
    }
}
