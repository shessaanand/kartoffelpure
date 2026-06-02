//! SQLite persistence for browsing history.

use super::entry::HistoryEntry;
use crate::database::open_connection_in_memory;
use rusqlite::{Connection, Result as SqlResult, params};
use std::cell::RefCell;
use std::path::Path;

const DEFAULT_LIST_LIMIT: usize = 500;

/// Low-level history table access.
pub struct HistoryRepository {
    conn: RefCell<Connection>,
}

impl HistoryRepository {
    /// Opens the database at `path`, creating it if needed.
    pub fn open(path: &Path) -> SqlResult<Self> {
        let conn = crate::database::open_connection(path)?;
        Ok(Self {
            conn: RefCell::new(conn),
        })
    }

    /// Opens an in-memory database (tests).
    pub fn open_in_memory() -> SqlResult<Self> {
        Ok(Self {
            conn: RefCell::new(open_connection_in_memory()?),
        })
    }

    /// Inserts a visit and returns the new row id.
    pub fn insert(&self, url: &str, title: Option<&str>, visited_at: i64) -> SqlResult<i64> {
        self.conn.borrow().execute(
            "INSERT INTO history (url, title, visited_at) VALUES (?1, ?2, ?3)",
            params![url, title, visited_at],
        )?;
        Ok(self.conn.borrow().last_insert_rowid())
    }

    /// Returns the most recent visit for `url`, if any.
    pub fn latest_for_url(&self, url: &str) -> SqlResult<Option<HistoryEntry>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT id, url, title, visited_at FROM history
             WHERE url = ?1 ORDER BY visited_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(params![url])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_entry(row)?))
        } else {
            Ok(None)
        }
    }

    /// Returns recent history newest-first.
    pub fn list_recent(&self, limit: usize) -> SqlResult<Vec<HistoryEntry>> {
        let limit = limit.min(DEFAULT_LIST_LIMIT) as i64;
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT id, url, title, visited_at FROM history
             ORDER BY visited_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], row_to_entry)?;
        rows.collect()
    }

    /// Searches title and URL with case-insensitive LIKE.
    pub fn search(&self, query: &str, limit: usize) -> SqlResult<Vec<HistoryEntry>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return self.list_recent(limit);
        }
        let pattern = format!("%{}%", escape_like(trimmed));
        let limit = limit.min(DEFAULT_LIST_LIMIT) as i64;
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT id, url, title, visited_at FROM history
             WHERE url LIKE ?1 ESCAPE '\\' OR title LIKE ?1 ESCAPE '\\'
             ORDER BY visited_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![pattern, limit], row_to_entry)?;
        rows.collect()
    }

    /// Deletes one row. Returns whether a row was removed.
    pub fn delete(&self, id: i64) -> SqlResult<bool> {
        let changed = self
            .conn
            .borrow()
            .execute("DELETE FROM history WHERE id = ?1", params![id])?;
        Ok(changed > 0)
    }

    /// Deletes all history rows. Returns the number removed.
    pub fn clear_all(&self) -> SqlResult<usize> {
        let changed = self.conn.borrow().execute("DELETE FROM history", [])?;
        Ok(changed)
    }
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> SqlResult<HistoryEntry> {
    Ok(HistoryEntry {
        id: row.get(0)?,
        url: row.get(1)?,
        title: row.get(2)?,
        visited_at: row.get(3)?,
    })
}

fn escape_like(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_secs() as i64
    }

    #[test]
    fn database_creation_and_insertion() {
        let repo = HistoryRepository::open_in_memory().expect("open");
        let id = repo
            .insert("https://example.com", Some("Example"), now())
            .expect("insert");
        assert!(id > 0);
        let entries = repo.list_recent(10).expect("list");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].url, "https://example.com");
    }

    #[test]
    fn history_lookup_and_search() {
        let repo = HistoryRepository::open_in_memory().expect("open");
        let ts = now();
        repo.insert("https://rust-lang.org", Some("Rust"), ts)
            .expect("insert");
        repo.insert("https://example.com", Some("Example Domain"), ts - 1)
            .expect("insert");

        let latest = repo
            .latest_for_url("https://rust-lang.org")
            .expect("lookup")
            .expect("found");
        assert_eq!(latest.title.as_deref(), Some("Rust"));

        let hits = repo.search("rust", 10).expect("search");
        assert_eq!(hits.len(), 1);
        assert!(hits[0].url.contains("rust-lang.org"));

        let title_hits = repo.search("Example", 10).expect("search title");
        assert_eq!(title_hits.len(), 1);
    }

    #[test]
    fn delete_and_clear_all() {
        let repo = HistoryRepository::open_in_memory().expect("open");
        let id = repo
            .insert("https://example.com", None, now())
            .expect("insert");
        assert!(repo.delete(id).expect("delete"));
        assert!(repo.list_recent(10).expect("list").is_empty());

        repo.insert("https://a.test", None, now()).expect("insert");
        repo.insert("https://b.test", None, now()).expect("insert");
        assert_eq!(repo.clear_all().expect("clear"), 2);
        assert!(repo.list_recent(10).expect("list").is_empty());
    }

    #[test]
    fn open_on_disk_creates_file() {
        let dir = std::env::temp_dir().join(format!("kartoffelpure-test-{}", std::process::id()));
        let path = dir.join("kartoffelpure.db");
        let _ = std::fs::remove_dir_all(&dir);
        let repo = HistoryRepository::open(&path).expect("open");
        repo.insert("https://example.com", Some("Ex"), now())
            .expect("insert");
        assert!(path.is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
