use std::sync::Mutex;
use std::time::Duration;

use rusqlite::{params, Connection};

use orchestra_core::cover;
use orchestra_core::error::AppError;

use crate::media_session::MediaSessionState;

#[tauri::command]
pub async fn update_now_playing(
    db: tauri::State<'_, Mutex<Connection>>,
    session: tauri::State<'_, Mutex<MediaSessionState>>,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    duration_secs: Option<f64>,
    file_path: String,
) -> Result<(), AppError> {
    // Validate file_path against the library root to prevent path traversal.
    let library_root: Option<String> = {
        let conn = db
            .lock()
            .map_err(|e| AppError::General(format!("update_now_playing: db lock poisoned: {e}")))?;
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'library_root'")?;
        stmt.query_row(params![], |row| row.get(0)).ok()
    };

    let root = library_root.ok_or_else(|| {
        AppError::General("update_now_playing: no library root configured; cannot validate file path".to_string())
    })?;
    let file_path_for_check = file_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        check_path_in_root(&root, &file_path_for_check)
    })
    .await
    .map_err(|e| {
        AppError::General(format!("update_now_playing: path check task failed: {e}"))
    })??;

    let file_path_clone = file_path.clone();
    let cover_url = tauri::async_runtime::spawn_blocking(move || {
        cover::extract_cover(&file_path_clone, "orchestra-art.jpg")
    })
    .await
    .unwrap_or(None);

    let s = session.lock().map_err(|e| {
        AppError::General(format!(
            "update_now_playing: media session lock poisoned: {e}"
        ))
    })?;
    s.update_metadata(title, artist, album, duration_secs, cover_url);
    Ok(())
}

/// Returns `Ok` if `file_path` canonicalizes to a path inside `library_root`.
/// Returns `Err` if the path cannot be resolved or falls outside the root.
pub(crate) fn check_path_in_root(library_root: &str, file_path: &str) -> Result<(), AppError> {
    let root_canonical = std::fs::canonicalize(library_root).map_err(|e| {
        AppError::General(format!(
            "path check: cannot canonicalize library root '{}': {e}",
            library_root
        ))
    })?;
    let file_canonical = std::fs::canonicalize(file_path).map_err(|e| {
        AppError::General(format!(
            "path check: cannot canonicalize '{}': {e}",
            file_path
        ))
    })?;
    if !file_canonical.starts_with(&root_canonical) {
        return Err(AppError::General(format!(
            "path check: file path '{}' is outside library root '{}'",
            file_canonical.display(),
            root_canonical.display()
        )));
    }
    Ok(())
}

#[tauri::command]
pub async fn update_playback_state(
    session: tauri::State<'_, Mutex<MediaSessionState>>,
    playing: bool,
    position_secs: f64,
) -> Result<(), AppError> {
    if !position_secs.is_finite() || position_secs < 0.0 {
        return Err(AppError::General(
            "update_playback_state: position_secs must be a finite non-negative value".to_string(),
        ));
    }
    let s = session.lock().map_err(|e| {
        AppError::General(format!(
            "update_playback_state: media session lock poisoned: {e}"
        ))
    })?;
    s.update_playback(playing, Duration::from_secs_f64(position_secs));
    Ok(())
}

#[cfg(test)]
mod path_traversal_tests {
    use super::*;
    use tempfile::TempDir;

    /// Create a real file inside a temp dir and return (dir, file_path_string).
    fn make_file(dir: &TempDir, name: &str) -> String {
        let path = dir.path().join(name);
        std::fs::write(&path, b"").unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn no_library_root_returns_error() {
        let library_root: Option<String> = None;
        let err = library_root
            .ok_or_else(|| {
                AppError::General(
                    "update_now_playing: no library root configured; cannot validate file path"
                        .to_string(),
                )
            })
            .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("no library root configured"),
            "error should mention missing library root: {msg}"
        );
    }

    #[test]
    fn nonexistent_library_root_returns_diagnostic_error() {
        let file = "/tmp/some_track.flac";
        let err = check_path_in_root("/nonexistent/library/root", file).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("cannot canonicalize library root"),
            "error should mention library root canonicalization: {msg}"
        );
        assert!(
            msg.contains("/nonexistent/library/root"),
            "error should include the library root path: {msg}"
        );
    }

    #[test]
    fn valid_path_inside_root_passes() {
        let tmp = TempDir::new().unwrap();
        let file = make_file(&tmp, "track.flac");
        assert!(check_path_in_root(tmp.path().to_str().unwrap(), &file).is_ok());
    }

    #[test]
    fn path_outside_root_is_rejected() {
        let root = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let file = make_file(&outside, "evil.flac");
        let err = check_path_in_root(root.path().to_str().unwrap(), &file).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("outside library root"),
            "unexpected error: {err}"
        );
        assert!(
            msg.contains("evil.flac"),
            "error should include the rejected file path: {msg}"
        );
    }

    #[test]
    fn nonexistent_file_returns_diagnostic_error() {
        let tmp = TempDir::new().unwrap();
        let missing = tmp
            .path()
            .join("missing.flac")
            .to_string_lossy()
            .into_owned();
        let err = check_path_in_root(tmp.path().to_str().unwrap(), &missing).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("cannot canonicalize"),
            "error should mention canonicalize: {msg}"
        );
        assert!(
            msg.contains("missing.flac"),
            "error should include the file path: {msg}"
        );
    }

    #[test]
    fn symlink_traversal_is_rejected() {
        let root = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let real_file = make_file(&outside, "secret.flac");
        // Create a symlink inside root pointing to the outside file.
        let link = root.path().join("link.flac");
        std::os::unix::fs::symlink(&real_file, &link).unwrap();
        let err =
            check_path_in_root(root.path().to_str().unwrap(), link.to_str().unwrap()).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("outside library root"),
            "symlink traversal should be rejected: {err}"
        );
        assert!(
            msg.contains("secret.flac"),
            "error should include the resolved symlink target: {msg}"
        );
    }
}
