//! SQLite persistence for bookmarks.

use super::entry::BookmarkEntry;
use crate::database::open_connection_in_memory;
use rusqlite::{Connection, Result as SqlResult, params};
use std::cell::RefCell;
use std::path::Path;

const DEFAULT_LIST_LIMIT: usize = 500;

/// Low-level bookmark table access.
pub struct BookmarkRepository {
    conn: RefCell<Connection>,
}

impl BookmarkRepository {
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

    /// Returns bookmark for URL if it already exists.
    pub fn by_url(&self, url: &str) -> SqlResult<Option<BookmarkEntry>> {
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT id, title, url, created_at FROM bookmarks
             WHERE url = ?1 LIMIT 1",
        )?;
        let mut rows = stmt.query(params![url])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_entry(row)?))
        } else {
            Ok(None)
        }
    }

    /// Inserts a bookmark and returns row id.
    pub fn insert(&self, title: &str, url: &str, created_at: i64) -> SqlResult<i64> {
        self.conn.borrow().execute(
            "INSERT INTO bookmarks (title, url, created_at) VALUES (?1, ?2, ?3)",
            params![title, url, created_at],
        )?;
        Ok(self.conn.borrow().last_insert_rowid())
    }

    /// Returns recent bookmarks newest-first.
    pub fn list_recent(&self, limit: usize) -> SqlResult<Vec<BookmarkEntry>> {
        let limit = limit.min(DEFAULT_LIST_LIMIT) as i64;
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT id, title, url, created_at FROM bookmarks
             ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], row_to_entry)?;
        rows.collect()
    }

    /// Searches title and URL with LIKE.
    pub fn search(&self, query: &str, limit: usize) -> SqlResult<Vec<BookmarkEntry>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return self.list_recent(limit);
        }
        let pattern = format!("%{}%", escape_like(trimmed));
        let limit = limit.min(DEFAULT_LIST_LIMIT) as i64;
        let conn = self.conn.borrow();
        let mut stmt = conn.prepare(
            "SELECT id, title, url, created_at FROM bookmarks
             WHERE title LIKE ?1 ESCAPE '\\' OR url LIKE ?1 ESCAPE '\\'
             ORDER BY created_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![pattern, limit], row_to_entry)?;
        rows.collect()
    }

    /// Deletes one bookmark row.
    pub fn delete(&self, id: i64) -> SqlResult<bool> {
        let changed = self
            .conn
            .borrow()
            .execute("DELETE FROM bookmarks WHERE id = ?1", params![id])?;
        Ok(changed > 0)
    }

    /// Clears all bookmarks.
    pub fn clear_all(&self) -> SqlResult<usize> {
        let changed = self.conn.borrow().execute("DELETE FROM bookmarks", [])?;
        Ok(changed)
    }
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> SqlResult<BookmarkEntry> {
    Ok(BookmarkEntry {
        id: row.get(0)?,
        title: row.get(1)?,
        url: row.get(2)?,
        created_at: row.get(3)?,
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
    fn database_creation_and_insert_bookmark() {
        let repo = BookmarkRepository::open_in_memory().expect("open");
        let id = repo
            .insert("Example", "https://example.com", now())
            .expect("insert");
        assert!(id > 0);
        assert_eq!(repo.list_recent(10).expect("list").len(), 1);
    }

    #[test]
    fn search_matches_title_and_url() {
        let repo = BookmarkRepository::open_in_memory().expect("open");
        let ts = now();
        repo.insert("Rust", "https://rust-lang.org", ts)
            .expect("insert");
        repo.insert("Example Domain", "https://example.com", ts - 1)
            .expect("insert");
        assert_eq!(repo.search("Rust", 10).expect("search").len(), 1);
        assert_eq!(repo.search("example.com", 10).expect("search").len(), 1);
    }

    #[test]
    fn delete_and_clear_all() {
        let repo = BookmarkRepository::open_in_memory().expect("open");
        let id = repo
            .insert("Ex", "https://example.com", now())
            .expect("insert");
        assert!(repo.delete(id).expect("delete"));
        assert!(repo.list_recent(10).expect("list").is_empty());

        repo.insert("A", "https://a.test", now()).expect("insert");
        repo.insert("B", "https://b.test", now()).expect("insert");
        assert_eq!(repo.clear_all().expect("clear"), 2);
    }

    #[test]
    fn open_on_disk_creates_file() {
        let dir =
            std::env::temp_dir().join(format!("kartoffelpure-bookmarks-{}", std::process::id()));
        let path = dir.join("kartoffelpure.db");
        let _ = std::fs::remove_dir_all(&dir);
        let repo = BookmarkRepository::open(&path).expect("open");
        repo.insert("Ex", "https://example.com", now())
            .expect("insert");
        assert!(path.is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
