//! Main browser window: tab strip, toolbar, and stacked WebKit views.

use crate::browser::{TabId, TabManager, normalize_url};
use crate::database::bookmark::repository::BookmarkRepository;
use crate::database::bookmark::service::{AddBookmarkResult, BookmarkService};
use crate::database::history::{HistoryRepository, HistoryService};
use crate::ui::bookmark_window::BookmarkDialog;
use crate::ui::history_window::HistoryDialog;
use crate::ui::tab_layout::{TabLayoutMode, TabStripConfig};
use crate::ui::tab_strip::TabStrip;
use gtk4::gdk::{Key, ModifierType};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Entry, EventControllerKey, Orientation,
    Stack, StackTransitionType,
};
use std::cell::RefCell;
use std::rc::Rc;
use webkit6::prelude::*;
use webkit6::{LoadEvent, WebView};

/// Shared window state used by GTK signal handlers.
struct WindowState {
    window: ApplicationWindow,
    tab_manager: TabManager,
    stack: Stack,
    tab_strip: TabStrip,
    browser_column: GtkBox,
    bookmarks: Rc<BookmarkService>,
    history: Rc<HistoryService>,
    address_entry: Entry,
    back_button: Button,
    forward_button: Button,
    reload_button: Button,
    bookmark_button: Button,
    bookmarks_button: Button,
    history_button: Button,
}

/// GTK application window hosting the browser shell.
pub struct BrowserWindow {
    window: ApplicationWindow,
    state: Rc<RefCell<WindowState>>,
}

impl BrowserWindow {
    /// Builds the window with default horizontal tab layout.
    pub fn new(app: &Application) -> Self {
        Self::with_tab_config(app, TabStripConfig::default())
    }

    /// Builds the window using the given tab strip configuration.
    pub fn with_tab_config(app: &Application, tab_config: TabStripConfig) -> Self {
        let back_button = Button::with_mnemonic("_Back");
        let forward_button = Button::with_mnemonic("_Forward");
        let reload_button = Button::with_mnemonic("_Reload");
        let bookmark_button = Button::with_mnemonic("_Bookmark");
        let bookmarks_button = Button::with_mnemonic("Bookmar_ks");
        let history_button = Button::with_mnemonic("_History");

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
        toolbar.append(&bookmark_button);
        toolbar.append(&bookmarks_button);
        toolbar.append(&history_button);
        toolbar.append(&address_entry);

        let stack = Stack::builder()
            .vexpand(true)
            .hexpand(true)
            .transition_type(StackTransitionType::None)
            .build();

        let browser_column = GtkBox::new(Orientation::Vertical, 0);
        browser_column.append(&toolbar);
        browser_column.append(&stack);

        let tab_strip = TabStrip::new(tab_config);
        let chrome_root = Self::build_chrome_root(&tab_strip, &browser_column);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("KartoffelPure")
            .default_width(1200)
            .default_height(800)
            .child(&chrome_root)
            .build();

        let history = Rc::new(HistoryService::open_default().unwrap_or_else(|err| {
            eprintln!("failed to open history database: {err}; using in-memory store");
            HistoryService::from_repository(
                HistoryRepository::open_in_memory().expect("in-memory history"),
            )
        }));
        let bookmarks = Rc::new(BookmarkService::open_default().unwrap_or_else(|err| {
            eprintln!("failed to open bookmark database: {err}; using in-memory store");
            BookmarkService::from_repository(
                BookmarkRepository::open_in_memory().expect("in-memory bookmarks"),
            )
        }));

        let state = Rc::new(RefCell::new(WindowState {
            window: window.clone(),
            tab_manager: TabManager::default(),
            stack,
            tab_strip,
            browser_column,
            bookmarks,
            history,
            address_entry,
            back_button,
            forward_button,
            reload_button,
            bookmark_button,
            bookmarks_button,
            history_button,
        }));

        Self::wire_toolbar(Rc::clone(&state));
        Self::wire_bookmark_button(Rc::clone(&state));
        Self::wire_bookmarks_button(Rc::clone(&state));
        Self::wire_history_button(Rc::clone(&state));
        Self::wire_keyboard_shortcuts(&window, Rc::clone(&state));
        Self::wire_new_tab_buttons(Rc::clone(&state));

        Self::open_tab(Rc::clone(&state));

        Self { window, state }
    }

    /// Shows the window.
    pub fn present(&self) {
        self.window.present();
    }

