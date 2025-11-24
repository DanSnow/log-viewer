use crate::error::{LogViewerError, Result};
use crate::ingestion::models::JsonLog;
use crate::ingestion::parser::parse_json_line;
use rootcause::prelude::{Report, ResultExt};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogFileReader {
    reader: BufReader<File>,
    line_number: usize,
}

impl LogFileReader {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path.as_ref())
            .map_err(|e| LogViewerError::from(e))
            .attach("Failed to open log file")?;
        Ok(Self {
            reader: BufReader::new(file),
            line_number: 0,
        })
    }

    pub fn read_logs(&mut self) -> Vec<(usize, Result<JsonLog>)> {
        let mut logs = Vec::new();

        loop {
            let mut line = String::new();
            match self.reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    self.line_number += 1;
                    let parse_result = parse_json_line(&line);
                    logs.push((self.line_number, parse_result));
                }
                Err(e) => {
                    self.line_number += 1;
                    logs.push((self.line_number, Err(Report::new(LogViewerError::from(e)))));
                }
            }
        }
        logs
    }

    pub fn current_line_number(&self) -> usize {
        self.line_number
    }
}
