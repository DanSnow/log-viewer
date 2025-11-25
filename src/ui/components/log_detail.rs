use crate::ingestion::JsonLog;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct LogDetailWidget<'a> {
    log: Option<&'a JsonLog>,
    log_index: usize,
    total_logs: usize,
}

impl<'a> LogDetailWidget<'a> {
    pub fn new(log: Option<&'a JsonLog>, log_index: usize, total_logs: usize) -> Self {
        Self {
            log,
            log_index,
            total_logs,
        }
    }

    fn format_log_details(log: &JsonLog) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Pretty-print the JSON
        let json_value = serde_json::to_value(&log.fields).unwrap_or(serde_json::Value::Null);
        let pretty_json = serde_json::to_string_pretty(&json_value).unwrap_or_default();

        // Add syntax highlighting for JSON
        for line in pretty_json.lines() {
            let trimmed = line.trim_start();
            let indent_level = line.len() - trimmed.len();
            let indent = " ".repeat(indent_level);

            if trimmed.starts_with('"') && trimmed.contains(':') {
                // JSON key
                let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                let mut spans = vec![Span::raw(indent)];
                spans.push(Span::styled(
                    parts[0].to_string(),
                    Style::default().fg(Color::Cyan),
                ));
                if parts.len() > 1 {
                    spans.push(Span::styled(":", Style::default().fg(Color::White)));
                    spans.push(Span::styled(
                        parts[1].to_string(),
                        Style::default().fg(Color::Green),
                    ));
                }
                lines.push(Line::from(spans));
            } else if trimmed.starts_with('{') || trimmed.starts_with('}') {
                // Braces
                lines.push(Line::from(vec![
                    Span::raw(indent),
                    Span::styled(trimmed.to_string(), Style::default().fg(Color::White)),
                ]));
            } else {
                // Other content
                lines.push(Line::from(line.to_string()));
            }
        }

        lines
    }
}

impl<'a> Widget for LogDetailWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = if self.total_logs > 0 {
            format!("Log Details ({} of {})", self.log_index + 1, self.total_logs)
        } else {
            "Log Details (No logs)".to_string()
        };

        let content = if let Some(log) = self.log {
            Self::format_log_details(log)
        } else {
            vec![Line::from("No log selected")]
        };

        let paragraph = Paragraph::new(content).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );

        paragraph.render(area, buf);
    }
}

/// Render the log detail panel
pub fn render_log_detail(
    log: Option<&JsonLog>,
    log_index: usize,
    total_logs: usize,
    area: Rect,
    buf: &mut Buffer,
) {
    let widget = LogDetailWidget::new(log, log_index, total_logs);
    widget.render(area, buf);
}
