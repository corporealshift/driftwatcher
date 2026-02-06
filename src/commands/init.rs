use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::frontmatter;

pub fn run(doc_file: &Path) -> Result<()> {
    // Check file exists
    if !doc_file.exists() {
        return Err(anyhow!("Invalid file: {}", doc_file.display()));
    }

    // Read content
    let content = fs::read_to_string(doc_file)?;

    // Check current state
    match frontmatter::parse(&content)? {
        Some(fm) if fm.has_driftwatcher() => {
            println!("driftwatcher already initialized in {}", doc_file.display());
            Ok(())
        }
        Some(_) => {
            // Has frontmatter but no driftwatcher key - add it
            let new_content = frontmatter::add_driftwatcher_to_existing(&content)?;
            frontmatter::write_file(doc_file, &new_content)?;
            println!("Added driftwatcher to existing frontmatter in {}", doc_file.display());
            Ok(())
        }
        None => {
            // No frontmatter - add complete block
            let new_content = frontmatter::add_empty_frontmatter(&content);
            frontmatter::write_file(doc_file, &new_content)?;
            println!("Initialized driftwatcher in {}", doc_file.display());
            Ok(())
        }
    }
}
