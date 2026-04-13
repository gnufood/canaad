//! tests for the `validate` subcommand.
//!
//! covers success + quiet flag, bad structure, and missing required field.

use assert_cmd::{cargo::cargo_bin_cmd, Command};
use predicates::prelude::*;

const VALID_JSON: &str =
    r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;

fn cmd() -> Command {
    cargo_bin_cmd!("canaad")
}

#[test]
fn valid_json_exits_zero_and_prints_valid() {
    cmd()
        .args(["validate", VALID_JSON])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn quiet_flag_suppresses_stdout() {
    cmd().args(["validate", "--quiet", VALID_JSON]).assert().success().stdout("");
}

#[test]
fn structurally_invalid_json_exits_one() {
    cmd().args(["validate", "{not json}"]).assert().failure().code(1);
}

#[test]
fn missing_required_field_exits_one() {
    // omit purpose — a required field
    let no_purpose = r#"{"v":1,"tenant":"org_abc","resource":"secrets/db"}"#;
    cmd().args(["validate", no_purpose]).assert().failure().code(1);
}
