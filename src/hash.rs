use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Hash a single file's contents
pub fn hash_file(path: &Path) -> Result<String> {
    let contents =
        fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Hash multiple files together (for glob patterns)
/// Files are sorted alphabetically for deterministic output
pub fn hash_files(paths: &[PathBuf]) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut sorted_paths = paths.to_vec();
    sorted_paths.sort();

    for path in &sorted_paths {
        // Include relative path in hash for structure sensitivity
        hasher.update(path.to_string_lossy().as_bytes());
        hasher.update(b"\n");
        let contents =
            fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))?;
        hasher.update(&contents);
        hasher.update(b"\n");
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Hash a directory recursively (excluding hidden files)
pub fn hash_directory(dir: &Path) -> Result<String> {
    let files = collect_files_recursive(dir)?;
    if files.is_empty() {
        // Empty directory - hash the path itself
        let mut hasher = Sha256::new();
        hasher.update(dir.to_string_lossy().as_bytes());
        let result = hasher.finalize();
        return Ok(format!("{:x}", result));
    }
    hash_files(&files)
}

/// Collect all files in a directory recursively, excluding hidden files
pub fn collect_files_recursive(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_recursive_inner(dir, &mut files)?;
    Ok(files)
}

fn collect_files_recursive_inner(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();

        // Skip hidden files/directories
        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            collect_files_recursive_inner(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hash_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, b"hello world").unwrap();

        let hash = hash_file(&file_path).unwrap();
        assert_eq!(hash.len(), 64); // SHA256 is 64 hex chars
    }

    #[test]
    fn test_hash_deterministic() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, b"hello world").unwrap();

        let hash1 = hash_file(&file_path).unwrap();
        let hash2 = hash_file(&file_path).unwrap();
        assert_eq!(hash1, hash2);
    }
}
