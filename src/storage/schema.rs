use crate::ingestion::JsonLog;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    Text,
    Integer,
    Float,
    Boolean,
    Json,
}

impl FieldType {
    pub fn to_sql(&self) -> &'static str {
        match self {
            FieldType::Text => "TEXT",
            FieldType::Integer => "BIGINT",
            FieldType::Float => "DOUBLE",
            FieldType::Boolean => "BOOLEAN",
            FieldType::Json => "TEXT",
        }
    }

    /// Merge two field types - if they differ, use a more general type
    pub fn merge(&self, other: &FieldType) -> FieldType {
        if self == other {
            return self.clone();
        }

        match (self, other) {
            // Integer can be promoted to Float
            (FieldType::Integer, FieldType::Float) | (FieldType::Float, FieldType::Integer) => {
                FieldType::Float
            }
            // If types conflict, use Text as fallback
            _ => FieldType::Text,
        }
    }
}

/// Detect the field type from a JSON value
pub fn detect_field_type(value: &Value) -> FieldType {
    match value {
        Value::Null => FieldType::Text, // Treat null as text for flexibility
        Value::Bool(_) => FieldType::Boolean,
        Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                FieldType::Integer
            } else {
                FieldType::Float
            }
        }
        Value::String(_) => FieldType::Text,
        Value::Array(_) | Value::Object(_) => FieldType::Json, // Complex types stored as JSON
    }
}

/// Normalize common field names to standard names
pub fn normalize_field_name(field: &str) -> &str {
    match field {
        "msg" => "message",
        "lvl" => "level",
        "timestamp" => "time",
        _ => field,
    }
}

