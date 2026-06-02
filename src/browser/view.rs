//! WebKit `WebView` wrapper for a single browsing surface.

use webkit6::WebView;
use webkit6::prelude::*;

/// Default page loaded on startup.
pub const DEFAULT_HOME_URL: &str = "https://example.com";

/// Owns the WebKit view used for rendering pages.
pub struct BrowserView {
    webview: WebView,
}

impl Default for BrowserView {
    fn default() -> Self {
        let webview = WebView::new();
        webview.set_hexpand(true);
        webview.set_vexpand(true);
        Self { webview }
    }
}

impl BrowserView {
    /// Returns the underlying GTK widget.
    pub fn widget(&self) -> &WebView {
        &self.webview
    }

    /// Loads the default home page.
    pub fn load_home(&self) {
        self.load_url(DEFAULT_HOME_URL);
    }

    /// Requests navigation to `url` (caller should normalize user input first).
    pub fn load_url(&self, url: &str) {
        self.webview.load_uri(url);
    }

    /// Steps back in session history when possible.
    pub fn go_back(&self) {
        if self.webview.can_go_back() {
            self.webview.go_back();
        }
    }

    /// Steps forward in session history when possible.
    pub fn go_forward(&self) {
        if self.webview.can_go_forward() {
            self.webview.go_forward();
        }
    }

    /// Reloads the current page.
    pub fn reload(&self) {
        self.webview.reload();
    }

    /// Whether back navigation is available.
    pub fn can_go_back(&self) -> bool {
        self.webview.can_go_back()
    }

    /// Whether forward navigation is available.
    pub fn can_go_forward(&self) -> bool {
        self.webview.can_go_forward()
    }
}

/// Turns user input into a loadable URI string.
///
/// Bare hostnames get `https://`. Strings that already contain a scheme are
/// trimmed and passed through unchanged.
pub fn normalize_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return DEFAULT_HOME_URL.to_string();
    }
    if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_bare_host() {
        assert_eq!(normalize_url("example.org"), "https://example.org");
    }

    #[test]
    fn normalize_preserves_scheme() {
        assert_eq!(normalize_url("http://example.org"), "http://example.org");
    }

    #[test]
    fn normalize_empty_uses_home() {
        assert_eq!(normalize_url("   "), DEFAULT_HOME_URL);
    }
}
