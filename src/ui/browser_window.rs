//! Main browser window: toolbar plus WebKit view.

use crate::browser::{BrowserView, normalize_url};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Button, Entry, Orientation};
use webkit6::prelude::*;
use webkit6::{LoadEvent, WebView};

/// GTK application window hosting the v0.1 browser shell.
pub struct BrowserWindow {
    window: ApplicationWindow,
}

impl BrowserWindow {
    /// Builds the window, toolbar, and web view for `app`.
    pub fn new(app: &Application) -> Self {
        let browser = BrowserView::new();
        let webview = browser.widget().clone();

        let back_button = Button::with_mnemonic("_Back");
        let forward_button = Button::with_mnemonic("_Forward");
        let reload_button = Button::with_mnemonic("_Reload");

        let address_entry = Entry::builder()
            .placeholder_text("Enter URL")
            .hexpand(true)
            .build();

        let toolbar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(6)
            .margin_end(6)
            .build();
        toolbar.append(&back_button);
        toolbar.append(&forward_button);
        toolbar.append(&reload_button);
        toolbar.append(&address_entry);

        let content = GtkBox::new(Orientation::Vertical, 0);
        content.append(&toolbar);
        content.append(browser.widget());

        let window = ApplicationWindow::builder()
            .application(app)
            .title("KartoffelPure")
            .default_width(1200)
            .default_height(800)
            .child(&content)
            .build();

        let update_navigation_buttons =
            |webview: &WebView, back: &Button, forward: &Button| {
                back.set_sensitive(webview.can_go_back());
                forward.set_sensitive(webview.can_go_forward());
            };

        let sync_address_bar = |webview: &WebView, entry: &Entry| {
            if entry.has_focus() {
                return;
            }
            if let Some(uri) = webview.uri() {
                entry.set_text(&uri);
            }
        };

        // Initial toolbar state and address bar text.
        update_navigation_buttons(&webview, &back_button, &forward_button);
        address_entry.set_text(crate::browser::DEFAULT_HOME_URL);

        // --- Navigation buttons ---
        {
            let webview = webview.clone();
            let back = back_button.clone();
            let forward = forward_button.clone();
            back_button.connect_clicked(move |_| {
                if webview.can_go_back() {
                    webview.go_back();
                }
                update_navigation_buttons(&webview, &back, &forward);
            });
        }

        {
            let webview = webview.clone();
            let back = back_button.clone();
            let forward = forward_button.clone();
            forward_button.connect_clicked(move |_| {
                if webview.can_go_forward() {
                    webview.go_forward();
                }
                update_navigation_buttons(&webview, &back, &forward);
            });
        }

        {
            let webview = webview.clone();
            reload_button.connect_clicked(move |_| {
                webview.reload();
            });
        }

        // --- Address bar ---
        {
            let webview = webview.clone();
            address_entry.connect_activate(move |entry| {
                let url = normalize_url(&entry.text());
                webview.load_uri(&url);
                entry.set_text(&url);
            });
        }

        // --- WebView signals ---
        {
            let entry = address_entry.clone();
            webview.connect_uri_notify(move |wv| {
                sync_address_bar(wv, &entry);
            });
        }

        {
            let entry = address_entry.clone();
            let back = back_button.clone();
            let forward = forward_button.clone();
            webview.connect_load_changed(move |wv, event| {
                if event == LoadEvent::Finished {
                    sync_address_bar(wv, &entry);
                    update_navigation_buttons(wv, &back, &forward);
                }
            });
        }

        webview.connect_load_failed(|_wv, _event, failing_uri, error| {
            eprintln!("load failed for {failing_uri}: {error}");
            false
        });

        browser.load_home();

        Self { window }
    }

    /// Shows the window.
    pub fn present(&self) {
        self.window.present();
    }
}
