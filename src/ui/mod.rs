mod app;
pub mod components;
mod event;
pub mod terminal;

pub use app::App;
pub use event::handle_events;
pub use terminal::{cleanup_terminal, setup_terminal, Tui};
