//! tests for the exit code contract.
//!
//! 0 = success, 1 = validation/aad error, 2 = i/o error.
//! validates that 1 and 2 are distinct and that each trigger class maps
//! to the correct code.

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;

const VALID_JSON: &str =
    r#"{"v":1,"tenant":"org_abc","resource":"secrets/db","purpose":"encryption"}"#;

fn cmd() -> Command {
    cargo_bin_cmd!("canaad")
}

#[test]
fn valid_json_exits_zero() {
    cmd().args(["canonicalize", VALID_JSON]).assert().code(0);
}

#[test]
fn structurally_invalid_json_exits_one() {
    // not parseable as JSON at all
    cmd().args(["canonicalize", "{"]).assert().code(1);
}

#[test]
fn invalid_aad_field_exits_one() {
    // tenant exceeds 256-byte maximum
    let long_tenant = "a".repeat(257);
    let json = format!(
        r#"{{"v":1,"tenant":"{long_tenant}","resource":"secrets/db","purpose":"encryption"}}"#
    );
    cmd().args(["canonicalize", &json]).assert().code(1);
}

#[test]
fn nonexistent_file_exits_two() {
    cmd().args(["canonicalize", "-f", "/tmp/canaad-no-such-file-abc123.json"]).assert().code(2);
}

#[test]
fn exit_one_and_exit_two_are_distinct() {
    let validation_code = cmd()
        .args(["canonicalize", "bad json"])
        .output()
        .expect("command must run")
        .status
        .code()
        .expect("process must exit with a code");

    let io_code = cmd()
        .args(["canonicalize", "-f", "/tmp/canaad-no-such-file-abc123.json"])
        .output()
        .expect("command must run")
        .status
        .code()
        .expect("process must exit with a code");

    assert_eq!(validation_code, 1, "validation errors must be code 1");
    assert_eq!(io_code, 2, "i/o errors must be code 2");
    assert_ne!(validation_code, io_code, "codes must differ");
}
