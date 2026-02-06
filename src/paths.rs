use anyhow::{anyhow, Context, Result};
use glob::glob;
use std::path::{Path, PathBuf};

use crate::hash;

/// Handles path resolution relative to a document file
pub struct PathResolver {
    doc_dir: PathBuf,
    project_root: PathBuf,
}

impl PathResolver {
    pub fn new(doc_path: &Path) -> Result<Self> {
        let doc_dir = doc_path
            .parent()
            .map(|p| {
                if p.as_os_str().is_empty() {
                    PathBuf::from(".")
                } else {
                    p.to_path_buf()
                }
            })
            .unwrap_or_else(|| PathBuf::from("."));

        let project_root = find_project_root(&doc_dir)?;

        Ok(Self {
            doc_dir,
            project_root,
        })
    }

    /// Resolve a pattern from frontmatter to actual file paths
    pub fn resolve(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let (base, relative_pattern) = if let Some(stripped) = pattern.strip_prefix("$ROOT/") {
            (&self.project_root, stripped)
        } else {
            (&self.doc_dir, pattern)
        };

        self.resolve_from(base, relative_pattern)
    }

    /// Compute the hash for a pattern (handles files, directories, and globs)
    pub fn hash_pattern(&self, pattern: &str) -> Result<String> {
        let paths = self.resolve(pattern)?;

        if paths.is_empty() {
            return Err(anyhow!("Pattern '{}' matches no files", pattern));
        }

        if paths.len() == 1 {
            let path = &paths[0];
            if path.is_dir() {
                hash::hash_directory(path)
            } else {
                hash::hash_file(path)
            }
        } else {
            // Multiple files from glob - filter out directories
            let files: Vec<_> = paths.into_iter().filter(|p| p.is_file()).collect();
            if files.is_empty() {
                return Err(anyhow!("Pattern '{}' matches no files", pattern));
            }
            hash::hash_files(&files)
        }
    }

    fn resolve_from(&self, base: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
        let full_pattern = base.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();

        if is_glob_pattern(pattern) {
            // Use glob for pattern matching
            let mut paths = Vec::new();
            for entry in glob(&pattern_str)
                .with_context(|| format!("Invalid glob pattern: {}", pattern))?
            {
                match entry {
                    Ok(path) => {
                        // Skip hidden files
                        if !is_hidden(&path) {
                            paths.push(path);
                        }
                    }
                    Err(e) => {
                        // Log but continue on glob errors
                        eprintln!("Warning: glob error: {}", e);
                    }
                }
            }
            Ok(paths)
        } else {
            // Literal path
            let path = full_pattern;
            if path.exists() {
                Ok(vec![path])
            } else {
                Ok(vec![]) // Return empty, caller decides if this is an error
            }
        }
    }
}

/// Find project root by walking up to find .git directory
fn find_project_root(start: &Path) -> Result<PathBuf> {
    let start = if start.is_absolute() {
        start.to_path_buf()
    } else {
        std::env::current_dir()?.join(start)
    };

    let mut current = start.as_path();

    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() {
            return Ok(current.to_path_buf());
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => {
                return Err(anyhow!(
                    "Could not find project root (.git directory) starting from {}",
                    start.display()
                ))
            }
        }
    }
}

/// Check if a pattern contains glob characters
fn is_glob_pattern(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[')
}

/// Check if a path component is hidden (starts with . but not ..)
fn is_hidden(path: &Path) -> bool {
    path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        s.starts_with('.') && s != "." && s != ".."
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_glob_pattern() {
        assert!(is_glob_pattern("*.rs"));
        assert!(is_glob_pattern("src/**/*.rs"));
        assert!(is_glob_pattern("file?.txt"));
        assert!(is_glob_pattern("[abc].txt"));
        assert!(!is_glob_pattern("src/main.rs"));
        assert!(!is_glob_pattern("path/to/file.txt"));
    }

    #[test]
    fn test_is_hidden() {
        assert!(is_hidden(Path::new(".git")));
        assert!(is_hidden(Path::new("src/.hidden")));
        assert!(is_hidden(Path::new(".config/file.txt")));
        assert!(!is_hidden(Path::new("src/main.rs")));
        assert!(!is_hidden(Path::new("visible.txt")));
        // Ensure .. and . are not considered hidden
        assert!(!is_hidden(Path::new("../src/main.rs")));
        assert!(!is_hidden(Path::new("./src/main.rs")));
        assert!(!is_hidden(Path::new("foo/../bar/file.rs")));
    }
}
