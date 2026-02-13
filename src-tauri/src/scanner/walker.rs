use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::models::track::is_audio_file;

pub struct WalkResult {
    pub audio_files: Vec<PathBuf>,
    pub root: PathBuf,
}

pub fn walk_directory(root: &Path, exclude_patterns: &[String]) -> WalkResult {
    let compiled_patterns: Vec<glob::Pattern> = exclude_patterns
        .iter()
        .filter_map(|p| glob::Pattern::new(p).ok())
        .collect();

    let audio_files: Vec<PathBuf> = WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_audio_file(e.path()))
        .filter(|e| {
            let rel = e.path().strip_prefix(root).unwrap_or(e.path());
            let rel_str = rel.to_string_lossy();
            !compiled_patterns.iter().any(|p| p.matches(&rel_str))
        })
        .map(|e| e.into_path())
        .collect();

    WalkResult {
        audio_files,
        root: root.to_path_buf(),
    }
}
