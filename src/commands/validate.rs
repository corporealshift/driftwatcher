use anyhow::Result;
use std::process;

use crate::frontmatter;
use crate::paths::PathResolver;
use crate::scanner;

pub fn run() -> Result<()> {
    let docs = scanner::find_markdown_files(None)?;
    let mut all_valid = true;
    let mut checked_count = 0;

    for doc_path in docs {
        // Try to parse frontmatter
        let fm = match frontmatter::parse_file(&doc_path) {
            Ok(Some(fm)) => fm,
            Ok(None) => continue, // No frontmatter, skip
            Err(e) => {
                eprintln!("{}: Invalid YAML - {}", doc_path.display(), e);
                all_valid = false;
                continue;
            }
        };

        if !fm.has_driftwatcher() {
            continue; // No driftwatcher section, skip
        }

        checked_count += 1;
        let resolver = match PathResolver::new(&doc_path) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}: {}", doc_path.display(), e);
                all_valid = false;
                continue;
            }
        };

        for entry in &fm.entries {
            // Check has hash (INVALID status check)
            if entry.hash.is_none() {
                eprintln!(
                    "{}: Entry '{}' has no hash",
                    doc_path.display(),
                    entry.pattern
                );
                all_valid = false;
            }

            // Check paths exist / pattern matches files
            match resolver.resolve(&entry.pattern) {
                Ok(paths) if paths.is_empty() => {
                    eprintln!(
                        "{}: Pattern '{}' matches no files",
                        doc_path.display(),
                        entry.pattern
                    );
                    all_valid = false;
                }
                Err(e) => {
                    eprintln!(
                        "{}: Pattern '{}' - {}",
                        doc_path.display(),
                        entry.pattern,
                        e
                    );
                    all_valid = false;
                }
                Ok(_) => {} // Valid
            }
        }
    }

    if checked_count == 0 {
        println!("No driftwatcher entries found.");
        return Ok(());
    }

    if all_valid {
        println!(
            "All driftwatcher entries are valid ({} file(s) checked).",
            checked_count
        );
        Ok(())
    } else {
        process::exit(1)
    }
}
