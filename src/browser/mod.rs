//! WebKitGTK integration and navigation helpers.

mod tab;
mod tab_manager;
mod view;

pub use tab::{Tab, TabId};
pub use tab_manager::TabManager;
pub use view::{BrowserView, DEFAULT_HOME_URL, normalize_url};
