use crate::error::Result;
use crate::ingestion::JsonLog;
use crate::storage::{LogDatabase, FieldType};
use rootcause::prelude::ResultExt;
use tui_textarea::TextArea;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    AllLogs,
    Filtered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    LogList,
    FilterInput,
    FilterPresets,  // When filter panel is shown but input is not focused
}

pub struct App {
    // Data
    pub db: LogDatabase,
    pub all_logs: Vec<JsonLog>,
    pub filtered_logs: Vec<JsonLog>,
    pub field_schema: Vec<(String, FieldType)>,

    // View State
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub view_mode: ViewMode,
    pub show_detail_panel: bool,

    // Filter State
    pub active_filter: Option<String>,
    pub filter_input: TextArea<'static>,
    pub show_filter_panel: bool,
    pub filter_error: Option<String>,

    // UI State
    pub show_help: bool,
    pub show_debug_logs: bool,
    pub focus: Focus,
    pub should_quit: bool,
}

impl App {
    pub fn new(db: LogDatabase, all_logs: Vec<JsonLog>) -> Result<Self> {
        let field_schema = db
            .get_schema()
            .attach("Failed to get database schema")?;

        let mut filter_input = TextArea::default();
        filter_input.set_placeholder_text("Enter SQL WHERE clause (e.g., level >= 40)");

        Ok(Self {
            db,
            all_logs,
            filtered_logs: Vec::new(),
            field_schema,
            selected_index: 0,
            scroll_offset: 0,
            view_mode: ViewMode::AllLogs,
            show_detail_panel: false,
            active_filter: None,
            filter_input,
            show_filter_panel: false,
            filter_error: None,
            show_help: false,
            show_debug_logs: false,
            focus: Focus::LogList,
            should_quit: false,
        })
    }

    /// Toggle debug logs panel
    pub fn toggle_debug_logs(&mut self) {
        self.show_debug_logs = !self.show_debug_logs;
    }

    /// Get the currently visible logs (all or filtered)
    pub fn current_logs(&self) -> &[JsonLog] {
        match self.view_mode {
            ViewMode::AllLogs => &self.all_logs,
            ViewMode::Filtered => &self.filtered_logs,
        }
    }

    /// Get the currently selected log
    pub fn selected_log(&self) -> Option<&JsonLog> {
        self.current_logs().get(self.selected_index)
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        let logs_len = self.current_logs().len();
        if logs_len > 0 {
            self.selected_index = (self.selected_index + 1).min(logs_len - 1);
        }
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Jump to first log
    pub fn jump_to_first(&mut self) {
        self.selected_index = 0;
    }

    /// Jump to last log
    pub fn jump_to_last(&mut self) {
        let logs_len = self.current_logs().len();
        if logs_len > 0 {
            self.selected_index = logs_len - 1;
        }
    }

    /// Scroll down half page
    pub fn scroll_down_half_page(&mut self, page_height: usize) {
        let logs_len = self.current_logs().len();
        if logs_len > 0 {
            let half_page = page_height / 2;
            self.selected_index = (self.selected_index + half_page).min(logs_len - 1);
        }
    }

    /// Scroll up half page
    pub fn scroll_up_half_page(&mut self, page_height: usize) {
        let half_page = page_height / 2;
        self.selected_index = self.selected_index.saturating_sub(half_page);
    }

    /// Scroll down full page
    pub fn scroll_down_page(&mut self, page_height: usize) {
        let logs_len = self.current_logs().len();
        if logs_len > 0 {
            self.selected_index = (self.selected_index + page_height).min(logs_len - 1);
        }
    }

    /// Scroll up full page
    pub fn scroll_up_page(&mut self, page_height: usize) {
        self.selected_index = self.selected_index.saturating_sub(page_height);
    }

    /// Toggle detail panel
    pub fn toggle_detail_panel(&mut self) {
        self.show_detail_panel = !self.show_detail_panel;
    }

    /// Toggle filter panel
    pub fn toggle_filter_panel(&mut self) {
        self.show_filter_panel = !self.show_filter_panel;
        if self.show_filter_panel {
            self.focus = Focus::FilterPresets;
        } else {
            self.focus = Focus::LogList;
        }
    }

    /// Toggle help menu
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Apply the current filter from the input
    pub fn apply_filter(&mut self) -> Result<()> {
        let filter_text = self.filter_input.lines().join("");
        let trimmed = filter_text.trim();

        if trimmed.is_empty() {
            self.clear_filter();
            return Ok(());
        }

        match self.db.query_logs(Some(trimmed)) {
            Ok(logs) => {
                self.filtered_logs = logs;
                self.active_filter = Some(trimmed.to_string());
                self.view_mode = ViewMode::Filtered;
                self.selected_index = 0;
                self.filter_error = None;
                self.show_filter_panel = false;
                self.focus = Focus::LogList;
                Ok(())
            }
            Err(e) => {
                self.filter_error = Some(format!("SQL Error: {}", e));
                Err(e)
            }
        }
    }

    /// Clear the active filter and return to all logs
    pub fn clear_filter(&mut self) {
        self.active_filter = None;
        self.view_mode = ViewMode::AllLogs;
        self.selected_index = 0;
        self.filter_error = None;
        self.filter_input = TextArea::default();
        self.filter_input.set_placeholder_text("Enter SQL WHERE clause (e.g., level >= 40)");
    }

    /// Apply a preset filter
    pub fn apply_preset_filter(&mut self, filter: &str) -> Result<()> {
        self.filter_input = TextArea::from([filter]);
        self.apply_filter()
    }

    /// Focus on filter input
    pub fn focus_filter(&mut self) {
        self.focus = Focus::FilterInput;
        self.show_filter_panel = true;
    }

    /// Return focus to log list
    pub fn focus_log_list(&mut self) {
        self.focus = Focus::LogList;
        self.show_filter_panel = false;
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
