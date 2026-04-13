//! tests for input routing across all three subcommands.
//!
//! verifies inline argument, -f FILE, and nonexistent-file → exit 2.

use assert_cmd::{cargo::cargo_bin_cmd, Command};
use predicates::prelude::*;
use tempfile::NamedTempFile;

const VALID_JSON: &str =
    r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;

const CANONICAL: &str =
    r#"{"purpose":"encryption","resource":"secrets/db","tenant":"org_abc","v":1}"#;

fn cmd() -> Command {
    cargo_bin_cmd!("canaad")
}

#[test]
fn inline_argument_is_used_when_provided() {
    cmd()
        .args(["canonicalize", VALID_JSON])
        .assert()
        .success()
        .stdout(predicate::str::contains(CANONICAL));
}

#[test]
fn file_flag_reads_correct_contents() {
    let mut f = NamedTempFile::new().expect("tempfile must be creatable");
    std::io::Write::write_all(&mut f, VALID_JSON.as_bytes())
        .expect("write to tempfile must succeed");
    let path = f.path().to_str().expect("tempfile path must be utf-8");
    cmd()
        .args(["canonicalize", "-f", path])
        .assert()
        .success()
        .stdout(predicate::str::contains(CANONICAL));
}

#[test]
fn nonexistent_file_exits_two() {
    cmd()
        .args(["canonicalize", "-f", "/tmp/canaad-test-does-not-exist-xyz.json"])
        .assert()
        .failure()
        .code(2);
}
