use crate::error::{LogViewerError, Result};
use crate::ingestion::JsonLog;
use crate::storage::schema::{SchemaBuilder, normalize_field_name};
use duckdb::{Connection, params_from_iter};
use rootcause::prelude::*;
use serde_json::Value;

pub struct LogDatabase {
    conn: Connection,
    table_name: String,
    field_names: Vec<String>,
}

impl LogDatabase {
    /// Create a new in-memory database
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(LogViewerError::from)
            .attach("Failed to create in-memory DuckDB connection")?;

        Ok(Self {
            conn,
            table_name: "logs".to_string(),
            field_names: Vec::new(),
        })
    }

    /// Create a new file-based database
    pub fn new_with_file(path: &str) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(LogViewerError::from)
            .attach_with(|| format!("Failed to open DuckDB database at {}", path))?;

        Ok(Self {
            conn,
            table_name: "logs".to_string(),
            field_names: Vec::new(),
        })
    }

    /// Create table with auto-generated schema from sample logs
    /// Samples the first `sample_size` logs to detect field types
    pub fn create_table_from_logs(&mut self, logs: &[JsonLog], sample_size: usize) -> Result<()> {
        let sample_logs = if logs.len() > sample_size {
            &logs[..sample_size]
        } else {
            logs
        };

        tracing::info!("Analyzing {} sample logs to detect schema", sample_logs.len());

        let mut schema_builder = SchemaBuilder::new();
        schema_builder.analyze_logs(sample_logs);

        let create_sql = schema_builder.generate_create_table_sql(&self.table_name);

        tracing::debug!("Creating table with SQL: {}", create_sql);

        self.conn
            .execute(&create_sql, [])
            .map_err(LogViewerError::from)
            .attach_with(|| format!("Failed to create table with SQL: {}", create_sql))?;

        self.field_names = schema_builder.field_names();

        tracing::info!("Created table '{}' with {} fields: {:?}",
            self.table_name, self.field_names.len(), self.field_names);

        Ok(())
    }

    /// Insert a single log entry
    pub fn insert_log(&self, log: &JsonLog) -> Result<()> {
        if self.field_names.is_empty() {
            return Err(LogViewerError::Database(
                duckdb::Error::InvalidParameterCount(0, 0),
            ))
            .attach("Cannot insert log: table not created yet. Call create_table_from_logs first");
        }

        let placeholders: Vec<String> = (1..=self.field_names.len())
            .map(|i| format!("?{}", i))
            .collect();
        let insert_sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table_name,
            self.field_names.join(", "),
            placeholders.join(", ")
        );

        let params = self.extract_params_from_log(log);

        self.conn
            .execute(&insert_sql, params_from_iter(params.iter()))
            .map_err(LogViewerError::from)
            .attach_with(|| format!("Failed to insert log with SQL: {}", insert_sql))?;

        Ok(())
    }

    /// Insert multiple logs in a batch (using a transaction for efficiency)
    pub fn insert_logs(&mut self, logs: &[JsonLog]) -> Result<usize> {
        if self.field_names.is_empty() {
            return Err(LogViewerError::Database(
                duckdb::Error::InvalidParameterCount(0, 0),
            ))
            .attach(
                "Cannot insert logs: table not created yet. Call create_table_from_logs first",
            );
        }

        tracing::info!("Inserting {} logs into database", logs.len());

        // Extract all params before starting transaction to avoid borrow issues
        let all_params: Vec<_> = logs
            .iter()
            .enumerate()
            .map(|(idx, log)| {
                let params = self.extract_params_from_log(log);
                if idx == 0 {
                    tracing::debug!("First log has {} fields: {:?}", log.fields.len(), log.fields.keys().collect::<Vec<_>>());
                }
                params
            })
            .collect();

        let tx = self
            .conn
            .transaction()
            .map_err(LogViewerError::from)
            .attach("Failed to start transaction")?;

        let placeholders: Vec<String> = (1..=self.field_names.len())
            .map(|i| format!("?{}", i))
            .collect();
        let insert_sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table_name,
            self.field_names.join(", "),
            placeholders.join(", ")
        );

        tracing::debug!("Insert SQL: {}", insert_sql);

        let mut inserted = 0;
        for params in all_params {
            tx.execute(&insert_sql, params_from_iter(params.iter()))
                .map_err(LogViewerError::from)
                .attach_with(|| {
                    format!("Failed to insert log in batch with SQL: {}", insert_sql)
                })?;

            inserted += 1;
        }

        tx.commit()
            .map_err(LogViewerError::from)
            .attach("Failed to commit transaction")?;

        tracing::info!("Successfully inserted {} logs", inserted);

        Ok(inserted)
    }

    /// Extract parameter values from a log entry in the order of field_names
    fn extract_params_from_log(&self, log: &JsonLog) -> Vec<Box<dyn duckdb::ToSql>> {
        let mut params: Vec<Box<dyn duckdb::ToSql>> = Vec::new();

        for field_name in &self.field_names {
            // Try to find the field with normalization
            let value = log
                .fields
                .iter()
                .find(|(k, _)| normalize_field_name(k) == field_name)
                .map(|(_, v)| v);

            match value {
                Some(Value::Null) | None => params.push(Box::new(None::<String>)),
                Some(Value::Bool(b)) => params.push(Box::new(*b)),
                Some(Value::Number(n)) => {
                    if let Some(i) = n.as_i64() {
                        params.push(Box::new(i));
                    } else if let Some(f) = n.as_f64() {
                        params.push(Box::new(f));
                    } else {
                        params.push(Box::new(None::<i64>));
                    }
                }
                Some(Value::String(s)) => params.push(Box::new(s.clone())),
                Some(Value::Array(_)) | Some(Value::Object(_)) => {
                    // Store complex types as JSON strings
                    params.push(Box::new(value.unwrap().to_string()));
                }
            }
        }

        params
    }

    /// Get the number of rows in the logs table
    pub fn count_logs(&self) -> Result<usize> {
        let count: usize = self
            .conn
            .query_row(
                &format!("SELECT COUNT(*) FROM {}", self.table_name),
                [],
                |row| row.get(0),
            )
            .map_err(LogViewerError::from)
            .attach("Failed to count logs")?;

        Ok(count)
    }

    /// Get the table name
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// Get the field names
    pub fn field_names(&self) -> &[String] {
        &self.field_names
    }

    /// Query logs with optional WHERE clause
    /// Returns JsonLog instances constructed from database rows
    pub fn query_logs(&self, where_clause: Option<&str>) -> Result<Vec<JsonLog>> {
        let sql = if let Some(where_clause) = where_clause {
            format!("SELECT * FROM {} WHERE {}", self.table_name, where_clause)
        } else {
            format!("SELECT * FROM {}", self.table_name)
        };

        tracing::debug!("Executing query: {}", sql);

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(LogViewerError::from)
            .attach_with(|| format!("Failed to prepare query: {}", sql))?;

        stmt.execute([])
            .map_err(LogViewerError::from)
            .attach_with(|| format!("Fail to execute query: {sql}"))?;

        let column_count = stmt.column_count();
        let column_names: Vec<String> = (0..column_count)
            .map(|i| {
                stmt.column_name(i)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| "unknown".to_string())
            })
            .collect();

        tracing::debug!("Query returned {} columns: {:?}", column_count, column_names);

        let rows = stmt
            .query_map([], |row| {
                let mut fields = std::collections::HashMap::new();

                for (i, col_name) in column_names.iter().enumerate() {
                    // Skip the id column
                    if col_name == "id" {
                        continue;
                    }

                    // Try to get the value as different types
                    let value: Value = if let Ok(s) = row.get::<_, String>(i) {
                        tracing::trace!("Column '{}' [{}]: String = {:?}", col_name, i, s);
                        Value::String(s)
                    } else if let Ok(i_val) = row.get::<_, i64>(i) {
                        tracing::trace!("Column '{}' [{}]: i64 = {}", col_name, i, i_val);
                        Value::Number(i_val.into())
                    } else if let Ok(f) = row.get::<_, f64>(i) {
                        tracing::trace!("Column '{}' [{}]: f64 = {}", col_name, i, f);
                        serde_json::Number::from_f64(f)
                            .map(Value::Number)
                            .unwrap_or(Value::Null)
                    } else if let Ok(b) = row.get::<_, bool>(i) {
                        tracing::trace!("Column '{}' [{}]: bool = {}", col_name, i, b);
                        Value::Bool(b)
                    } else {
                        tracing::warn!("Column '{}' [{}]: Could not parse, using Null", col_name, i);
                        Value::Null
                    };

                    fields.insert(col_name.clone(), value);
                }

                tracing::trace!("Parsed log with {} fields: {:?}", fields.len(), fields.keys().collect::<Vec<_>>());
                Ok(JsonLog::new(fields))
            })
            .map_err(LogViewerError::from)
            .attach_with(|| format!("Failed to query logs with SQL: {}", sql))?;

        let logs: std::result::Result<Vec<_>, _> = rows.collect();
        let logs = logs.map_err(LogViewerError::from)
            .attach("Failed to collect query results")?;

        tracing::info!("Query returned {} log entries", logs.len());

        Ok(logs)
    }

    /// Get the schema (field names and types) for the UI
    pub fn get_schema(&self) -> Result<Vec<(String, crate::storage::FieldType)>> {
        use crate::storage::FieldType;

        let sql = format!("PRAGMA table_info({})", self.table_name);
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(LogViewerError::from)
            .attach_with(|| format!("Failed to prepare schema query: {}", sql))?;

        let rows = stmt
            .query_map([], |row| {
                let name: String = row.get(1)?;
                let type_str: String = row.get(2)?;
                Ok((name, type_str))
            })
            .map_err(LogViewerError::from)
            .attach("Failed to query table schema")?;

        let mut schema = Vec::new();
        for row_result in rows {
            let (name, type_str) = row_result
                .map_err(LogViewerError::from)
                .attach("Failed to read schema row")?;

            // Skip the id column
            if name == "id" {
                continue;
            }

            let field_type = match type_str.as_str() {
                "TEXT" => FieldType::Text,
                "BIGINT" => FieldType::Integer,
                "DOUBLE" => FieldType::Float,
                "BOOLEAN" => FieldType::Boolean,
                _ => FieldType::Text,
            };

            schema.push((name, field_type));
        }

        Ok(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_create_in_memory_database() {
        let db = LogDatabase::new_in_memory();
        assert!(db.is_ok());
    }

    #[test]
    fn test_create_table_and_insert() {
        let mut db = LogDatabase::new_in_memory().unwrap();

        let mut fields = HashMap::new();
        fields.insert("msg".to_string(), json!("test message"));
        fields.insert("level".to_string(), json!(30));
        fields.insert("time".to_string(), json!(1234567890));

        let log = JsonLog::new(fields);

        db.create_table_from_logs(&[log.clone()], 100).unwrap();
        db.insert_log(&log).unwrap();

        let count = db.count_logs().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_batch_insert() {
        let mut db = LogDatabase::new_in_memory().unwrap();

        let mut logs = Vec::new();
        for i in 0..10 {
            let mut fields = HashMap::new();
            fields.insert("msg".to_string(), json!(format!("message {}", i)));
            fields.insert("level".to_string(), json!(30));
            fields.insert("time".to_string(), json!(1234567890 + i));
            logs.push(JsonLog::new(fields));
        }

        db.create_table_from_logs(&logs, 100).unwrap();
        let inserted = db.insert_logs(&logs).unwrap();

        assert_eq!(inserted, 10);
        assert_eq!(db.count_logs().unwrap(), 10);
    }

    #[test]
    fn test_field_normalization() {
        let mut db = LogDatabase::new_in_memory().unwrap();

        let mut fields1 = HashMap::new();
        fields1.insert("msg".to_string(), json!("message 1"));
        fields1.insert("lvl".to_string(), json!(30));

        let mut fields2 = HashMap::new();
        fields2.insert("message".to_string(), json!("message 2"));
        fields2.insert("level".to_string(), json!(40));

        let log1 = JsonLog::new(fields1);
        let log2 = JsonLog::new(fields2);

        db.create_table_from_logs(&[log1.clone(), log2.clone()], 100)
            .unwrap();
        db.insert_logs(&[log1, log2]).unwrap();

        assert_eq!(db.count_logs().unwrap(), 2);
    }
}
