use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::models::track::is_audio_file;

pub fn walk_directory_iter(root: &Path, exclude_patterns: &[String]) -> impl Iterator<Item = PathBuf> {
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