    /// Switches tab presentation between horizontal and vertical layouts.
    pub fn set_tab_layout_mode(&self, mode: TabLayoutMode) {
        let mut s = self.state.borrow_mut();
        s.tab_strip.set_layout_mode(mode);
        Self::rebuild_chrome_layout(&mut s);
    }

    /// Returns the current tab layout mode.
    pub fn tab_layout_mode(&self) -> TabLayoutMode {
        self.state.borrow().tab_strip.layout_mode()
    }

    fn build_chrome_root(tab_strip: &TabStrip, browser_column: &GtkBox) -> GtkBox {
        match tab_strip.layout_mode() {
            TabLayoutMode::Horizontal => {
                let root = GtkBox::new(Orientation::Vertical, 0);
                root.append(tab_strip.horizontal_strip());
                root.append(browser_column);
                root
            }
            TabLayoutMode::Vertical => {
                let root = GtkBox::new(Orientation::Horizontal, 0);
                root.append(tab_strip.vertical_sidebar());
                browser_column.set_hexpand(true);
                browser_column.set_vexpand(true);
                root.append(browser_column);
                root
            }
        }
    }

    fn rebuild_chrome_layout(state: &mut WindowState) {
        if let Some(parent) = state.browser_column.parent()
            && let Ok(box_parent) = parent.downcast::<GtkBox>()
        {
            box_parent.remove(&state.browser_column);
        }
        let chrome_root = Self::build_chrome_root(&state.tab_strip, &state.browser_column);
        state.window.set_child(Some(&chrome_root));
    }

    fn wire_new_tab_buttons(state: Rc<RefCell<WindowState>>) {
        let state_for_click = Rc::clone(&state);
        state.borrow().tab_strip.for_each_new_tab_button(|btn| {
            let state = Rc::clone(&state_for_click);
            btn.connect_clicked(move |_| Self::open_tab(Rc::clone(&state)));
        });
    }

    fn wire_history_button(state: Rc<RefCell<WindowState>>) {
        let state_for_click = Rc::clone(&state);
        let history_button = state.borrow().history_button.clone();
        history_button.connect_clicked(move |_| {
            let history = Rc::clone(&state_for_click.borrow().history);
            let window = state_for_click.borrow().window.clone();
            let navigate_state = Rc::clone(&state_for_click);
            HistoryDialog::show(Some(&window), history, move |url| {
                let s = navigate_state.borrow();
                if let Some(tab) = s.tab_manager.active_tab() {
                    tab.view().widget().load_uri(&url);
                }
            });
        });
    }

    fn wire_bookmark_button(state: Rc<RefCell<WindowState>>) {
        let state_for_click = Rc::clone(&state);
        let bookmark_button = state.borrow().bookmark_button.clone();
        bookmark_button.connect_clicked(move |_| {
            let (title, url, bookmarks) = {
                let s = state_for_click.borrow();
                let Some(tab) = s.tab_manager.active_tab() else {
                    return;
                };
                let webview = tab.view().widget();
                let Some(uri) = webview.uri() else {
                    eprintln!("bookmark skipped: active tab has no URL");
                    return;
                };
                (
                    webview.title().map(|t| t.to_string()),
                    normalize_url(&uri),
                    Rc::clone(&s.bookmarks),
                )
            };

            match bookmarks.add_bookmark(title.as_deref(), &url) {
                Ok(AddBookmarkResult::Added(_)) => {
                    eprintln!("bookmark saved: {url}");
                }
                Ok(AddBookmarkResult::Duplicate) => {
                    eprintln!("bookmark already exists: {url}");
                }
                Ok(AddBookmarkResult::InvalidUrl) => {
                    eprintln!("bookmark skipped: invalid URL");
                }
                Err(err) => {
                    eprintln!("bookmark save failed: {err}");
                }
            }
        });
    }

    fn wire_bookmarks_button(state: Rc<RefCell<WindowState>>) {
        let state_for_click = Rc::clone(&state);
        let bookmarks_button = state.borrow().bookmarks_button.clone();
        bookmarks_button.connect_clicked(move |_| {
            let bookmarks = Rc::clone(&state_for_click.borrow().bookmarks);
            let window = state_for_click.borrow().window.clone();
            let navigate_state = Rc::clone(&state_for_click);
            BookmarkDialog::show(Some(&window), bookmarks, move |url| {
                let s = navigate_state.borrow();
                if let Some(tab) = s.tab_manager.active_tab() {
                    tab.view().widget().load_uri(&normalize_url(&url));
                }
            });
        });
    }

