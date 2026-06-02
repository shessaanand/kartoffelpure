//! GTK4 application UI.

mod bookmark_window;
mod browser_window;
mod history_window;
mod tab_layout;
mod tab_strip;

pub use browser_window::BrowserWindow;
pub use tab_layout::{TabLayoutMode, TabStripConfig};
