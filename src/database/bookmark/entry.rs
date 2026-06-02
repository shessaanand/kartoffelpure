//! Bookmark row model.

/// One persisted bookmark.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkEntry {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub created_at: i64,
}