    fn wire_toolbar(state: Rc<RefCell<WindowState>>) {
        {
            let state = Rc::clone(&state);
            let back_button = state.borrow().back_button.clone();
            back_button.connect_clicked(move |_| {
                let webview = {
                    let s = state.borrow();
                    s.tab_manager
                        .active_tab()
                        .map(|t| t.view().widget().clone())
                };
                if let Some(webview) = webview {
                    if webview.can_go_back() {
                        webview.go_back();
                    }
                    Self::sync_chrome(&state);
                }
            });
        }

        {
            let state = Rc::clone(&state);
            let forward_button = state.borrow().forward_button.clone();
            forward_button.connect_clicked(move |_| {
                let webview = {
                    let s = state.borrow();
                    s.tab_manager
                        .active_tab()
                        .map(|t| t.view().widget().clone())
                };
                if let Some(webview) = webview {
                    if webview.can_go_forward() {
                        webview.go_forward();
                    }
                    Self::sync_chrome(&state);
                }
            });
        }

        {
            let state = Rc::clone(&state);
            let reload_button = state.borrow().reload_button.clone();
            reload_button.connect_clicked(move |_| {
                let s = state.borrow();
                if let Some(tab) = s.tab_manager.active_tab() {
                    tab.view().widget().reload();
                }
            });
        }

        {
            let state = Rc::clone(&state);
            let address_entry = state.borrow().address_entry.clone();
            address_entry.connect_activate(move |entry| {
                let url = normalize_url(&entry.text());
                let s = state.borrow();
                if let Some(tab) = s.tab_manager.active_tab() {
                    tab.view().widget().load_uri(&url);
                    entry.set_text(&url);
                }
            });
        }
    }

    fn wire_keyboard_shortcuts(window: &ApplicationWindow, state: Rc<RefCell<WindowState>>) {
        let controller = EventControllerKey::new();
        controller.connect_key_pressed(move |_, key, _, modifiers| {
            let ctrl = modifiers.contains(ModifierType::CONTROL_MASK);
            if ctrl && key == Key::t {
                Self::open_tab(Rc::clone(&state));
                return gtk4::glib::Propagation::Stop;
            }
            if ctrl && key == Key::w {
                if let Some(id) = state.borrow().tab_manager.active_id() {
                    Self::close_tab(Rc::clone(&state), id);
                }
                return gtk4::glib::Propagation::Stop;
            }
            gtk4::glib::Propagation::Proceed
        });
        window.add_controller(controller);
    }

    fn open_tab(state: Rc<RefCell<WindowState>>) {
        let tab_id = {
            let mut s = state.borrow_mut();
            let tab_id = s.tab_manager.create_tab();
            let tab = s.tab_manager.tab(tab_id).expect("tab created");
            let name = tab.stack_child_name();
            s.stack.add_named(tab.view().widget(), Some(&name));
            tab_id
        };

        Self::register_tab_ui(Rc::clone(&state), tab_id);
        Self::switch_tab(Rc::clone(&state), tab_id);
    }

    fn register_tab_ui(state: Rc<RefCell<WindowState>>, tab_id: TabId) {
        let title = state
            .borrow()
            .tab_manager
            .tab(tab_id)
            .map(|t| t.title().to_string())
            .unwrap_or_else(|| String::from("New Tab"));

        let select_state = Rc::clone(&state);
        let close_state = Rc::clone(&state);

        state.borrow_mut().tab_strip.add_tab(
            tab_id,
            &title,
            move |id| Self::switch_tab(Rc::clone(&select_state), id),
            move |id| Self::close_tab(Rc::clone(&close_state), id),
        );

        Self::wire_tab_webview_signals(Rc::clone(&state), tab_id);
    }

    fn wire_tab_webview_signals(state: Rc<RefCell<WindowState>>, tab_id: TabId) {
        let webview = state
            .borrow()
            .tab_manager
            .tab(tab_id)
            .expect("tab exists")
            .view()
            .widget()
            .clone();

        {
            let state = Rc::clone(&state);
            webview.connect_uri_notify(move |wv| {
                Self::on_tab_uri_changed(&state, tab_id, wv);
            });
        }

        {
            let state = Rc::clone(&state);
            webview.connect_title_notify(move |wv| {
                Self::on_tab_title_changed(&state, tab_id, wv);
            });
        }

        {
            let state = Rc::clone(&state);
            webview.connect_load_changed(move |wv, event| {
                if event == LoadEvent::Finished {
                    Self::on_tab_load_finished(&state, tab_id, wv);
                }
            });
        }

        webview.connect_load_failed(|_wv, _event, failing_uri, error| {
            eprintln!("load failed for {failing_uri}: {error}");
            false
        });
    }

