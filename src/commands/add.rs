use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::frontmatter;
use crate::paths::PathResolver;

pub fn run(doc_file: &Path, watch_pattern: &str) -> Result<()> {
    // Check doc file exists
    if !doc_file.exists() {
        return Err(anyhow!("Invalid file: {}", doc_file.display()));
    }

    // Read content
    let content = fs::read_to_string(doc_file)?;

    // Check frontmatter exists and has driftwatcher
    let fm = frontmatter::parse(&content)?;
    if fm.is_none() || !fm.as_ref().unwrap().has_driftwatcher() {
        return Err(anyhow!(
            "File not initialized. Run 'drifty init {}' first.",
            doc_file.display()
        ));
    }

    // Check if pattern already exists
    let fm = fm.unwrap();
    if fm.entries.iter().any(|e| e.pattern == watch_pattern) {
        return Err(anyhow!(
            "Pattern '{}' already exists in {}",
            watch_pattern,
            doc_file.display()
        ));
    }

    // Resolve the pattern and compute hash
    let resolver = PathResolver::new(doc_file)?;
    let paths = resolver.resolve(watch_pattern)?;

    if paths.is_empty() {
        return Err(anyhow!(
            "Pattern '{}' matches no files",
            watch_pattern
        ));
    }

    let hash = resolver.hash_pattern(watch_pattern)?;

    // Add entry to frontmatter
    let new_content = frontmatter::add_entry(&content, watch_pattern, &hash)?;
    frontmatter::write_file(doc_file, &new_content)?;

    println!(
        "Added '{}' to {} ({} file(s), hash: {}...)",
        watch_pattern,
        doc_file.display(),
        paths.len(),
        &hash[..12]
    );

    Ok(())
}
