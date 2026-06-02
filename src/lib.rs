//! KartoffelPure browser — GTK4 + WebKitGTK shell.

pub mod browser;
pub mod database;
pub mod ui;

pub use ui::{TabLayoutMode, TabStripConfig};

use gtk4::prelude::*;
use gtk4::{Application, gio::ApplicationFlags};

const APP_ID: &str = "com.kartoffelpure.browser";

/// Starts the GTK application main loop.
pub fn run() {
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::empty())
        .build();

    app.connect_activate(|app| {
        let window = ui::BrowserWindow::new(app);
        window.present();
    });

    let _ = app.run();
}
