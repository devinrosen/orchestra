use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::models::track::is_audio_file;

pub fn walk_directory_iter(
    root: &Path,
    exclude_patterns: &[String],
) -> impl Iterator<Item = PathBuf> {
    let compiled_patterns: Vec<glob::Pattern> = exclude_patterns
        .iter()
        .filter_map(|p| glob::Pattern::new(p).ok())
        .collect();

    let root_owned = root.to_path_buf();
    WalkDir::new(root_owned.clone())
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_audio_file(e.path()))
        .filter(move |e| {
            let rel = e.path().strip_prefix(&root_owned).unwrap_or(e.path());
            let rel_str = rel.to_string_lossy();
            !compiled_patterns.iter().any(|p| p.matches(&rel_str))
        })
        .map(|e| e.into_path())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_file(dir: &TempDir, rel_path: &str) {
        let path = dir.path().join(rel_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, b"audio data").unwrap();
    }

    #[test]
    fn test_walk_finds_audio_files() {
        let dir = TempDir::new().unwrap();
        create_file(&dir, "artist/album/track01.flac");
        create_file(&dir, "artist/album/track02.mp3");
        create_file(&dir, "single.flac");

        let results: Vec<_> = walk_directory_iter(dir.path(), &[]).collect();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_walk_excludes_non_audio() {
        let dir = TempDir::new().unwrap();
        create_file(&dir, "track.flac");
        create_file(&dir, "readme.txt");
        create_file(&dir, "cover.jpg");
        create_file(&dir, "artist/track.mp3");

        let results: Vec<_> = walk_directory_iter(dir.path(), &[]).collect();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|p| {
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            ext == "flac" || ext == "mp3"
        }));
    }

    #[test]
    fn test_walk_respects_exclude_patterns() {
        let dir = TempDir::new().unwrap();
        create_file(&dir, "keep/track.flac");
        create_file(&dir, "ignore/track.flac");
        create_file(&dir, "ignore/nested/track.mp3");

        let results: Vec<_> = walk_directory_iter(dir.path(), &["ignore/**".to_string()]).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].to_string_lossy().contains("keep"));
    }

    #[test]
    fn test_walk_empty_directory() {
        let dir = TempDir::new().unwrap();
        let results: Vec<_> = walk_directory_iter(dir.path(), &[]).collect();
        assert!(results.is_empty());
    }

    #[test]
    fn test_walk_does_not_return_directories() {
        let dir = TempDir::new().unwrap();
        create_file(&dir, "nested/track.flac");

        let results: Vec<_> = walk_directory_iter(dir.path(), &[]).collect();
        for path in &results {
            assert!(path.is_file());
        }
    }
}
