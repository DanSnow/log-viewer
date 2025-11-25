pub mod error;
pub mod ingestion;
pub mod storage;
pub mod ui;

use error::Result;
use ingestion::LogFileReader;
use ratatui::layout::{Constraint, Layout};
use rootcause::prelude::ResultExt;
use storage::LogDatabase;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::prelude::*;
use ui::{App, cleanup_terminal, handle_events, setup_terminal};

fn main() -> Result<()> {
    let _ = tui_logger::init_logger(tui_logger::LevelFilter::Debug);

    // Initialize tracing subscriber with tui-logger support
    // This bridges tracing events to tui-logger for the debug panel
    let tui_logger_layer = tui_logger::tracing_subscriber_layer().with_filter(LevelFilter::DEBUG);

    tracing_subscriber::registry().with(tui_logger_layer).init();

    tracing::info!("Starting log-viewer application");

    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <log-file-path>", args[0]);
        std::process::exit(1);
    }

    let log_file = &args[1];
    tracing::info!("Loading log file: {}", log_file);

    // Load and parse logs
    let logs = load_logs(log_file)?;

    if logs.is_empty() {
        eprintln!("No logs to display. Exiting.");
        std::process::exit(1);
    }

    // Create database and insert logs
    let mut db = LogDatabase::new_in_memory().attach("Failed to create database")?;
    db.create_table_from_logs(&logs, 100)
        .attach("Failed to create table from logs")?;
    db.insert_logs(&logs)
        .attach("Failed to insert logs into database")?;

    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Create app state
    let mut app = App::new(db, logs).attach("Failed to initialize app")?;

    // Main event loop
    let result = run_app(&mut terminal, &mut app);

    // Cleanup terminal
    cleanup_terminal()?;

    // Handle any errors that occurred during the app run
    result?;

    Ok(())
}

fn load_logs(log_file: &str) -> Result<Vec<ingestion::JsonLog>> {
    let mut reader = LogFileReader::new(log_file)
        .attach_with(|| format!("Failed to open log file: {}", log_file))?;

    let log_results = reader.read_logs();
    let mut parsed_logs = Vec::new();

    for (_line_num, result) in log_results {
        if let Ok(log) = result {
            parsed_logs.push(log);
        }
        // Silently skip parse errors in TUI mode
    }

    Ok(parsed_logs)
}

fn run_app(terminal: &mut ui::terminal::Tui, app: &mut App) -> Result<()> {
    loop {
        // Draw UI
        terminal
            .draw(|frame| {
                render_ui(frame, app);
            })
            .map_err(error::LogViewerError::from)
            .attach("Failed to draw UI")?;

        // Get the height of the log list area for pagination
        let area = terminal.size().map_err(error::LogViewerError::from)?;
        let page_height = calculate_log_list_height(area.height, app.show_detail_panel);

        // Handle events
        handle_events(app, page_height)?;

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn calculate_log_list_height(total_height: u16, show_detail: bool) -> usize {
    if show_detail {
        // Split screen: top half for logs, bottom half for details
        ((total_height / 2).saturating_sub(4)) as usize
    } else {
        // Full screen for logs
        (total_height.saturating_sub(6)) as usize
    }
}

fn render_ui(frame: &mut ratatui::Frame, app: &App) {
    use ui::components::{filter_panel, help_menu};

    let area = frame.area();

    // If filter panel is shown, render it as an overlay
    if app.show_filter_panel {
        // Render main content first (dimmed)
        render_main_content(frame, app, area);

        // Render filter panel as centered overlay
        let popup_width = 80;
        let popup_height = 30;
        let x = (area.width.saturating_sub(popup_width)) / 2;
        let y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = ratatui::layout::Rect {
            x: area.x + x,
            y: area.y + y,
            width: popup_width.min(area.width),
            height: popup_height.min(area.height),
        };

        filter_panel::render_filter_panel(
            &app.field_schema,
            &app.filter_input,
            app.filter_error.as_deref(),
            popup_area,
            frame.buffer_mut(),
        );
    } else {
        // Normal view
        render_main_content(frame, app, area);
    }

    // Help menu has highest priority - render on top of everything
    if app.show_help {
        help_menu::render_help_menu(area, frame.buffer_mut());
    }
}

fn render_main_content(frame: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    use ui::components::{debug_logs, log_detail, log_list};

    // If debug logs are shown, split the screen
    let (main_area, debug_area) = if app.show_debug_logs {
        let chunks =
            Layout::vertical([Constraint::Percentage(70), Constraint::Percentage(30)]).split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
    };

    // Create layout based on whether detail panel is shown
    if app.show_detail_panel {
        // Split view: logs on top, detail on bottom
        let chunks = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_area);

        // Render log list
        let logs = app.current_logs();
        let title = create_log_list_title(app);
        log_list::render_log_list(
            logs,
            app.selected_index,
            title,
            chunks[0],
            frame.buffer_mut(),
        );

        // Render log detail
        let selected_log = app.selected_log();
        let total_logs = logs.len();
        log_detail::render_log_detail(
            selected_log,
            app.selected_index,
            total_logs,
            chunks[1],
            frame.buffer_mut(),
        );
    } else {
        // Full screen log list
        let logs = app.current_logs();
        let title = create_log_list_title(app);
        log_list::render_log_list(
            logs,
            app.selected_index,
            title,
            main_area,
            frame.buffer_mut(),
        );
    }

    // Render debug logs if enabled
    if let Some(debug_area) = debug_area {
        debug_logs::render_debug_logs(debug_area, frame.buffer_mut());
    }
}

fn create_log_list_title(app: &App) -> String {
    let total = app.current_logs().len();
    match &app.active_filter {
        Some(filter) => format!("Log Viewer - {} logs (Filtered: {})", total, filter),
        None => format!("Log Viewer - {} logs", total),
    }
}
