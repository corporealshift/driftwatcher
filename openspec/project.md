# Project Context

## Purpose

This project is a simple command line tool to help developers keep documentation from getting out of sync with the code it is documenting.

## How it works

DriftWatcher scans markdown files for YAML frontmatter containing a `driftwatcher:` key. This frontmatter maps file paths (or glob patterns) to SHA256 hashes. When `drifty check` runs, it recomputes hashes and compares them to the stored values, reporting any drift.

## Usage

The tool will provide the command `drifty` once installed. Here are the commands that are available:

- `drifty init <doc-file>`: Initializes the doc file with an empty drifty table.
- `drifty add <doc-file> <file-to-watch>`: Adds a file to watch to the doc file's frontmatter and computes its initial hash.
- `drifty check`: Checks all documentation in the current directory (recursively) for drift. Provides an interactive TUI for updating hashes.
- `drifty check <filename>`: Checks the specific file or directory.
- `drifty report --format json|yaml`: Reports status of all tracked files. Useful for CI.
- `drifty validate`: Verifies that all drifty YAML front matter is valid, including file paths.

## Command Behaviors

### `drifty init <doc-file>`
- If file doesn't exist: report "invalid file" error
- If file exists without frontmatter: add empty driftwatcher frontmatter to the top
- If file exists with non-driftwatcher frontmatter: add driftwatcher key to existing frontmatter
- If file already has driftwatcher frontmatter: report "drifty already in file"

### `drifty check`
- Interactive TUI with arrow keys and checkboxes for selecting which files to update
- Continue processing even if some markdown files have broken frontmatter
- Report broken files at the end, but don't stop the command

### `drifty report`
- Exit with non-zero status if any files are DRIFTED or MISSING (for CI integration)
- Default format is plaintext; supports `--format json` and `--format yaml`

### `drifty validate`
- Check that frontmatter is parseable YAML
- Check that all file paths exist
- For glob patterns: check that at least one file matches

## Path Resolution

Paths in frontmatter can be:
- Relative to the markdown file's location (e.g., `../src/lib.rs`)
- Prefixed with `$ROOT/` to be relative to project root (e.g., `$ROOT/src/lib.rs`)

Project root is determined by finding the nearest parent directory containing a `.git` folder.

## File Patterns

Both explicit paths and glob patterns are supported:
- Explicit: `src/lib/config.rs`
- Glob: `src/**/*.rs`
- Directory: `src/lib/` (hashes all files recursively, ignores hidden files)

## Status Values

- **CURRENT** - Hash matches, documentation is up-to-date
- **DRIFTED** - Hash doesn't match stored value
- **MISSING** - File had a hash but no longer exists
- **INVALID** - Entry has no hash (malformed frontmatter)

## Tech Stack
- Rust
- Shell install script for easy setup
- TUI crate for interactive checkbox selection (exception to minimal dependencies rule)

## Project Conventions

### Code Style
Follow Rust standards (rustfmt, clippy).

### Testing Strategy
Unit tests using the standard Rust testing approach.

### Dependency Philosophy
Minimize third-party crates. Exceptions:
- CLI argument parsing (if standard library is insufficient)
- TUI library for interactive selection (required for good UX)

## Domain Context
Supports Linux and macOS operating systems.

## External Dependencies
None. This should have no reliance on external tools aside from those installed in operating systems.
