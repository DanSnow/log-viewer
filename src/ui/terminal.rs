use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

use crate::error::Result;

pub type Tui = Terminal<CrosstermBackend<io::Stdout>>;

/// Setup terminal for TUI mode
pub fn setup_terminal() -> Result<Tui> {
    enable_raw_mode().map_err(crate::error::LogViewerError::from)?;
    execute!(io::stdout(), EnterAlternateScreen).map_err(crate::error::LogViewerError::from)?;

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend).map_err(crate::error::LogViewerError::from)?;

    Ok(terminal)
}

/// Restore terminal to normal mode
pub fn cleanup_terminal() -> Result<()> {
    disable_raw_mode().map_err(crate::error::LogViewerError::from)?;
    execute!(io::stdout(), LeaveAlternateScreen).map_err(crate::error::LogViewerError::from)?;

    Ok(())
}
