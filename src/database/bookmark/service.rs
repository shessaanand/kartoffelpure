//! High-level bookmark service.

use super::entry::BookmarkEntry;
use super::repository::BookmarkRepository;
use crate::browser::normalize_url;
use crate::database::default_database_path;
use rusqlite::Result as SqlResult;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const LIST_LIMIT: usize = 500;

/// Result of attempting to create a bookmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddBookmarkResult {
    Added(i64),
    Duplicate,
    InvalidUrl,
}

/// Browser-facing bookmark API.
pub struct BookmarkService {
    repository: BookmarkRepository,
}

impl BookmarkService {
    /// Opens default user DB.
    pub fn open_default() -> SqlResult<Self> {
        Self::open(default_database_path())
    }

    /// Opens bookmark storage at `path`.
    pub fn open(path: impl AsRef<Path>) -> SqlResult<Self> {
        Ok(Self {
            repository: BookmarkRepository::open(path.as_ref())?,
        })
    }

    /// Wraps existing repository (tests).
    pub fn from_repository(repository: BookmarkRepository) -> Self {
        Self { repository }
    }

    /// Adds bookmark from raw title + URL, normalizing URL and preventing duplicates.
    pub fn add_bookmark(&self, title: Option<&str>, raw_url: &str) -> SqlResult<AddBookmarkResult> {
        let trimmed = raw_url.trim();
        if trimmed.is_empty() {
            return Ok(AddBookmarkResult::InvalidUrl);
        }
        if is_disallowed_scheme(trimmed) {
            return Ok(AddBookmarkResult::InvalidUrl);
        }
        let normalized_url = normalize_url(trimmed);
        if !is_valid_bookmark_url(&normalized_url) {
            return Ok(AddBookmarkResult::InvalidUrl);
        }
        if self.repository.by_url(&normalized_url)?.is_some() {
            return Ok(AddBookmarkResult::Duplicate);
        }
        let bookmark_title = title
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .unwrap_or(&normalized_url);
        let id = self
            .repository
            .insert(bookmark_title, &normalized_url, unix_now())?;
        Ok(AddBookmarkResult::Added(id))
    }

    /// Returns recent bookmarks.
    pub fn list_recent(&self, limit: usize) -> SqlResult<Vec<BookmarkEntry>> {
        self.repository.list_recent(limit)
    }

    /// Searches bookmarks by title/url.
    pub fn search(&self, query: &str) -> SqlResult<Vec<BookmarkEntry>> {
        self.repository.search(query, LIST_LIMIT)
    }

    /// Deletes one bookmark.
    pub fn delete_entry(&self, id: i64) -> SqlResult<bool> {
        self.repository.delete(id)
    }

    /// Clears all bookmarks.
    pub fn clear_all(&self) -> SqlResult<usize> {
        self.repository.clear_all()
    }
}

fn is_valid_bookmark_url(url: &str) -> bool {
    !url.is_empty() && !is_disallowed_scheme(url)
}

fn is_disallowed_scheme(url: &str) -> bool {
    let lowered = url.to_ascii_lowercase();
    lowered.starts_with("about:")
        || lowered.starts_with("data:")
        || lowered.starts_with("javascript:")
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::bookmark::repository::BookmarkRepository;

    #[test]
    fn prevents_duplicate_urls() {
        let repo = BookmarkRepository::open_in_memory().expect("open");
        let service = BookmarkService::from_repository(repo);
        let first = service
            .add_bookmark(Some("Example"), "example.com")
            .expect("insert");
        assert!(matches!(first, AddBookmarkResult::Added(_)));
        let second = service
            .add_bookmark(Some("Duplicate"), "https://example.com")
            .expect("insert");
        assert_eq!(second, AddBookmarkResult::Duplicate);
    }

    #[test]
    fn rejects_invalid_url_schemes() {
        let repo = BookmarkRepository::open_in_memory().expect("open");
        let service = BookmarkService::from_repository(repo);
        assert_eq!(
            service
                .add_bookmark(Some("Bad"), "about:blank")
                .expect("add"),
            AddBookmarkResult::InvalidUrl
        );
    }
}
