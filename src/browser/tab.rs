//! A single browser tab and its browsing surface.

use super::view::BrowserView;

/// Stable identifier for a tab within a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub u64);

/// One tab: unique id, dedicated WebView, and display metadata.
pub struct Tab {
    id: TabId,
    view: BrowserView,
    title: String,
}

impl Tab {
    /// Creates a tab with a new WebView and loads the default home page.
    pub fn new(id: TabId) -> Self {
        let view = BrowserView::default();
        view.load_home();
        Self {
            id,
            view,
            title: String::from("New Tab"),
        }
    }

    /// Returns this tab's id.
    pub fn id(&self) -> TabId {
        self.id
    }

    /// Returns the tab's WebKit view wrapper.
    pub fn view(&self) -> &BrowserView {
        &self.view
    }

    /// Returns the tab's WebKit view wrapper mutably.
    pub fn view_mut(&mut self) -> &mut BrowserView {
        &mut self.view
    }

    /// Human-readable label shown in the tab bar.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Updates the tab bar label text.
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// Name used as the child id in a `GtkStack`.
    pub fn stack_child_name(&self) -> String {
        format!("tab-{}", self.id.0)
    }
}
