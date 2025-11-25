use crate::storage::FieldType;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table, Widget},
};
use tui_textarea::TextArea;

pub struct FilterPanelWidget<'a> {
    field_schema: &'a [(String, FieldType)],
    filter_input: &'a TextArea<'a>,
    filter_error: Option<&'a str>,
}

impl<'a> FilterPanelWidget<'a> {
    pub fn new(
        field_schema: &'a [(String, FieldType)],
        filter_input: &'a TextArea<'a>,
        filter_error: Option<&'a str>,
    ) -> Self {
        Self {
            field_schema,
            filter_input,
            filter_error,
        }
    }
}

impl<'a> Widget for FilterPanelWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Split the area into sections
        let chunks = Layout::vertical([
            Constraint::Length(3), // Title
            Constraint::Min(8),    // Field schema table
            Constraint::Length(3), // Preset buttons
            Constraint::Length(3), // Input
            Constraint::Length(4), // Error message
            Constraint::Length(2), // Help text
        ])
        .split(area);

        // Title
        let title = Paragraph::new("Filter Panel")
            .block(Block::default().borders(Borders::ALL))
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        title.render(chunks[0], buf);

        // Field schema table
        self.render_field_schema(chunks[1], buf);

        // Preset filters
        self.render_presets(chunks[2], buf);

        // Filter input
        self.render_input(chunks[3], buf);

        // Error message
        if let Some(error) = self.filter_error {
            let error_para = Paragraph::new(error)
                .style(Style::default().fg(Color::Red))
                .block(Block::default().borders(Borders::ALL).title("Error"));
            error_para.render(chunks[4], buf);
        }

        // Help text
        let help = Paragraph::new("Enter: Apply  Esc: Cancel  1-4: Preset Filters")
            .style(Style::default().fg(Color::DarkGray));
        help.render(chunks[5], buf);
    }
}

impl<'a> FilterPanelWidget<'a> {
    fn render_field_schema(&self, area: Rect, buf: &mut Buffer) {
        let header = Row::new(vec!["Field", "Type", "Example"])
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1);

        let rows: Vec<Row> = self
            .field_schema
            .iter()
            .map(|(name, field_type)| {
                let type_str = match field_type {
                    FieldType::Text => "TEXT",
                    FieldType::Integer => "INTEGER",
                    FieldType::Float => "FLOAT",
                    FieldType::Boolean => "BOOLEAN",
                    FieldType::Json => "JSON",
                };

                let example = match field_type {
                    FieldType::Text => "\"text\"",
                    FieldType::Integer => "12345",
                    FieldType::Float => "123.45",
                    FieldType::Boolean => "true",
                    FieldType::Json => "{}",
                };

                Row::new(vec![
                    name.clone(),
                    type_str.to_string(),
                    example.to_string(),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title("Available Fields")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

        Widget::render(table, area, buf);
    }

    fn render_presets(&self, area: Rect, buf: &mut Buffer) {
        let presets = vec![
            Span::styled("[1] ", Style::default().fg(Color::Yellow)),
            Span::raw("Errors Only  "),
            Span::styled("[2] ", Style::default().fg(Color::Yellow)),
            Span::raw("Warnings+  "),
            Span::styled("[3] ", Style::default().fg(Color::Yellow)),
            Span::raw("Last Hour  "),
            Span::styled("[4] ", Style::default().fg(Color::Yellow)),
            Span::raw("Custom"),
        ];

        let para = Paragraph::new(Line::from(presets)).block(
            Block::default()
                .title("Preset Filters")
                .borders(Borders::ALL),
        );

        para.render(area, buf);
    }

    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        // Render the TextArea widget directly
        Widget::render(self.filter_input, area, buf);
    }
}

/// Render the filter panel
pub fn render_filter_panel(
    field_schema: &[(String, FieldType)],
    filter_input: &TextArea,
    filter_error: Option<&str>,
    area: Rect,
    buf: &mut Buffer,
) {
    let widget = FilterPanelWidget::new(field_schema, filter_input, filter_error);
    widget.render(area, buf);
}
