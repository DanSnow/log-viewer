use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};
use tui_logger::TuiLoggerWidget;

pub struct DebugLogsWidget;

impl DebugLogsWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for DebugLogsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let widget = TuiLoggerWidget::default()
            .block(
                Block::default()
                    .title("Debug Logs (Press L to toggle)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style_error(Style::default().fg(Color::Red))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_info(Style::default().fg(Color::Green))
            .style_debug(Style::default().fg(Color::Blue))
            .style_trace(Style::default().fg(Color::DarkGray));

        widget.render(area, buf);
    }
}

/// Render the debug logs panel
pub fn render_debug_logs(area: Rect, buf: &mut Buffer) {
    let widget = DebugLogsWidget::new();
    widget.render(area, buf);
}
