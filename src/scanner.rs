use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Find all markdown files in a target path
pub fn find_markdown_files(target: Option<&Path>) -> Result<Vec<PathBuf>> {
    let start = target.unwrap_or(Path::new("."));

    if start.is_file() {
        if is_markdown(start) {
            return Ok(vec![start.to_path_buf()]);
        } else {
            return Err(anyhow!(
                "File is not a markdown file: {}",
                start.display()
            ));
        }
    }

    if !start.exists() {
        return Err(anyhow!("Path does not exist: {}", start.display()));
    }

    let mut files = Vec::new();
    scan_directory(start, &mut files)?;
    files.sort();
    Ok(files)
}

fn scan_directory(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();

        // Skip hidden directories
        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            scan_directory(&path, files)?;
        } else if is_markdown(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .map(|ext| {
            let ext = ext.to_string_lossy().to_lowercase();
            ext == "md" || ext == "markdown"
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_markdown() {
        assert!(is_markdown(Path::new("README.md")));
        assert!(is_markdown(Path::new("doc.markdown")));
        assert!(is_markdown(Path::new("path/to/file.MD")));
        assert!(!is_markdown(Path::new("file.txt")));
        assert!(!is_markdown(Path::new("file.rs")));
        assert!(!is_markdown(Path::new("noext")));
    }
}
