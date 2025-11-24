pub mod error;
pub mod ingestion;
pub mod storage;

use ingestion::LogFileReader;
use storage::LogDatabase;

fn main() {
    println!("Log Viewer - DuckDB Integration Test\n");

    let log_file = "example.log";
    println!("Reading logs from: {}\n", log_file);

    // Step 1: Read and parse logs
    let mut reader = match LogFileReader::new(log_file) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Failed to open log file: {}", e);
            std::process::exit(1);
        }
    };

    let log_results = reader.read_logs();
    let mut parsed_logs = Vec::new();
    let mut error_count = 0;

    println!("Parsing logs...");
    for (line_num, result) in log_results {
        match result {
            Ok(log) => {
                parsed_logs.push(log);
            }
            Err(e) => {
                error_count += 1;
                eprintln!("Line {}: ✗ Parse error: {}", line_num, e);
            }
        }
    }

    println!(
        "  Successfully parsed: {} logs",
        parsed_logs.len()
    );
    println!("  Failed to parse: {} logs\n", error_count);

    if parsed_logs.is_empty() {
        eprintln!("No logs to process. Exiting.");
        std::process::exit(1);
    }

    // Step 2: Create database and detect schema from first 100 logs
    println!("Creating in-memory DuckDB database...");
    let mut db = match LogDatabase::new_in_memory() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to create database: {}", e);
            std::process::exit(1);
        }
    };

    let sample_size = 100;
    println!("Detecting schema from first {} logs...", sample_size);
    if let Err(e) = db.create_table_from_logs(&parsed_logs, sample_size) {
        eprintln!("Failed to create table: {}", e);
        std::process::exit(1);
    }

    println!("  Table '{}' created", db.table_name());
    println!("  Detected fields: {}\n", db.field_names().join(", "));

    // Step 3: Insert logs into database
    println!("Inserting logs into database...");
    let inserted_count = match db.insert_logs(&parsed_logs) {
        Ok(count) => count,
        Err(e) => {
            eprintln!("Failed to insert logs: {}", e);
            std::process::exit(1);
        }
    };

    println!("  Inserted {} logs\n", inserted_count);

    // Step 4: Verify insertion
    let db_count = match db.count_logs() {
        Ok(count) => count,
        Err(e) => {
            eprintln!("Failed to count logs: {}", e);
            std::process::exit(1);
        }
    };

    println!("Summary:");
    println!("  Total parsed: {}", parsed_logs.len());
    println!("  Inserted into DB: {}", inserted_count);
    println!("  Verified in DB: {}", db_count);
    println!("\n✓ DuckDB integration successful!");
}

