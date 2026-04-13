//! Input routing: stdin detection, file reading, and argument string handling.

use std::fs::File;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;

use anyhow::{Context, Result};
use canaad_core::MAX_AAD_SIZE;

/// Reads at most `MAX_AAD_SIZE + 1` bytes from `reader` and returns the result
/// as a `String`, erroring if the content exceeds `MAX_AAD_SIZE`.
pub(crate) fn read_bounded<R: Read>(reader: R, source: &str) -> Result<String> {
    let mut buf = String::new();
    reader
        .take((MAX_AAD_SIZE as u64) + 1)
        .read_to_string(&mut buf)
        .with_context(|| format!("failed to read from {source}"))?;
    if buf.len() > MAX_AAD_SIZE {
        anyhow::bail!("input exceeds maximum allowed size ({MAX_AAD_SIZE} bytes)");
    }
    Ok(buf)
}

/// Read input from the CLI argument, a file path, or stdin — checked in that order.
///
/// # Errors
///
/// - No source provided and stdin is a terminal (interactive, no pipe).
/// - File path given but unreadable.
/// - Input exceeds `MAX_AAD_SIZE` bytes.
/// - Stdin read fails.
pub(crate) fn read_input(input: Option<String>, file: Option<PathBuf>) -> Result<String> {
    if let Some(json) = input {
        return Ok(json);
    }

    if let Some(path) = file {
        let f = File::open(&path)
            .with_context(|| format!("failed to read file: {}", path.display()))?;
        return read_bounded(f, &path.display().to_string());
    }

    if io::stdin().is_terminal() {
        anyhow::bail!("no input provided: use argument, -f FILE, or pipe to stdin");
    }

    read_bounded(io::stdin().lock(), "stdin")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// SEC-01: `read_bounded` must reject input larger than `MAX_AAD_SIZE`.
    #[test]
    fn read_bounded_rejects_oversized_input() {
        let oversized = vec![b'x'; MAX_AAD_SIZE + 1];
        let cursor = Cursor::new(oversized);
        let result = read_bounded(cursor, "test-source");
        assert!(result.is_err(), "expected error for oversized input");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("exceeds maximum allowed size"),
            "expected size-limit message, got: {msg}"
        );
    }

    /// SEC-01: `read_bounded` must accept input exactly at the limit.
    #[test]
    fn read_bounded_accepts_input_at_limit() {
        let at_limit = vec![b'x'; MAX_AAD_SIZE];
        let cursor = Cursor::new(at_limit);
        let result = read_bounded(cursor, "test-source");
        assert!(result.is_ok(), "expected Ok for input at the limit");
    }
}
