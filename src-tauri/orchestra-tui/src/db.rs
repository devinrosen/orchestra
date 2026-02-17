use std::path::PathBuf;

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};

/// Returns the expected path of the Orchestra SQLite database.
///
/// Checks:
/// 1. `$XDG_DATA_HOME/com.orchestra.app/orchestra.db` (Linux)
/// 2. `dirs::data_dir() / "com.orchestra.app" / "orchestra.db"`
///
/// Returns `None` if no matching file exists on disk.
pub fn find_db_path() -> Option<PathBuf> {
    let candidates: Vec<PathBuf> = {
        let mut v = Vec::new();

        // XDG_DATA_HOME override (Linux convention).
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            let p = PathBuf::from(xdg)
                .join("com.orchestra.app")
                .join("orchestra.db");
            v.push(p);
        }

        // Standard platform data dir (macOS: ~/Library/Application Support, Linux: ~/.local/share).
        if let Some(data_dir) = dirs::data_dir() {
            let p = data_dir.join("com.orchestra.app").join("orchestra.db");
            v.push(p);
        }

        v
    };

    candidates.into_iter().find(|path| path.exists())
}

/// Reads the `library_root` setting from the `settings` table.
///
/// Returns `Ok(None)` if the key is absent.
pub fn read_library_root(conn: &Connection) -> Result<Option<String>> {
    let result: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'library_root'",
            [],
            |row| row.get(0),
        )
        .optional()
        .context("Failed to query settings table")?;

    Ok(result)
}
