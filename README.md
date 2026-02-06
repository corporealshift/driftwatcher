# Driftwatcher

A tool to help watch for documentation drift.

It looks at all markdown files in your project for the driftwatcher YAML frontmatter and compares
the stored hashes with fresh hashes of the tracked files. It will list all documentation
files where the associated code files have changed, and you can then choose to update
the hashes or leave them as-is for later review.



## Getting started

Install via curl:

```bash
curl -fsSL https://raw.githubusercontent.com/corporealshift/driftwatcher/main/install.sh | bash
```

Or specify a custom install directory:

```bash
curl -fsSL https://raw.githubusercontent.com/corporealshift/driftwatcher/main/install.sh | INSTALL_DIR=~/.local/bin bash
```

### Build from source

```bash
git clone https://github.com/corporealshift/driftwatcher.git
cd driftwatcher
cargo build --release
cp target/release/drifty /usr/local/bin/
```

## Usage

Once installed you can run these commands (run just `drifty` to see this usage):

- `drifty init <doc-file>`: Initializes the doc file with an empty drifty table.
- `drifty add <doc-file> <file-to-watch>`: Adds a file to watch to the doc file's frontmatter and computes its initial hash.
- `drifty check`: Checks all documentation in the current directory (recursively) and checks if there are any updates. Provides an interactive update system.
- `drifty check <filename>`: Checks the specific file or directory.
- `drifty report --format json|yaml`: Reports status of all tracked files. Useful for CI.
- `drifty validate`: Verifies that all drifty YAML front matter is valid, including file paths.

## How it works

DriftWatcher operates on markdown files. It uses YAML front matter to store a map of
files (can be any kind of file) or directories to an SHA256 hash of the files/directory.
Once the map of paths and hashes is created, running `drifty check` will read the map,
create new hashes of the files/directories, and compare to the ones in the table.

**Path resolution:** Paths can be relative to the markdown file's location, or prefixed
with `$ROOT/` to be relative to the project root (the nearest parent directory containing
a `.git` folder).

**Glob patterns:** You can use glob patterns to watch multiple files (e.g., `src/**/*.rs`).

**Directory hashing:** When watching a directory, Driftwatcher hashes all files recursively
but ignores hidden files (those starting with `.`).

**Status reporting:**
- **CURRENT** - The file's hash matches, documentation is up-to-date.
- **DRIFTED** - The file's current hash doesn't match the stored hash.
- **MISSING** - The file had a hash but no longer exists.
- **INVALID** - The entry has no hash (malformed frontmatter). 

## Examples

See the [examples/](examples/) directory for complete working examples:

- [examples/basic.md](examples/basic.md) - Simple single-file watching
- [examples/glob-patterns.md](examples/glob-patterns.md) - Using glob patterns to watch multiple files
- [examples/root-paths.md](examples/root-paths.md) - Using `$ROOT/` prefix for project-relative paths

### Basic Example

```markdown
---
driftwatcher:
  - src/lib/config.rs: a1b2c3d4
  - src/lib/db_conn.rs: e5f6g7h8
  - src/models/**/*.rs: f9g0h1i2
---

# All about db connection configuration
...
```

With the above driftwatcher YAML frontmatter this documentation file is now discoverable
to Driftwatcher. When the `drifty check` command is run, it will hash the config.rs and
db_conn.rs files and compare the hashes in the map to the new ones. If they are different
you will get prompted with a select list of files to update. You can select individual
files, respond with (a) to update all, or (n) to update none.

### Quick Start

```bash
# Initialize a doc file with driftwatcher frontmatter
drifty init docs/api.md

# Add files to watch
drifty add docs/api.md src/api/handler.rs
drifty add docs/api.md "src/api/**/*.rs"

# Check for drift
drifty check

# Or just report status (useful for CI)
drifty report --format json
```

## Report output

The `drifty report` command outputs status for all tracked files. Default is plaintext,
or use `--format json` or `--format yaml`.

**Plaintext:**
```
docs/database.md
  CURRENT  src/lib/config.rs
  DRIFTED  src/lib/db_conn.rs

docs/auth.md
  CURRENT  src/auth/login.rs
  MISSING  src/auth/oauth.rs
```

**JSON:**
```json
{
  "docs/database.md": {
    "src/lib/config.rs": "CURRENT",
    "src/lib/db_conn.rs": "DRIFTED"
  },
  "docs/auth.md": {
    "src/auth/login.rs": "CURRENT",
    "src/auth/oauth.rs": "MISSING"
  }
}
```

**YAML:**
```yaml
docs/database.md:
  src/lib/config.rs: CURRENT
  src/lib/db_conn.rs: DRIFTED
docs/auth.md:
  src/auth/login.rs: CURRENT
  src/auth/oauth.rs: MISSING
```