    fn on_tab_uri_changed(state: &Rc<RefCell<WindowState>>, tab_id: TabId, webview: &WebView) {
        if state.borrow().tab_manager.active_id() != Some(tab_id) {
            return;
        }
        let entry = state.borrow().address_entry.clone();
        Self::sync_address_bar(webview, &entry);
    }

    fn on_tab_title_changed(state: &Rc<RefCell<WindowState>>, tab_id: TabId, webview: &WebView) {
        let title = webview
            .title()
            .map(|t| t.to_string())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| String::from("New Tab"));

        {
            let mut s = state.borrow_mut();
            if let Some(tab) = s.tab_manager.tab_mut(tab_id) {
                tab.set_title(title.clone());
            }
            s.tab_strip.set_tab_title(tab_id, &title);
        }

        if state.borrow().tab_manager.active_id() == Some(tab_id) {
            Self::set_window_title(state, &title);
        }
    }

    fn on_tab_load_finished(state: &Rc<RefCell<WindowState>>, tab_id: TabId, webview: &WebView) {
        Self::record_history(state, webview);
        if state.borrow().tab_manager.active_id() != Some(tab_id) {
            return;
        }
        Self::sync_chrome(state);
        if let Some(title) = webview.title().map(|t| t.to_string())
            && !title.is_empty()
        {
            Self::set_window_title(state, &title);
        }
    }

    fn record_history(state: &Rc<RefCell<WindowState>>, webview: &WebView) {
        let Some(uri) = webview.uri() else {
            return;
        };
        let url = uri.to_string();
        let title = webview.title().map(|t| t.to_string());
        if let Err(err) = state.borrow().history.record_visit(&url, title.as_deref()) {
            eprintln!("history record failed: {err}");
        }
    }

    fn switch_tab(state: Rc<RefCell<WindowState>>, tab_id: TabId) {
        {
            let mut s = state.borrow_mut();
            if !s.tab_manager.set_active(tab_id) {
                return;
            }
            let name = s
                .tab_manager
                .tab(tab_id)
                .expect("active tab")
                .stack_child_name();
            s.stack.set_visible_child_name(&name);
        }

        Self::highlight_active_tab(&state, tab_id);
        Self::sync_chrome(&state);
    }

    fn close_tab(state: Rc<RefCell<WindowState>>, tab_id: TabId) {
        let new_active = {
            let mut s = state.borrow_mut();
            let Some(tab) = s.tab_manager.tab(tab_id) else {
                return;
            };
            let stack_name = tab.stack_child_name();
            let Some(new_active) = s.tab_manager.close_tab(tab_id) else {
                return;
            };
            s.tab_strip.remove_tab(tab_id);
            if let Some(child) = s.stack.child_by_name(&stack_name) {
                s.stack.remove(&child);
            }
            new_active
        };

        Self::switch_tab(state, new_active);
    }

    fn sync_chrome(state: &Rc<RefCell<WindowState>>) {
        let s = state.borrow();
        let Some(tab) = s.tab_manager.active_tab() else {
            return;
        };
        let webview = tab.view().widget();
        Self::update_navigation_buttons(webview, &s.back_button, &s.forward_button);
        Self::sync_address_bar(webview, &s.address_entry);
        s.tab_strip.set_tab_title(tab.id(), tab.title());
        let title = tab.title().to_string();
        drop(s);
        Self::set_window_title(state, &title);
    }

    fn highlight_active_tab(state: &Rc<RefCell<WindowState>>, tab_id: TabId) {
        state.borrow().tab_strip.set_active_tab(tab_id);
    }

    fn set_window_title(state: &Rc<RefCell<WindowState>>, page_title: &str) {
        let title = if page_title.is_empty() {
            String::from("KartoffelPure")
        } else {
            format!("{page_title} — KartoffelPure")
        };
        state.borrow().window.set_title(Some(title.as_str()));
    }

    fn update_navigation_buttons(webview: &WebView, back: &Button, forward: &Button) {
        back.set_sensitive(webview.can_go_back());
        forward.set_sensitive(webview.can_go_forward());
    }

    fn sync_address_bar(webview: &WebView, entry: &Entry) {
        if entry.has_focus() {
            return;
        }
        if let Some(uri) = webview.uri() {
            entry.set_text(&uri);
        }
    }
}
