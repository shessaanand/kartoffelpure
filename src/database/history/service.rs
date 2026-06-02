//! High-level history recording with deduplication.

use super::entry::HistoryEntry;
use super::repository::HistoryRepository;
use crate::database::default_database_path;
use rusqlite::Result as SqlResult;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_DEDUPE_SECS: i64 = 60;
const LIST_LIMIT: usize = 500;

/// Browser-facing history API.
pub struct HistoryService {
    repository: HistoryRepository,
    dedupe_window_secs: i64,
}

impl HistoryService {
    /// Opens the default user database.
    pub fn open_default() -> SqlResult<Self> {
        Self::open(default_database_path())
    }

    /// Opens history storage at `path`.
    pub fn open(path: impl AsRef<Path>) -> SqlResult<Self> {
        Ok(Self {
            repository: HistoryRepository::open(path.as_ref())?,
            dedupe_window_secs: DEFAULT_DEDUPE_SECS,
        })
    }

    /// Wraps an existing repository (tests).
    pub fn from_repository(repository: HistoryRepository) -> Self {
        Self {
            repository,
            dedupe_window_secs: DEFAULT_DEDUPE_SECS,
        }
    }

    /// Records a successful page visit. Returns `None` when deduplicated.
    pub fn record_visit(&self, url: &str, title: Option<&str>) -> SqlResult<Option<i64>> {
        if !should_record_url(url) {
            return Ok(None);
        }
        let visited_at = unix_now();
        if let Some(latest) = self.repository.latest_for_url(url)?
            && visited_at - latest.visited_at < self.dedupe_window_secs
        {
            return Ok(None);
        }
        let title = title.filter(|t| !t.is_empty());
        let id = self.repository.insert(url, title, visited_at)?;
        Ok(Some(id))
    }

    /// Returns recent history.
    pub fn list_recent(&self, limit: usize) -> SqlResult<Vec<HistoryEntry>> {
        self.repository.list_recent(limit)
    }

    /// Searches history by URL and title.
    pub fn search(&self, query: &str) -> SqlResult<Vec<HistoryEntry>> {
        self.repository.search(query, LIST_LIMIT)
    }

    /// Deletes one entry.
    pub fn delete_entry(&self, id: i64) -> SqlResult<bool> {
        self.repository.delete(id)
    }

    /// Removes all history entries.
    pub fn clear_all(&self) -> SqlResult<usize> {
        self.repository.clear_all()
    }
}

fn should_record_url(url: &str) -> bool {
    let trimmed = url.trim();
    !trimmed.is_empty()
        && !trimmed.starts_with("about:")
        && !trimmed.starts_with("data:")
        && !trimmed.starts_with("javascript:")
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
    use crate::database::history::HistoryRepository;

    #[test]
    fn deduplicates_rapid_reloads() {
        let repo = HistoryRepository::open_in_memory().expect("open");
        let service = HistoryService::from_repository(repo);
        let first = service
            .record_visit("https://example.com", Some("Example"))
            .expect("record");
        assert!(first.is_some());
        let second = service
            .record_visit("https://example.com", Some("Example"))
            .expect("record");
        assert!(second.is_none());
        assert_eq!(service.list_recent(10).expect("list").len(), 1);
    }

    #[test]
    fn skips_non_http_urls() {
        let repo = HistoryRepository::open_in_memory().expect("open");
        let service = HistoryService::from_repository(repo);
        assert!(
            service
                .record_visit("about:blank", None)
                .expect("record")
                .is_none()
        );
    }
}
