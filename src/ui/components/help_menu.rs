use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

pub struct HelpMenuWidget;

impl HelpMenuWidget {
    fn create_help_content() -> Vec<Line<'static>> {
        vec![
            Line::from(vec![Span::styled(
                "Log Viewer - Help",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Navigation:",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )]),
            Line::from("  j / ↓       - Move down one log"),
            Line::from("  k / ↑       - Move up one log"),
            Line::from("  g           - Jump to first log"),
            Line::from("  G           - Jump to last log"),
            Line::from("  Ctrl+d      - Scroll down half page"),
            Line::from("  Ctrl+u      - Scroll up half page"),
            Line::from("  Ctrl+f      - Scroll down full page"),
            Line::from("  Ctrl+b      - Scroll up full page"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Actions:",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )]),
            Line::from("  d           - Toggle detail panel"),
            Line::from("  f           - Toggle filter panel"),
            Line::from("  /           - Focus filter input"),
            Line::from("  c           - Clear active filter"),
            Line::from("  L           - Toggle debug logs panel"),
            Line::from("  ?           - Toggle this help menu"),
            Line::from("  q / Esc     - Quit application"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Filter Panel:",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )]),
            Line::from("  1           - Apply \"Errors Only\" filter (level >= 50)"),
            Line::from("  2           - Apply \"Warnings+\" filter (level >= 40)"),
            Line::from("  3           - Apply \"Last Hour\" filter"),
            Line::from("  Any key     - Start typing custom SQL filter"),
            Line::from("  Enter       - Apply current filter"),
            Line::from("  Esc         - Back to presets / Close panel"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "SQL Filter Examples:",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )]),
            Line::from("  level >= 40"),
            Line::from("  message LIKE '%error%'"),
            Line::from("  level >= 40 AND hostname = 'server-01'"),
            Line::from("  time >= 1531171074000"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press ? or Esc to close this help menu",
                Style::default().fg(Color::DarkGray),
            )]),
        ]
    }
}

impl Widget for HelpMenuWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate centered position for the help modal
        let popup_width = 70;
        let popup_height = 38;

        let x = (area.width.saturating_sub(popup_width)) / 2;
        let y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect {
            x: area.x + x,
            y: area.y + y,
            width: popup_width.min(area.width),
            height: popup_height.min(area.height),
        };

        // Clear the area behind the popup
        Clear.render(popup_area, buf);

        // Render the help content
        let content = Self::create_help_content();
        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Left);

        paragraph.render(popup_area, buf);
    }
}

/// Render the help menu as a centered modal
pub fn render_help_menu(area: Rect, buf: &mut Buffer) {
    let widget = HelpMenuWidget;
    widget.render(area, buf);
}
