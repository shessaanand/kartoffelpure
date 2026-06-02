//! SQLite database access.

mod path;

pub mod bookmark;
pub mod history;

pub use path::default_database_path;

use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

/// Opens (or creates) the application database and ensures schema exists.
pub fn open_connection(path: &Path) -> SqlResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                Some(format!("create data directory: {e}")),
            )
        })?;
    }
    let conn = Connection::open(path)?;
    init_schema(&conn)?;
    Ok(conn)
}

/// Opens an in-memory database with schema applied (for tests).
pub fn open_connection_in_memory() -> SqlResult<Connection> {
    let conn = Connection::open_in_memory()?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL,
            title TEXT,
            visited_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_history_visited_at ON history(visited_at DESC);
        CREATE INDEX IF NOT EXISTS idx_history_url ON history(url);
        CREATE TABLE IF NOT EXISTS bookmarks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            url TEXT NOT NULL UNIQUE,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_bookmarks_created_at ON bookmarks(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_bookmarks_title ON bookmarks(title);
        ",
    )
}
