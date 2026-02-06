use anyhow::Result;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process;

use crate::frontmatter::{self, WatchEntry};
use crate::paths::PathResolver;
use crate::scanner;
use crate::status::Status;

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Plaintext,
    Json,
    Yaml,
}

#[derive(Debug)]
struct DocumentReport {
    doc_path: PathBuf,
    results: Vec<(String, Status)>,
}

pub fn run(format: OutputFormat) -> Result<()> {
    let docs = scanner::find_markdown_files(None)?;
    let mut reports = Vec::new();
    let mut has_problems = false;

    for doc_path in docs {
        let fm = match frontmatter::parse_file(&doc_path) {
            Ok(Some(fm)) if fm.has_driftwatcher() => fm,
            Ok(_) => continue,
            Err(e) => {
                eprintln!("Warning: {}: {}", doc_path.display(), e);
                continue;
            }
        };

        let resolver = match PathResolver::new(&doc_path) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Warning: {}: {}", doc_path.display(), e);
                continue;
            }
        };

        let mut results = Vec::new();

        for entry in &fm.entries {
            let status = check_entry(&resolver, entry);
            if status.is_problem() {
                has_problems = true;
            }
            results.push((entry.pattern.clone(), status));
        }

        if !results.is_empty() {
            reports.push(DocumentReport { doc_path, results });
        }
    }

    match format {
        OutputFormat::Plaintext => print_plaintext(&reports),
        OutputFormat::Json => print_json(&reports),
        OutputFormat::Yaml => print_yaml(&reports),
    }

    if has_problems {
        process::exit(1);
    }

    Ok(())
}

fn check_entry(resolver: &PathResolver, entry: &WatchEntry) -> Status {
    // Check if entry has a hash
    let stored_hash = match &entry.hash {
        Some(h) => h,
        None => return Status::Invalid,
    };

    // Check if files exist
    let paths = match resolver.resolve(&entry.pattern) {
        Ok(p) => p,
        Err(_) => return Status::Missing,
    };

    if paths.is_empty() {
        return Status::Missing;
    }

    // Compute current hash
    let current_hash = match resolver.hash_pattern(&entry.pattern) {
        Ok(h) => h,
        Err(_) => return Status::Missing,
    };

    if current_hash == *stored_hash {
        Status::Current
    } else {
        Status::Drifted
    }
}

fn print_plaintext(reports: &[DocumentReport]) {
    if reports.is_empty() {
        println!("No driftwatcher entries found.");
        return;
    }

    for report in reports {
        println!("{}", report.doc_path.display());
        for (pattern, status) in &report.results {
            println!("  {:8} {}", status, pattern);
        }
        println!();
    }
}

fn print_json(reports: &[DocumentReport]) {
    let map: BTreeMap<String, BTreeMap<String, Status>> = reports
        .iter()
        .map(|r| {
            let inner: BTreeMap<_, _> = r.results.iter().map(|(p, s)| (p.clone(), *s)).collect();
            (r.doc_path.display().to_string(), inner)
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&map).unwrap());
}

fn print_yaml(reports: &[DocumentReport]) {
    let map: BTreeMap<String, BTreeMap<String, String>> = reports
        .iter()
        .map(|r| {
            let inner: BTreeMap<_, _> = r
                .results
                .iter()
                .map(|(p, s)| (p.clone(), s.to_string()))
                .collect();
            (r.doc_path.display().to_string(), inner)
        })
        .collect();

    println!("{}", serde_yaml::to_string(&map).unwrap());
}
