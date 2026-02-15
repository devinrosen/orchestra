use rusqlite::Connection;

use crate::error::AppError;
use crate::models::library_root::LibraryRoot;

/// Insert a new library root. If the path already exists, does nothing (idempotent).
pub fn add_library_root(
    conn: &Connection,
    path: &str,
    label: Option<&str>,
    added_at: i64,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO library_roots (path, label, added_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(path) DO NOTHING",
        rusqlite::params![path, label, added_at],
    )?;
    Ok(())
}

/// Delete a library root and cascade to all tracks belonging to that root.
pub fn remove_library_root(conn: &Connection, path: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM library_roots WHERE path = ?1",
        rusqlite::params![path],
    )?;
    conn.execute(
        "DELETE FROM tracks WHERE library_root = ?1",
        rusqlite::params![path],
    )?;
    Ok(())
}

/// Return all library roots ordered by `added_at` ascending.
pub fn list_library_roots(conn: &Connection) -> Result<Vec<LibraryRoot>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT path, label, added_at FROM library_roots ORDER BY added_at",
    )?;
    let roots = stmt
        .query_map([], |row| {
            Ok(LibraryRoot {
                path: row.get(0)?,
                label: row.get(1)?,
                added_at: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(roots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::run_migrations(&conn).unwrap();
        conn
    }

    fn insert_track(conn: &Connection, library_root: &str, file_path: &str) {
        conn.execute(
            "INSERT INTO tracks (file_path, relative_path, library_root, format, file_size, modified_at)
             VALUES (?1, ?2, ?3, 'flac', 1000, 1700000000)",
            rusqlite::params![file_path, "relative.flac", library_root],
        )
        .unwrap();
    }

    #[test]
    fn test_list_roots_empty() {
        let conn = setup_db();
        let roots = list_library_roots(&conn).unwrap();
        assert!(roots.is_empty());
    }

    #[test]
    fn test_add_and_list_roots() {
        let conn = setup_db();
        add_library_root(&conn, "/music/rock", Some("Rock"), 1000).unwrap();
        add_library_root(&conn, "/music/jazz", None, 2000).unwrap();

        let roots = list_library_roots(&conn).unwrap();
        assert_eq!(roots.len(), 2);
        // Ordered by added_at
        assert_eq!(roots[0].path, "/music/rock");
        assert_eq!(roots[0].label.as_deref(), Some("Rock"));
        assert_eq!(roots[0].added_at, 1000);
        assert_eq!(roots[1].path, "/music/jazz");
        assert!(roots[1].label.is_none());
        assert_eq!(roots[1].added_at, 2000);
    }

    #[test]
    fn test_add_duplicate_root_is_idempotent() {
        let conn = setup_db();
        add_library_root(&conn, "/music", Some("First"), 1000).unwrap();
        // Second insert with the same path is silently ignored
        add_library_root(&conn, "/music", Some("Second"), 2000).unwrap();

        let roots = list_library_roots(&conn).unwrap();
        assert_eq!(roots.len(), 1);
        // The first insert's values are preserved
        assert_eq!(roots[0].label.as_deref(), Some("First"));
        assert_eq!(roots[0].added_at, 1000);
    }

    #[test]
    fn test_remove_root_cascades_tracks() {
        let conn = setup_db();
        add_library_root(&conn, "/music", None, 1000).unwrap();
        insert_track(&conn, "/music", "/music/song.flac");

        // Verify track is present
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tracks WHERE library_root = '/music'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        remove_library_root(&conn, "/music").unwrap();

        // Root is gone
        let roots = list_library_roots(&conn).unwrap();
        assert!(roots.is_empty());

        // Tracks belonging to that root are gone
        let count_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tracks WHERE library_root = '/music'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count_after, 0);
    }

    #[test]
    fn test_remove_nonexistent_root_ok() {
        let conn = setup_db();
        // Should not error even if path does not exist
        let result = remove_library_root(&conn, "/nonexistent");
        assert!(result.is_ok());
    }
}
