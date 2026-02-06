---
driftwatcher:
  - "../src/commands/*.rs": def456
  - "../src/frontmatter.rs": ghi789
---

# Command Architecture

This document covers how commands are structured in driftwatcher.

## Commands Module

All commands live in `src/commands/` and follow a consistent pattern:

- `init.rs` - Initialize frontmatter in a doc file
- `add.rs` - Add a watch entry
- `check.rs` - Interactive drift checking with TUI
- `report.rs` - Generate status reports
- `validate.rs` - Validate frontmatter entries

Each command exports a `run()` function that takes the appropriate arguments
and returns `anyhow::Result<()>`.

## Frontmatter Handling

The `frontmatter.rs` module provides parsing and writing utilities for
YAML frontmatter in markdown files.
