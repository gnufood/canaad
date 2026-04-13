//! CLI argument structs, subcommand enums, and output format types.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// AAD canonicalization tool per RFC 8785.
#[derive(Parser)]
#[command(name = "canaad")]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    pub(crate) command: Commands,
}

/// Canonicalize, validate, or hash AAD JSON.
#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Canonicalize AAD JSON to deterministic form.
    Canonicalize {
        /// JSON input as command-line argument.
        input: Option<String>,

        /// Read input from file.
        #[arg(short = 'f', long = "file", conflicts_with = "input")]
        file: Option<PathBuf>,

        /// Output format.
        #[arg(short = 'o', long = "output", default_value = "utf8")]
        output_format: OutputFormat,

        /// Write output to file instead of stdout.
        #[arg(long = "to-file")]
        to_file: Option<PathBuf>,
    },

    /// Validate AAD JSON structure and fields.
    Validate {
        /// JSON input as command-line argument.
        input: Option<String>,

        /// Read input from file.
        #[arg(short = 'f', long = "file", conflicts_with = "input")]
        file: Option<PathBuf>,

        /// Suppress success message (exit code only).
        #[arg(short = 'q', long = "quiet")]
        quiet: bool,
    },

    /// Compute SHA-256 hash of canonical AAD.
    Hash {
        /// JSON input as command-line argument.
        input: Option<String>,

        /// Read input from file.
        #[arg(short = 'f', long = "file", conflicts_with = "input")]
        file: Option<PathBuf>,

        /// Output format.
        #[arg(short = 'o', long = "output", default_value = "hex")]
        output_format: HashOutputFormat,
    },
}

/// Output format for the canonicalize command.
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum OutputFormat {
    /// UTF-8 string with newline.
    Utf8,
    /// Hexadecimal encoding.
    Hex,
    /// Base64 encoding.
    Base64,
    /// Raw binary (no newline).
    Raw,
}

/// Output format for the hash command.
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum HashOutputFormat {
    /// Hexadecimal encoding.
    Hex,
    /// Base64 encoding.
    Base64,
}
