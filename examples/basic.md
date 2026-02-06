---
driftwatcher:
  - "../src/main.rs": abc123
---

# CLI Entry Point

This document describes the main entry point for the drifty CLI tool.

The `main.rs` file handles argument parsing and dispatches to the appropriate
command handler.

## Overview

When you run `drifty`, it:
1. Parses command-line arguments using clap
2. Matches the subcommand (init, add, check, report, validate)
3. Calls the corresponding handler in the `commands` module
