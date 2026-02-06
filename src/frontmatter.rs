use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A single watch entry (pattern -> hash)
#[derive(Debug, Clone)]
pub struct WatchEntry {
    pub pattern: String,
    pub hash: Option<String>,
}

/// Parsed driftwatcher frontmatter
#[derive(Debug)]
pub struct Frontmatter {
    pub entries: Vec<WatchEntry>,
    /// Raw YAML content between --- delimiters (for preservation)
    raw_yaml: String,
    /// Character position where frontmatter ends (after closing ---)
    end_pos: usize,
}

/// Internal struct for serde parsing
#[derive(Debug, Deserialize, Serialize)]
struct YamlFrontmatter {
    #[serde(default)]
    driftwatcher: Option<Vec<HashMap<String, Option<String>>>>,

    #[serde(flatten)]
    other: HashMap<String, serde_yaml::Value>,
}

impl Frontmatter {
    /// Check if driftwatcher is already configured
    pub fn has_driftwatcher(&self) -> bool {
        !self.entries.is_empty() || self.raw_yaml.contains("driftwatcher:")
    }
}

/// Parse frontmatter from file content
pub fn parse(content: &str) -> Result<Option<Frontmatter>> {
    // Check for frontmatter delimiters
    if !content.starts_with("---") {
        return Ok(None);
    }

    // Find the closing ---
    let rest = &content[3..];
    let close_pos = rest
        .find("\n---")
        .ok_or_else(|| anyhow!("Frontmatter not closed (missing closing ---)"))?;

    let yaml_content = &rest[1..close_pos]; // Skip initial newline
    let end_pos = 3 + close_pos + 4; // "---" + content + "\n---"

    // Parse as YAML
    let parsed: YamlFrontmatter =
        serde_yaml::from_str(yaml_content).with_context(|| "Failed to parse YAML frontmatter")?;

    // Extract driftwatcher entries
    let entries = if let Some(dw_entries) = parsed.driftwatcher {
        dw_entries
            .into_iter()
            .filter_map(|map| {
                // Each entry is a single-key map: { "pattern": "hash" } or { "pattern": null }
                map.into_iter()
                    .next()
                    .map(|(pattern, hash)| WatchEntry { pattern, hash })
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(Some(Frontmatter {
        entries,
        raw_yaml: yaml_content.to_string(),
        end_pos,
    }))
}

/// Parse frontmatter from a file path
pub fn parse_file(path: &Path) -> Result<Option<Frontmatter>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    parse(&content)
}

/// Add empty driftwatcher frontmatter to content that has no frontmatter
pub fn add_empty_frontmatter(content: &str) -> String {
    format!("---\ndriftwatcher:\n---\n{}", content)
}

/// Add driftwatcher key to existing frontmatter
pub fn add_driftwatcher_to_existing(content: &str) -> Result<String> {
    let fm = parse(content)?.ok_or_else(|| anyhow!("No frontmatter found"))?;

    // Insert driftwatcher: before the closing ---
    let before_close = &content[..content[..fm.end_pos].rfind("\n---").unwrap()];
    let after_close = &content[fm.end_pos..];

    Ok(format!(
        "{}\ndriftwatcher:\n---{}",
        before_close, after_close
    ))
}

/// Add a watch entry to the frontmatter
pub fn add_entry(content: &str, pattern: &str, hash: &str) -> Result<String> {
    let fm = parse(content)?.ok_or_else(|| anyhow!("No frontmatter found"))?;

    if !fm.has_driftwatcher() {
        // Add driftwatcher section first
        let with_dw = add_driftwatcher_to_existing(content)?;
        return add_entry(&with_dw, pattern, hash);
    }

    // Find where to insert the new entry (after "driftwatcher:" line)
    let dw_pos = content
        .find("driftwatcher:")
        .ok_or_else(|| anyhow!("driftwatcher key not found"))?;

    // Find the end of that line
    let line_end = content[dw_pos..]
        .find('\n')
        .map(|p| dw_pos + p)
        .unwrap_or(content.len());

    let before = &content[..=line_end];
    let after = &content[line_end + 1..];

    // Format the new entry
    let entry_line = format!("  - \"{}\": {}\n", pattern, hash);

    Ok(format!("{}{}{}", before, entry_line, after))
}

/// Update a hash for an existing entry
pub fn update_entry(content: &str, pattern: &str, new_hash: &str) -> Result<String> {
    // Find the entry line and replace the hash
    // This is a bit tricky because we need to handle different formats

    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut found = false;

    for line in lines {
        if !found && line.trim().starts_with("- ") && line.contains(pattern) {
            // This might be our entry - parse it to be sure
            let trimmed = line.trim().strip_prefix("- ").unwrap_or(line);

            // Handle both quoted and unquoted patterns
            let is_match = trimmed.starts_with(&format!("\"{}\":", pattern))
                || trimmed.starts_with(&format!("'{}':", pattern))
                || trimmed.starts_with(&format!("{}:", pattern));

            if is_match {
                // Determine the indentation
                let indent = line.len() - line.trim_start().len();
                let indent_str = &line[..indent];

                // Reconstruct with new hash
                result.push(format!("{}- \"{}\": {}", indent_str, pattern, new_hash));
                found = true;
                continue;
            }
        }
        result.push(line.to_string());
    }

    if !found {
        return Err(anyhow!("Entry not found: {}", pattern));
    }

    Ok(result.join("\n") + "\n")
}

/// Write updated content to a file
pub fn write_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).with_context(|| format!("Failed to write file: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_frontmatter() {
        let content = "---\ndriftwatcher:\n---\n# Doc";
        let fm = parse(content).unwrap().unwrap();
        assert!(fm.entries.is_empty());
        assert!(fm.has_driftwatcher());
    }

    #[test]
    fn test_parse_with_entries() {
        let content = r#"---
driftwatcher:
  - "src/main.rs": abc123def456
  - "lib/**/*.rs": 789xyz
---
# Doc"#;
        let fm = parse(content).unwrap().unwrap();
        assert_eq!(fm.entries.len(), 2);
        assert_eq!(fm.entries[0].pattern, "src/main.rs");
        assert_eq!(fm.entries[0].hash, Some("abc123def456".to_string()));
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# Just a doc\nNo frontmatter here.";
        let fm = parse(content).unwrap();
        assert!(fm.is_none());
    }

    #[test]
    fn test_add_empty_frontmatter() {
        let content = "# My Doc\nSome content.";
        let result = add_empty_frontmatter(content);
        assert!(result.starts_with("---\ndriftwatcher:\n---\n"));
        assert!(result.contains("# My Doc"));
    }

    #[test]
    fn test_parse_with_other_frontmatter() {
        let content = r#"---
title: My Doc
author: Someone
driftwatcher:
  - "src/main.rs": abc123
---
# Content"#;
        let fm = parse(content).unwrap().unwrap();
        assert_eq!(fm.entries.len(), 1);
    }
}
