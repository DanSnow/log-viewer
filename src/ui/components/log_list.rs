use crate::ingestion::{JsonLog, LogLevel};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, StatefulWidget, Widget},
};

pub struct LogListWidget<'a> {
    logs: &'a [JsonLog],
    title: String,
}

impl<'a> LogListWidget<'a> {
    pub fn new(logs: &'a [JsonLog], title: String) -> Self {
        Self { logs, title }
    }

    /// Format a log entry as a single line
    fn format_log_line(log: &JsonLog) -> Line<'static> {
        let mut spans = Vec::new();

        // Format timestamp
        if let Some(timestamp) = log.timestamp() {
            let time_str = timestamp
                .strftime("%H:%M:%S")
                .to_string();
            spans.push(Span::styled(
                format!("[{}] ", time_str),
                Style::default().fg(Color::DarkGray),
            ));
        } else if let Some(time_ms) = log.get_timestamp_ms() {
            spans.push(Span::styled(
                format!("[{}] ", time_ms),
                Style::default().fg(Color::DarkGray),
            ));
        }

        // Format level with color
        if let Some(level) = log.get_level() {
            let (level_str, color) = match level {
                LogLevel::Trace => ("TRACE", Color::DarkGray),
                LogLevel::Debug => ("DEBUG", Color::Blue),
                LogLevel::Info => ("INFO ", Color::Cyan),
                LogLevel::Warn => ("WARN ", Color::Yellow),
                LogLevel::Error => ("ERROR", Color::Red),
                LogLevel::Fatal => ("FATAL", Color::LightRed),
            };
            spans.push(Span::styled(
                format!("{} ", level_str),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
        }

        // Format message
        if let Some(message) = log.get_message() {
            // Truncate long messages
            let truncated = if message.len() > 80 {
                format!("{}...", &message[..77])
            } else {
                message.to_string()
            };
            spans.push(Span::raw(truncated));
        }

        // Show field count
        let field_count = log.fields.len();
        if field_count > 0 {
            spans.push(Span::styled(
                format!(" (+{})", field_count),
                Style::default().fg(Color::DarkGray),
            ));
        }

        Line::from(spans)
    }
}

impl<'a> Widget for LogListWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .logs
            .iter()
            .map(|log| ListItem::new(Self::format_log_line(log)))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(self.title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        // Create a stateful widget and render with state
        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(0)); // This will be overridden by the actual selected index
        StatefulWidget::render(list, area, buf, &mut state);
    }
}

pub struct LogListState {
    pub selected: usize,
}

impl LogListState {
    pub fn new(selected: usize) -> Self {
        Self { selected }
    }
}

/// Render the log list with proper state management
pub fn render_log_list(
    logs: &[JsonLog],
    selected_index: usize,
    title: String,
    area: Rect,
    buf: &mut Buffer,
) {
    let items: Vec<ListItem> = logs
        .iter()
        .map(|log| ListItem::new(LogListWidget::format_log_line(log)))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ratatui::widgets::ListState::default();
    if !logs.is_empty() {
        state.select(Some(selected_index));
    }

    StatefulWidget::render(list, area, buf, &mut state);
}
