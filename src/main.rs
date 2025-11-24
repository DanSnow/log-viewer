pub mod error;
pub mod ingestion;

use ingestion::LogFileReader;

fn main() {
    println!("Log Viewer - Ingestion Test\n");

    let log_file = "example.log";
    println!("Reading logs from: {}\n", log_file);

    let mut reader = match LogFileReader::new(log_file) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Failed to open log file: {}", e);
            std::process::exit(1);
        }
    };

    let logs = reader.read_logs();
    let mut success_count = 0;
    let mut error_count = 0;

    for (line_num, result) in logs {
        match result {
            Ok(log) => {
                success_count += 1;
                println!("Line {}: ✓ Parsed successfully", line_num);

                if let Some(msg) = log.get_message() {
                    println!("  Message: {}", msg);
                }
                if let Some(level) = log.get_level() {
                    println!("  Level: {}", level);
                }
                if let Some(timestamp) = log.timestamp() {
                    println!("  Timestamp: {}", timestamp);
                }

                println!("  All fields: {:?}", log.fields);
                println!();
            }
            Err(e) => {
                error_count += 1;
                eprintln!("Line {}: ✗ Parse error: {}", line_num, e);
            }
        }
    }

    println!("\nSummary:");
    println!("  Successfully parsed: {}", success_count);
    println!("  Failed to parse: {}", error_count);
    println!("  Total lines: {}", success_count + error_count);
}

