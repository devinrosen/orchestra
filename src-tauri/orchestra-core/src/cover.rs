use lofty::picture::PictureType;
use lofty::prelude::*;

/// Extract embedded album art from `file_path` and write it to
/// `$TMPDIR/{temp_filename}`. Returns a `file://` URI on success.
///
/// Using a fixed path per caller avoids accumulating orphaned temp files —
/// the file is overwritten on each track change.
pub fn extract_cover(file_path: &str, temp_filename: &str) -> Option<String> {
    let path = std::path::Path::new(file_path);
    let tagged_file = lofty::read_from_path(path).ok()?;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())?;

    let picture = tag
        .pictures()
        .iter()
        .find(|p| p.pic_type() == PictureType::CoverFront)
        .or_else(|| tag.pictures().first())?;

    let temp_path = std::env::temp_dir().join(temp_filename);
    std::fs::write(&temp_path, picture.data()).ok()?;

    Some(format!("file://{}", temp_path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_cover_nonexistent_file_returns_none() {
        assert!(extract_cover("/nonexistent/path/to/file.mp3", "test-art.jpg").is_none());
    }

    #[test]
    fn extract_cover_empty_file_returns_none() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        assert!(extract_cover(tmp.path().to_str().unwrap(), "test-art.jpg").is_none());
    }
}
