//! tests for the `canonicalize` subcommand.
//!
//! covers all output formats (utf8, hex, base64, raw), --to-file, -f FILE input,
//! and invalid-json rejection. all expected values derived from spec §10.1.

use assert_cmd::{cargo::cargo_bin_cmd, Command};
use predicates::prelude::*;
use tempfile::NamedTempFile;

/// minimal valid AAD JSON (field order does not matter on input).
const VALID_JSON: &str =
    r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;

const CANONICAL: &str =
    r#"{"purpose":"encryption","resource":"secrets/db","tenant":"org_abc","v":1}"#;

/// hex encoding of CANONICAL (no newline in source bytes).
const CANONICAL_HEX: &str = "7b22707572706f7365223a22656e6372797074696f6e222c\
    227265736f75726365223a22736563726574732f6462222c\
    2274656e616e74223a226f72675f616263222c2276223a317d";

const CANONICAL_B64: &str =
    "eyJwdXJwb3NlIjoiZW5jcnlwdGlvbiIsInJlc291cmNlIjoic2VjcmV0cy9kYiIsInRlbmFudCI6Im9yZ19hYmMiLCJ2IjoxfQ==";

fn cmd() -> Command {
    cargo_bin_cmd!("canaad")
}

#[test]
fn utf8_format_is_default_and_exits_zero() {
    cmd()
        .args(["canonicalize", VALID_JSON])
        .assert()
        .success()
        .stdout(predicate::str::contains(CANONICAL));
}

#[test]
fn utf8_output_has_trailing_newline() {
    let expected = format!("{CANONICAL}\n");
    cmd().args(["canonicalize", VALID_JSON]).assert().success().stdout(expected);
}

#[test]
fn hex_output_format() {
    // strip whitespace from the multi-line constant before comparing
    let expected = format!("{}\n", CANONICAL_HEX.replace(' ', ""));
    cmd().args(["canonicalize", "-o", "hex", VALID_JSON]).assert().success().stdout(expected);
}

#[test]
fn base64_output_format() {
    let expected = format!("{CANONICAL_B64}\n");
    cmd().args(["canonicalize", "-o", "base64", VALID_JSON]).assert().success().stdout(expected);
}

#[test]
fn raw_output_has_no_trailing_newline() {
    let expected = CANONICAL.as_bytes().to_vec();
    cmd().args(["canonicalize", "-o", "raw", VALID_JSON]).assert().success().stdout(expected);
}

#[test]
fn to_file_writes_file_and_stdout_is_empty() {
    let out = NamedTempFile::new().expect("tempfile must be creatable");
    let path = out.path().to_str().expect("tempfile path must be utf-8");
    cmd().args(["canonicalize", "--to-file", path, VALID_JSON]).assert().success().stdout("");
    let contents = std::fs::read_to_string(path).expect("output file must be readable");
    assert!(contents.contains(CANONICAL), "file must contain canonical JSON");
}

#[test]
fn file_input_flag() {
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
fn invalid_json_exits_one() {
    cmd().args(["canonicalize", "not-valid-json"]).assert().failure().code(1);
}
