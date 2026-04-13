//! CLI for AAD canonicalization per RFC 8785.
//!
//! Accepts JSON input from an inline argument, a file (`-f`), or piped stdin.
//! Stdin is only read when it is not a terminal; interactive stdin is rejected
//! with an error.
//!
//! Exit codes:
//! - `0` — success
//! - `1` — validation error (malformed JSON, schema violation)
//! - `2` — I/O error (unreadable file, stdin failure)

mod args;
mod commands;
mod io;

use std::process::ExitCode;

const EXIT_VALIDATION: u8 = 1;
const EXIT_IO: u8 = 2;

fn main() -> ExitCode {
    match commands::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let exit_code =
                if commands::is_validation_error(&e) { EXIT_VALIDATION } else { EXIT_IO };
            eprintln!("error: {e:#}");
            ExitCode::from(exit_code)
        }
    }
}
