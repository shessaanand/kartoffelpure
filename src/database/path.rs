//! Default database file location.

use std::path::PathBuf;

const APP_DIR: &str = ".local/share/kartoffelpure";
const DB_FILE: &str = "kartoffelpure.db";

/// Returns `~/.local/share/kartoffelpure/kartoffelpure.db` on Linux.
pub fn default_database_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("."));
    PathBuf::from(home).join(APP_DIR).join(DB_FILE)
}
