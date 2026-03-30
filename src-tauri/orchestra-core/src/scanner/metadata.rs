use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::Accessor;
use std::path::Path;

use crate::error::AppError;
use crate::models::track::Track;

pub fn extract_metadata(path: &Path, library_root: &Path) -> Result<Track, AppError> {
    let relative_path = path
        .strip_prefix(library_root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    let file_meta = std::fs::metadata(path)?;
    let modified_at = file_meta
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let format = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("unknown")
        .to_lowercase();

    let tagged_file = lofty::read_from_path(path)
        .map_err(|e| AppError::Metadata(format!("{}: {}", path.display(), e)))?;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());
    let properties = tagged_file.properties();

    let (title, artist, album_artist, album, track_number, disc_number, year, genre, has_album_art) =
        if let Some(tag) = tag {
            (
                tag.title().map(|s| s.to_string()),
                tag.artist().map(|s| s.to_string()),
                tag.get_string(&lofty::tag::ItemKey::AlbumArtist)
                    .map(|s| s.to_string()),
                tag.album().map(|s| s.to_string()),
                tag.track(),
                tag.disk(),
                tag.year(),
                tag.genre().map(|s| s.to_string()),
                !tag.pictures().is_empty(),
            )
        } else {
            (None, None, None, None, None, None, None, None, false)
        };

    let duration_secs = properties.duration().as_secs_f64();
    let bitrate = properties.overall_bitrate();

    Ok(Track {
        id: None,
        file_path: path.to_string_lossy().to_string(),
        relative_path,
        library_root: library_root.to_string_lossy().to_string(),
        title,
        artist,
        album_artist,
        album,
        track_number,
        disc_number,
        year: year.map(|y| y as i32),
        genre,
        duration_secs: Some(duration_secs),
        format,
        file_size: file_meta.len(),
        modified_at,
        hash: None,
        has_album_art,
        bitrate,
        scanned_at: 0, // set by upsert_track to the current timestamp
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::Builder;

    #[test]
    fn test_extract_nonexistent_file_returns_error() {
        let path = std::path::PathBuf::from("/nonexistent/path/track.flac");
        let library_root = std::path::PathBuf::from("/nonexistent");
        let result = extract_metadata(&path, &library_root);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_non_audio_file_returns_error() {
        let mut file = Builder::new().suffix(".txt").tempfile().unwrap();
        file.write_all(b"this is not audio data").unwrap();
        file.flush().unwrap();

        let parent = file.path().parent().unwrap().to_path_buf();
        let result = extract_metadata(file.path(), &parent);
        assert!(result.is_err());
    }
}
