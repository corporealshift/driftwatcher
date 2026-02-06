use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::fs;
use std::path::{Path, PathBuf};

use crate::frontmatter::{self, WatchEntry};
use crate::paths::PathResolver;
use crate::scanner;
use crate::status::Status;

#[derive(Debug)]
struct DriftedEntry {
    doc_path: PathBuf,
    pattern: String,
    current_hash: String,
}

pub fn run(target: Option<&Path>) -> Result<()> {
    let docs = scanner::find_markdown_files(target)?;
    let mut drifted: Vec<DriftedEntry> = Vec::new();
    let mut broken_files: Vec<(PathBuf, String)> = Vec::new();
    let mut current_count = 0;
    let mut missing_count = 0;

    for doc_path in docs {
        let content = match fs::read_to_string(&doc_path) {
            Ok(c) => c,
            Err(e) => {
                broken_files.push((doc_path, e.to_string()));
                continue;
            }
        };

        let fm = match frontmatter::parse(&content) {
            Ok(Some(fm)) if fm.has_driftwatcher() => fm,
            Ok(_) => continue,
            Err(e) => {
                broken_files.push((doc_path, e.to_string()));
                continue;
            }
        };

        let resolver = match PathResolver::new(&doc_path) {
            Ok(r) => r,
            Err(e) => {
                broken_files.push((doc_path, e.to_string()));
                continue;
            }
        };

        for entry in &fm.entries {
            match check_entry(&resolver, entry) {
                (Status::Current, _) => current_count += 1,
                (Status::Missing, _) => {
                    missing_count += 1;
                    eprintln!(
                        "MISSING: {} -> {}",
                        doc_path.display(),
                        entry.pattern
                    );
                }
                (Status::Invalid, _) => {
                    eprintln!(
                        "INVALID: {} -> {} (no hash)",
                        doc_path.display(),
                        entry.pattern
                    );
                }
                (Status::Drifted, Some(current_hash)) => {
                    drifted.push(DriftedEntry {
                        doc_path: doc_path.clone(),
                        pattern: entry.pattern.clone(),
                        current_hash,
                    });
                }
                (Status::Drifted, None) => {
                    // Shouldn't happen, but handle gracefully
                    missing_count += 1;
                }
            }
        }
    }

    // Report summary
    println!(
        "\nFound {} current, {} drifted, {} missing",
        current_count,
        drifted.len(),
        missing_count
    );

    if drifted.is_empty() {
        if current_count > 0 {
            println!("All documentation is up-to-date!");
        }
    } else {
        // Present TUI for selection
        let items: Vec<String> = drifted
            .iter()
            .map(|d| format!("{}: {}", d.doc_path.display(), d.pattern))
            .collect();

        println!();
        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select entries to update (space to toggle, enter to confirm, 'a' for all)")
            .items(&items)
            .interact()?;

        if selections.is_empty() {
            println!("No entries selected.");
        } else {
            // Group updates by document
            let mut updates: std::collections::HashMap<PathBuf, Vec<(&str, &str)>> =
                std::collections::HashMap::new();

            for idx in &selections {
                let entry = &drifted[*idx];
                updates
                    .entry(entry.doc_path.clone())
                    .or_default()
                    .push((&entry.pattern, &entry.current_hash));
            }

            // Apply updates
            for (doc_path, entries) in updates {
                let mut content = fs::read_to_string(&doc_path)?;

                for (pattern, new_hash) in entries {
                    content = frontmatter::update_entry(&content, pattern, new_hash)?;
                }

                frontmatter::write_file(&doc_path, &content)?;
            }

            println!("Updated {} entries.", selections.len());
        }
    }

    // Report broken files at end
    if !broken_files.is_empty() {
        eprintln!("\nWarning: The following files had errors:");
        for (path, err) in broken_files {
            eprintln!("  {}: {}", path.display(), err);
        }
    }

    Ok(())
}

fn check_entry(resolver: &PathResolver, entry: &WatchEntry) -> (Status, Option<String>) {
    // Check if entry has a hash
    let stored_hash = match &entry.hash {
        Some(h) => h,
        None => return (Status::Invalid, None),
    };

    // Check if files exist
    let paths = match resolver.resolve(&entry.pattern) {
        Ok(p) => p,
        Err(_) => return (Status::Missing, None),
    };

    if paths.is_empty() {
        return (Status::Missing, None);
    }

    // Compute current hash
    let current_hash = match resolver.hash_pattern(&entry.pattern) {
        Ok(h) => h,
        Err(_) => return (Status::Missing, None),
    };

    if current_hash == *stored_hash {
        (Status::Current, Some(current_hash))
    } else {
        (Status::Drifted, Some(current_hash))
    }
}
