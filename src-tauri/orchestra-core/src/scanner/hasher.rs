use std::path::Path;

use crate::error::AppError;

pub fn hash_file(path: &Path) -> Result<String, AppError> {
    let mut hasher = blake3::Hasher::new();
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::with_capacity(1024 * 1024, file);
    std::io::copy(&mut reader, &mut hasher)?;
    Ok(hasher.finalize().to_hex().to_string())
}
