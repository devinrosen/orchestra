use crate::models::track::Track;

/// Substitutes pattern tokens with sanitized track field values.
///
/// Supported tokens:
/// - `{artist}`       — COALESCE(album_artist, artist, "Unknown Artist")
/// - `{album_artist}` — album_artist or "Unknown Artist"
/// - `{album}`        — album or "Unknown Album"
/// - `{title}`        — title or file stem
/// - `{track_number}` — zero-padded 2 digits; empty if None or 0
/// - `{disc_number}`  — disc number string; empty if None or 0
/// - `{year}`         — year string; empty if None
/// - `{genre}`        — genre or empty string
/// - `{ext}`          — lowercase file extension
pub fn apply_pattern(pattern: &str, track: &Track) -> String {
    let artist = track
        .album_artist
        .as_deref()
        .filter(|s| !s.is_empty())
        .or_else(|| track.artist.as_deref().filter(|s| !s.is_empty()))
        .unwrap_or("Unknown Artist");

    let album_artist = track
        .album_artist
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("Unknown Artist");

    let album = track
        .album
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("Unknown Album");

    let title_fallback = std::path::Path::new(&track.file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Title")
        .to_string();
    let title = track
        .title
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(&title_fallback);

    let track_number = match track.track_number {
        Some(n) if n > 0 => format!("{:02}", n),
        _ => String::new(),
    };

    let disc_number = match track.disc_number {
        Some(n) if n > 0 => n.to_string(),
        _ => String::new(),
    };

    let year = match track.year {
        Some(y) => y.to_string(),
        None => String::new(),
    };

    let genre = track.genre.as_deref().unwrap_or("").to_string();

    let ext = std::path::Path::new(&track.file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut result = pattern.to_string();
    result = result.replace("{artist}", &sanitize_path_component(artist));
    result = result.replace("{album_artist}", &sanitize_path_component(album_artist));
    result = result.replace("{album}", &sanitize_path_component(album));
    result = result.replace("{title}", &sanitize_path_component(title));
    result = result.replace("{track_number}", &track_number);
    result = result.replace("{disc_number}", &disc_number);
    result = result.replace("{year}", &year);
    result = result.replace("{genre}", &sanitize_path_component(&genre));
    result = result.replace("{ext}", &ext);

    result
}

/// Strips/replaces filesystem-invalid characters from a single path component value.
/// Replaces `\ / : * ? " < > |` with `_`, trims whitespace, truncates to 200 chars.
/// All-dot components (`.`, `..`, `...`) are replaced with `_` to prevent path traversal.
pub fn sanitize_path_component(s: &str) -> String {
    let sanitized: String = s
        .chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect();

    let trimmed = sanitized.trim();

    // Prevent path traversal: all-dot components (e.g. "..") are replaced with "_"
    if !trimmed.is_empty() && trimmed.chars().all(|c| c == '.') {
        return "_".to_string();
    }

    // Truncate by character count to avoid panicking on multi-byte Unicode boundaries
    if trimmed.chars().count() > 200 {
        trimmed.chars().take(200).collect()
    } else {
        trimmed.to_string()
    }
}

/// Moves a file from `src` to `dst`, creating parent directories as needed.
///
/// Tries an atomic rename first (fast path, same filesystem). Falls back to
/// copy-then-rename for cross-device moves using the safe-write pattern
/// (copy → fsync → rename → remove source). Temp files are cleaned up on failure.
pub fn apply_file_move(src: &std::path::Path, dst: &std::path::Path) -> Result<(), std::io::Error> {
    // Ensure destination directory exists
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Attempt atomic rename (fast path, same filesystem)
    if std::fs::rename(src, dst).is_ok() {
        return Ok(());
    }

    // Copy-then-rename fallback (handles cross-device moves)
    let tmp = dst.with_extension("tmp_organize");
    std::fs::copy(src, &tmp)?;
    {
        let f = std::fs::File::open(&tmp)?;
        f.sync_all()?;
    }
    if let Err(e) = std::fs::rename(&tmp, dst) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    // Best-effort: remove source. If it fails (e.g. permissions), the file is at the
    // destination and the DB will be updated to the new path; the stale source becomes
    // an orphan that the user can clean up manually. Returning Err here would leave
    // dst existing while DB still points to src, which is worse.
    let _ = std::fs::remove_file(src);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::track::Track;

    fn make_track(
        title: Option<&str>,
        artist: Option<&str>,
        album_artist: Option<&str>,
        album: Option<&str>,
        track_number: Option<u32>,
        disc_number: Option<u32>,
        year: Option<i32>,
        genre: Option<&str>,
        file_path: &str,
    ) -> Track {
        Track {
            id: Some(1),
            file_path: file_path.to_string(),
            relative_path: String::new(),
            library_root: String::new(),
            title: title.map(|s| s.to_string()),
            artist: artist.map(|s| s.to_string()),
            album_artist: album_artist.map(|s| s.to_string()),
            album: album.map(|s| s.to_string()),
            track_number,
            disc_number,
            year,
            genre: genre.map(|s| s.to_string()),
            duration_secs: None,
            format: "FLAC".to_string(),
            file_size: 0,
            modified_at: 0,
            hash: None,
            has_album_art: false,
            bitrate: None,
            scanned_at: 0,
        }
    }

    #[test]
    fn test_basic_pattern_substitution() {
        let track = make_track(
            Some("Time"),
            Some("Pink Floyd"),
            Some("Pink Floyd"),
            Some("The Dark Side of the Moon"),
            Some(3),
            Some(1),
            Some(1973),
            Some("Progressive Rock"),
            "/music/Pink Floyd/The Dark Side of the Moon/03 Time.flac",
        );
        let result = apply_pattern("{artist}/{album}/{track_number} - {title}.{ext}", &track);
        assert_eq!(
            result,
            "Pink Floyd/The Dark Side of the Moon/03 - Time.flac"
        );
    }

    #[test]
    fn test_artist_coalesce_prefers_album_artist() {
        let track = make_track(
            Some("Track"),
            Some("Various Artists"),
            Some("AC_DC"),
            Some("Album"),
            Some(1),
            None,
            None,
            None,
            "/music/track.flac",
        );
        let result = apply_pattern("{artist}", &track);
        assert_eq!(result, "AC_DC");
    }

    #[test]
    fn test_artist_coalesce_falls_back_to_artist() {
        let track = make_track(
            Some("Track"),
            Some("Pink Floyd"),
            None,
            Some("Album"),
            Some(1),
            None,
            None,
            None,
            "/music/track.flac",
        );
        let result = apply_pattern("{artist}", &track);
        assert_eq!(result, "Pink Floyd");
    }

    #[test]
    fn test_artist_coalesce_unknown_when_none() {
        let track = make_track(
            Some("Track"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            "/music/track.mp3",
        );
        let result = apply_pattern("{artist}", &track);
        assert_eq!(result, "Unknown Artist");
    }

    #[test]
    fn test_track_number_zero_padded() {
        let track = make_track(
            Some("T"),
            Some("A"),
            None,
            Some("B"),
            Some(5),
            None,
            None,
            None,
            "/music/t.flac",
        );
        let result = apply_pattern("{track_number}", &track);
        assert_eq!(result, "05");
    }

    #[test]
    fn test_track_number_zero_is_empty() {
        let track = make_track(
            Some("T"),
            Some("A"),
            None,
            Some("B"),
            Some(0),
            None,
            None,
            None,
            "/music/t.flac",
        );
        let result = apply_pattern("{track_number}", &track);
        assert_eq!(result, "");
    }

    #[test]
    fn test_track_number_none_is_empty() {
        let track = make_track(
            Some("T"),
            Some("A"),
            None,
            Some("B"),
            None,
            None,
            None,
            None,
            "/music/t.flac",
        );
        let result = apply_pattern("{track_number}", &track);
        assert_eq!(result, "");
    }

    #[test]
    fn test_sanitize_strips_invalid_chars() {
        assert_eq!(sanitize_path_component("AC/DC"), "AC_DC");
        assert_eq!(sanitize_path_component("file:name"), "file_name");
        assert_eq!(sanitize_path_component("a*b?c"), "a_b_c");
        assert_eq!(sanitize_path_component(r#"a"b<c>d|e"#), "a_b_c_d_e");
    }

    #[test]
    fn test_sanitize_trims_whitespace() {
        assert_eq!(sanitize_path_component("  Pink Floyd  "), "Pink Floyd");
    }

    #[test]
    fn test_ext_lowercased() {
        let track = make_track(
            Some("T"),
            Some("A"),
            None,
            Some("B"),
            None,
            None,
            None,
            None,
            "/music/t.FLAC",
        );
        let result = apply_pattern("{ext}", &track);
        assert_eq!(result, "flac");
    }

    #[test]
    fn test_title_fallback_to_file_stem() {
        let track = make_track(
            None,
            Some("A"),
            None,
            Some("B"),
            None,
            None,
            None,
            None,
            "/music/my_track.flac",
        );
        let result = apply_pattern("{title}", &track);
        assert_eq!(result, "my_track");
    }

    #[test]
    fn test_sanitize_truncates_at_char_boundary_not_byte_boundary() {
        // Each '文' is 3 bytes in UTF-8; 201 of them exceed the 200-char limit.
        // Truncation must happen at a char boundary (char 200), not at byte 200 (which
        // would land mid-sequence and panic when slicing the string).
        let long_unicode: String = std::iter::repeat('文').take(201).collect();
        let result = sanitize_path_component(&long_unicode);
        assert_eq!(result.chars().count(), 200);
        assert!(result.is_char_boundary(result.len()));
    }
}
