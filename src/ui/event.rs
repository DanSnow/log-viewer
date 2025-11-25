use crate::error::Result;
use crate::ui::app::{App, Focus};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Handle keyboard events for the application
pub fn handle_events(app: &mut App, page_height: usize) -> Result<()> {
    // Poll for events with a timeout
    if event::poll(Duration::from_millis(100))
        .map_err(crate::error::LogViewerError::from)?
    {
        if let Event::Key(key) = event::read().map_err(crate::error::LogViewerError::from)? {
            handle_key_event(app, key, page_height)?;
        }
    }
    Ok(())
}

/// Handle a single key event
fn handle_key_event(app: &mut App, key: KeyEvent, page_height: usize) -> Result<()> {
    // Help menu has priority - if it's shown, only handle keys that close it
    if app.show_help {
        match key.code {
            KeyCode::Char('?') | KeyCode::Esc => {
                app.toggle_help();
            }
            _ => {}
        }
        return Ok(());
    }

    // Handle keys based on current focus
    match app.focus {
        Focus::LogList => handle_log_list_keys(app, key, page_height),
        Focus::FilterInput => handle_filter_input_keys(app, key),
        Focus::FilterPresets => handle_filter_presets_keys(app, key),
    }
}

/// Handle keys when focus is on the log list (normal mode)
fn handle_log_list_keys(app: &mut App, key: KeyEvent, page_height: usize) -> Result<()> {
    match key.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Esc => {
            app.quit();
        }

        // Navigation - vim style
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
        }

        // Jump to first/last - vim style
        KeyCode::Char('g') if matches!(key.modifiers, KeyModifiers::NONE) => {
            app.jump_to_first();
        }
        KeyCode::Char('G') => {
            app.jump_to_last();
        }

        // Page scrolling
        KeyCode::Char('d') if matches!(key.modifiers, KeyModifiers::CONTROL) => {
            app.scroll_down_half_page(page_height);
        }
        KeyCode::Char('u') if matches!(key.modifiers, KeyModifiers::CONTROL) => {
            app.scroll_up_half_page(page_height);
        }
        KeyCode::Char('f') if matches!(key.modifiers, KeyModifiers::CONTROL) => {
            app.scroll_down_page(page_height);
        }
        KeyCode::Char('b') if matches!(key.modifiers, KeyModifiers::CONTROL) => {
            app.scroll_up_page(page_height);
        }
        KeyCode::PageDown => {
            app.scroll_down_page(page_height);
        }
        KeyCode::PageUp => {
            app.scroll_up_page(page_height);
        }

        // Toggle detail panel
        KeyCode::Char('d') if matches!(key.modifiers, KeyModifiers::NONE) => {
            app.toggle_detail_panel();
        }

        // Toggle filter panel
        KeyCode::Char('f') if matches!(key.modifiers, KeyModifiers::NONE) => {
            app.toggle_filter_panel();
        }

        // Focus filter input
        KeyCode::Char('/') => {
            app.focus_filter();
        }

        // Clear filter
        KeyCode::Char('c') => {
            app.clear_filter();
        }

        // Toggle help
        KeyCode::Char('?') => {
            app.toggle_help();
        }

        _ => {}
    }

    Ok(())
}

/// Handle keys when focus is on filter input
fn handle_filter_input_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        // Exit filter input back to presets
        KeyCode::Esc => {
            app.focus = Focus::FilterPresets;
        }

        // Apply filter
        KeyCode::Enter => {
            let _ = app.apply_filter();
        }

        // Pass other keys to the text area widget
        _ => {
            app.filter_input.input(key);
        }
    }

    Ok(())
}

/// Handle keys when focus is on filter presets (not text input)
fn handle_filter_presets_keys(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        // Close filter panel
        KeyCode::Esc => {
            app.toggle_filter_panel();
        }

        // Preset filters (number keys)
        KeyCode::Char('1') => {
            let _ = app.apply_preset_filter("level >= 50");
        }
        KeyCode::Char('2') => {
            let _ = app.apply_preset_filter("level >= 40");
        }
        KeyCode::Char('3') => {
            // Last hour - calculate timestamp
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;
            let one_hour_ago = now - (60 * 60 * 1000);
            let filter = format!("time >= {}", one_hour_ago);
            let _ = app.apply_preset_filter(&filter);
        }

        // Any printable character - switch to input mode and type it
        KeyCode::Char(_) => {
            app.focus = Focus::FilterInput;
            app.filter_input.input(key);
        }

        // Special navigation keys that should also enter input mode
        KeyCode::Backspace | KeyCode::Delete | KeyCode::Left | KeyCode::Right | KeyCode::Home
        | KeyCode::End => {
            app.focus = Focus::FilterInput;
            app.filter_input.input(key);
        }

        // Ignore other keys (Tab, F-keys, etc.)
        _ => {}
    }

    Ok(())
}
