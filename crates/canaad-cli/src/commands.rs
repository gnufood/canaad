//! Command handler functions for canonicalize, validate, and hash subcommands.

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use base64::prelude::*;
use clap::Parser;
use sha2::{Digest, Sha256};

use canaad_core::{canonicalize, validate, AadError};

use crate::args::{Cli, Commands, HashOutputFormat, OutputFormat};
use crate::io::read_input;

pub(crate) fn is_validation_error(err: &anyhow::Error) -> bool {
    err.downcast_ref::<AadError>().is_some()
        || err.chain().any(|e| e.downcast_ref::<AadError>().is_some())
}

/// Parse CLI args and dispatch to the matched subcommand handler.
///
/// # Errors
///
/// Propagates any error returned by the subcommand handler.
pub(crate) fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Canonicalize { input, file, output_format, to_file } => {
            cmd_canonicalize(input, file, output_format, to_file)
        }
        Commands::Validate { input, file, quiet } => cmd_validate(input, file, quiet),
        Commands::Hash { input, file, output_format } => cmd_hash(input, file, output_format),
    }
}

fn cmd_canonicalize(
    input: Option<String>,
    file: Option<PathBuf>,
    output_format: OutputFormat,
    to_file: Option<PathBuf>,
) -> Result<()> {
    let json = read_input(input, file)?;
    let canonical = canonicalize(&json).context("failed to canonicalize")?;

    let output: Vec<u8> = match output_format {
        OutputFormat::Utf8 => {
            let mut s =
                String::from_utf8(canonical).context("canonical output is not valid UTF-8")?;
            s.push('\n');
            s.into_bytes()
        }
        OutputFormat::Hex => {
            let mut s = hex::encode(&canonical);
            s.push('\n');
            s.into_bytes()
        }
        OutputFormat::Base64 => {
            let mut s = BASE64_STANDARD.encode(&canonical);
            s.push('\n');
            s.into_bytes()
        }
        OutputFormat::Raw => canonical,
    };

    if let Some(path) = to_file {
        fs::write(&path, &output)
            .with_context(|| format!("failed to write to {}", path.display()))?;
    } else {
        io::stdout().write_all(&output).context("failed to write to stdout")?;
    }

    Ok(())
}

fn cmd_validate(input: Option<String>, file: Option<PathBuf>, quiet: bool) -> Result<()> {
    let json = read_input(input, file)?;
    validate(&json).context("validation failed")?;

    if !quiet {
        println!("valid");
    }

    Ok(())
}

fn cmd_hash(
    input: Option<String>,
    file: Option<PathBuf>,
    output_format: HashOutputFormat,
) -> Result<()> {
    let json = read_input(input, file)?;
    let canonical = canonicalize(&json).context("failed to canonicalize")?;

    let hash = Sha256::digest(&canonical);

    let output = match output_format {
        HashOutputFormat::Hex => hex::encode(hash),
        HashOutputFormat::Base64 => BASE64_STANDARD.encode(hash),
    };

    println!("{output}");

    Ok(())
}
