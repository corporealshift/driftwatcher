use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

mod commands;
mod frontmatter;
mod hash;
mod paths;
mod scanner;
mod status;

#[derive(Parser)]
#[command(name = "drifty")]
#[command(about = "Watch for documentation drift")]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a markdown file with empty driftwatcher frontmatter
    Init {
        /// The documentation file to initialize
        doc_file: PathBuf,
    },

    /// Add a file or pattern to watch in a documentation file
    Add {
        /// The documentation file to update
        doc_file: PathBuf,

        /// The file, directory, or glob pattern to watch
        watch_pattern: String,
    },

    /// Check all documentation for drift (interactive)
    Check {
        /// Specific file or directory to check (default: current directory)
        target: Option<PathBuf>,
    },

    /// Report status of all tracked files
    Report {
        /// Output format
        #[arg(long, short, default_value = "plaintext")]
        format: OutputFormat,
    },

    /// Validate all driftwatcher frontmatter
    Validate,

    /// Show this help message
    Help,
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Plaintext,
    Json,
    Yaml,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Help) => {
            print_help();
            Ok(())
        }
        Some(Commands::Init { doc_file }) => commands::init::run(&doc_file),
        Some(Commands::Add {
            doc_file,
            watch_pattern,
        }) => commands::add::run(&doc_file, &watch_pattern),
        Some(Commands::Check { target }) => commands::check::run(target.as_deref()),
        Some(Commands::Report { format }) => commands::report::run(format.into()),
        Some(Commands::Validate) => commands::validate::run(),
    }
}

fn print_help() {
    println!(
        r#"drifty - Watch for documentation drift

Usage:

  drifty init <doc-file>
      Initializes the doc file with an empty driftwatcher table.

  drifty add <doc-file> <file-to-watch>
      Adds a file to watch to the doc file's frontmatter and computes its
      initial hash.

  drifty check [<filename>]
      Checks all documentation in the current directory (recursively) and
      checks if there are any updates. Provides an interactive update system.
      Optionally specify a specific file or directory to check.

  drifty report [--format json|yaml|plaintext]
      Reports status of all tracked files. Useful for CI.

  drifty validate
      Verifies that all driftwatcher YAML front matter is valid, including
      file paths.

  drifty help
      Show this help message."#
    );
}

impl From<OutputFormat> for commands::report::OutputFormat {
    fn from(f: OutputFormat) -> Self {
        match f {
            OutputFormat::Plaintext => commands::report::OutputFormat::Plaintext,
            OutputFormat::Json => commands::report::OutputFormat::Json,
            OutputFormat::Yaml => commands::report::OutputFormat::Yaml,
        }
    }
}
