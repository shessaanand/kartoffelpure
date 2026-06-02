//! History row model.

/// One persisted history visit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub visited_at: i64,
}

impl HistoryEntry {
    /// Display title, falling back to the URL.
    pub fn display_title(&self) -> &str {
        self.title
            .as_deref()
            .filter(|t| !t.is_empty())
            .unwrap_or(&self.url)
    }
}
