/// Tests for various commandline option behaviour.

// To run tests while enabling printing to stdout/stderr
//
//    cargo test --test commandline -- --show-output

use predicates::prelude::*;
use assert_cmd::Command;
use test_log::test;


#[test]
fn test_command_spurious () {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["--spurious-option",
               "https://example.org/mpd"])
        .assert()
        .stderr(predicate::str::contains("unexpected argument"))
        .stderr(predicate::str::contains("Usage:"))
        .failure();
}

#[test]
fn test_command_mpd_missing () {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["--verbose"])
        .assert()
        .stderr(predicate::str::contains("following required arguments were not provided"))
        .stderr(predicate::str::contains("Usage:"))
        .failure();
}

#[test]
fn test_command_have_help () {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["--help"])
        .assert()
        .stdout(predicate::str::contains("--help"))
        .stdout(predicate::str::contains("Usage:"))
        .success();
    // We can't implement this with predicates crate
    // assert!(msg.lines().count() > 20);
}

#[test]
fn test_command_missing_file () {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["--add-root-certificate", "/missing/file",
               "https://example.org/ignored.mpd"])
        .assert()
        .stderr(predicate::str::contains("Can't read root certificate"))
        .stderr(predicate::str::contains("/missing/file"))
        .failure();
}

#[test]
fn test_command_missing_value () {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["--max-error-count",
               "https://example.org/ignored.mpd"])
        .assert()
        .stderr(predicate::str::contains("error: invalid value"))
        .stderr(predicate::str::contains("--help"))
        .failure();
}

#[test]
fn test_command_funky_source () {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["--source-address", "33.44.55.66.77",
               "https://example.org/ignored.mpd"])
        .assert()
        .stderr(predicate::str::contains("Ignoring invalid argument to --source-address"))
        .failure();
}

#[test]
fn test_command_bad_rate () {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["--limit-rate", "42Z",
               "https://example.org/ignored.mpd"])
        .assert()
        .stderr(predicate::str::contains("Ignoring unrecognized suffix on limit-rate"))
        .failure();
}


