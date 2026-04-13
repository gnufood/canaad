//! tests for the `hash` subcommand.
//!
//! covers default hex output, base64 output, and the spec §10.1 known-answer
//! test vector. the sha-256 reference value is
//! 03fdc63d2f82815eb0a97e6f1a02890e152c021a795142b9c22e2b31a3bd83eb.

use assert_cmd::{cargo::cargo_bin_cmd, Command};
use predicates::prelude::*;

/// spec §10.1 input — field order irrelevant before canonicalization.
const SPEC_INPUT: &str =
    r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;

/// sha-256 of the canonical form, per spec §10.1.
const SPEC_SHA256_HEX: &str = "03fdc63d2f82815eb0a97e6f1a02890e152c021a795142b9c22e2b31a3bd83eb";

const SPEC_SHA256_B64: &str = "A/3GPS+CgV6wqX5vGgKJDhUsAhp5UUK5wi4rMaO9g+s=";

fn cmd() -> Command {
    cargo_bin_cmd!("canaad")
}

#[test]
fn default_format_is_hex_and_exits_zero() {
    cmd()
        .args(["hash", SPEC_INPUT])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^[0-9a-f]{64}\n$").expect("regex must compile"));
}

#[test]
fn base64_output_format() {
    cmd()
        .args(["hash", "-o", "base64", SPEC_INPUT])
        .assert()
        .success()
        .stdout(predicate::str::contains(SPEC_SHA256_B64));
}

#[test]
fn spec_10_1_known_answer_vector() {
    // exact match including the trailing newline that println! adds
    let expected = format!("{SPEC_SHA256_HEX}\n");
    cmd().args(["hash", SPEC_INPUT]).assert().success().stdout(expected);
}
