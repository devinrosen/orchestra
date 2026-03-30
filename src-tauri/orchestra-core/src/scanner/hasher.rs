use std::path::Path;

use crate::error::AppError;

pub fn hash_file(path: &Path) -> Result<String, AppError> {
    let mut hasher = blake3::Hasher::new();
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::with_capacity(1024 * 1024, file);
    std::io::copy(&mut reader, &mut hasher)?;
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::Builder;

    #[test]
    fn test_hash_produces_64_char_hex() {
        let mut file = Builder::new().suffix(".flac").tempfile().unwrap();
        file.write_all(b"hello world").unwrap();
        file.flush().unwrap();

        let hash = hash_file(file.path()).unwrap();
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_same_content_same_hash() {
        let content = b"identical content for both files";

        let mut file1 = Builder::new().suffix(".flac").tempfile().unwrap();
        file1.write_all(content).unwrap();
        file1.flush().unwrap();

        let mut file2 = Builder::new().suffix(".flac").tempfile().unwrap();
        file2.write_all(content).unwrap();
        file2.flush().unwrap();

        let hash1 = hash_file(file1.path()).unwrap();
        let hash2 = hash_file(file2.path()).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_content_different_hash() {
        let mut file1 = Builder::new().suffix(".flac").tempfile().unwrap();
        file1.write_all(b"content A").unwrap();
        file1.flush().unwrap();

        let mut file2 = Builder::new().suffix(".flac").tempfile().unwrap();
        file2.write_all(b"content B").unwrap();
        file2.flush().unwrap();

        let hash1 = hash_file(file1.path()).unwrap();
        let hash2 = hash_file(file2.path()).unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_nonexistent_file_returns_error() {
        let path = PathBuf::from("/nonexistent/path/file.flac");
        let result = hash_file(&path);
        assert!(result.is_err());
    }
}