pub struct SchemaBuilder {
    field_types: HashMap<String, FieldType>,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        Self {
            field_types: HashMap::new(),
        }
    }

    /// Analyze a log entry and update field type information
    pub fn analyze_log(&mut self, log: &JsonLog) {
        for (field_name, value) in &log.fields {
            let normalized_name = normalize_field_name(field_name).to_string();
            let detected_type = detect_field_type(value);

            self.field_types
                .entry(normalized_name)
                .and_modify(|existing_type| {
                    *existing_type = existing_type.merge(&detected_type);
                })
                .or_insert(detected_type);
        }
    }

    /// Analyze multiple logs to build the schema
    pub fn analyze_logs(&mut self, logs: &[JsonLog]) {
        for log in logs {
            self.analyze_log(log);
        }
    }

    /// Generate CREATE TABLE SQL statement
    pub fn generate_create_table_sql(&self, table_name: &str) -> String {
        let mut sql = format!("CREATE SEQUENCE IF NOT EXISTS seq_{}_id START 1;\n", table_name);
        sql.push_str(&format!("CREATE TABLE {} (\n", table_name));
        sql.push_str("    id INTEGER PRIMARY KEY DEFAULT nextval('seq_");
        sql.push_str(table_name);
        sql.push_str("_id'),\n");

        // Sort fields for consistent output
        let mut fields: Vec<_> = self.field_types.iter().collect();
        fields.sort_by_key(|(name, _)| *name);

        for (i, (field_name, field_type)) in fields.iter().enumerate() {
            let sql_type = field_type.to_sql();
            sql.push_str(&format!("    {} {}", field_name, sql_type));

            if i < fields.len() - 1 {
                sql.push(',');
            }
            sql.push('\n');
        }

        sql.push_str(")");
        sql
    }

    /// Get the detected field types
    pub fn field_types(&self) -> &HashMap<String, FieldType> {
        &self.field_types
    }

    /// Get all field names in sorted order
    pub fn field_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.field_types.keys().cloned().collect();
        names.sort();
        names
    }
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_detect_field_type() {
        assert_eq!(detect_field_type(&json!(42)), FieldType::Integer);
        assert_eq!(detect_field_type(&json!(3.14)), FieldType::Float);
        assert_eq!(detect_field_type(&json!("hello")), FieldType::Text);
        assert_eq!(detect_field_type(&json!(true)), FieldType::Boolean);
        assert_eq!(detect_field_type(&json!(null)), FieldType::Text);
        assert_eq!(detect_field_type(&json!([])), FieldType::Json);
        assert_eq!(detect_field_type(&json!({})), FieldType::Json);
    }

    #[test]
    fn test_field_type_merge() {
        assert_eq!(
            FieldType::Integer.merge(&FieldType::Float),
            FieldType::Float
        );
        assert_eq!(
            FieldType::Float.merge(&FieldType::Integer),
            FieldType::Float
        );
        assert_eq!(
            FieldType::Integer.merge(&FieldType::Text),
            FieldType::Text
        );
        assert_eq!(
            FieldType::Text.merge(&FieldType::Integer),
            FieldType::Text
        );
    }

    #[test]
    fn test_normalize_field_name() {
        assert_eq!(normalize_field_name("msg"), "message");
        assert_eq!(normalize_field_name("message"), "message");
        assert_eq!(normalize_field_name("lvl"), "level");
        assert_eq!(normalize_field_name("level"), "level");
        assert_eq!(normalize_field_name("timestamp"), "time");
        assert_eq!(normalize_field_name("time"), "time");
        assert_eq!(normalize_field_name("other"), "other");
    }

    #[test]
    fn test_schema_builder() {
        let mut builder = SchemaBuilder::new();

        let mut fields1 = HashMap::new();
        fields1.insert("msg".to_string(), json!("test message"));
        fields1.insert("level".to_string(), json!(30));
        fields1.insert("time".to_string(), json!(1234567890));

        let mut fields2 = HashMap::new();
        fields2.insert("message".to_string(), json!("another message"));
        fields2.insert("lvl".to_string(), json!(40));
        fields2.insert("timestamp".to_string(), json!(1234567891));

        let log1 = JsonLog::new(fields1);
        let log2 = JsonLog::new(fields2);

        builder.analyze_logs(&[log1, log2]);

        let field_types = builder.field_types();
        assert_eq!(field_types.get("message"), Some(&FieldType::Text));
        assert_eq!(field_types.get("level"), Some(&FieldType::Integer));
        assert_eq!(field_types.get("time"), Some(&FieldType::Integer));
    }

    #[test]
    fn test_generate_create_table_sql() {
        let mut builder = SchemaBuilder::new();

        let mut fields = HashMap::new();
        fields.insert("msg".to_string(), json!("test"));
        fields.insert("level".to_string(), json!(30));
        fields.insert("time".to_string(), json!(1234567890));

        let log = JsonLog::new(fields);
        builder.analyze_log(&log);

        let sql = builder.generate_create_table_sql("logs");
        insta::assert_snapshot!(sql);
    }

    #[test]
    fn test_generate_create_table_sql_complex() {
        let mut builder = SchemaBuilder::new();

        // Test with multiple field types including JSON and normalization
        let mut fields1 = HashMap::new();
        fields1.insert("msg".to_string(), json!("hello"));
        fields1.insert("level".to_string(), json!(30));
        fields1.insert("time".to_string(), json!(1234567890));
        fields1.insert("count".to_string(), json!(42));
        fields1.insert("ratio".to_string(), json!(3.14));
        fields1.insert("enabled".to_string(), json!(true));
        fields1.insert("metadata".to_string(), json!({"foo": "bar"}));

        let log1 = JsonLog::new(fields1);
        builder.analyze_log(&log1);

        let sql = builder.generate_create_table_sql("app_logs");
        insta::assert_snapshot!(sql);
    }

    #[test]
    fn test_generate_create_table_sql_with_type_merging() {
        let mut builder = SchemaBuilder::new();

        // First log has integer
        let mut fields1 = HashMap::new();
        fields1.insert("value".to_string(), json!(42));
        let log1 = JsonLog::new(fields1);

        // Second log has float - should merge to Float
        let mut fields2 = HashMap::new();
        fields2.insert("value".to_string(), json!(3.14));
        let log2 = JsonLog::new(fields2);

        builder.analyze_logs(&[log1, log2]);

        let sql = builder.generate_create_table_sql("metrics");
        insta::assert_snapshot!(sql);
    }
}
