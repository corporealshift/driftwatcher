---
driftwatcher:
  - "$ROOT/src/hash.rs": jkl012
  - "$ROOT/src/paths.rs": mno345
  - "$ROOT/Cargo.toml": pqr678
---

# Core Utilities

This document describes the core utility modules used throughout driftwatcher.

## Hashing (hash.rs)

The hash module provides SHA-256 hashing for:
- Single files
- Multiple files (sorted alphabetically for determinism)
- Directories (recursive, excluding hidden files)

## Path Resolution (paths.rs)

The paths module handles:
- Resolving paths relative to the document file
- Resolving `$ROOT/` prefixed paths relative to project root
- Glob pattern expansion
- Finding the project root by locating `.git`

## Configuration (Cargo.toml)

The Cargo.toml defines our dependencies and build configuration.
