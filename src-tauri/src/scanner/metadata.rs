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

    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());
    let properties = tagged_file.properties();

    let (title, artist, album_artist, album, track_number, disc_number, year, genre) =
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
            )
        } else {
            (None, None, None, None, None, None, None, None)
        };

    let duration_secs = properties.duration().as_secs_f64();

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
    })
}
