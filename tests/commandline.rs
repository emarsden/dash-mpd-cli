/// Tests for various commandline option behaviour.

// To run tests while enabling printing to stdout/stderr
//
//    cargo test --test commandline -- --show-output


use std::process::Command;


#[test]
fn test_command_spurious () {
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--spurious-option",
               "https://example.org/mpd"])
        .output()
        .expect("failure spawning dash-mpd-cli");
    assert!(!cli.status.success());
    let msg = String::from_utf8_lossy(&cli.stderr);
    assert!(msg.contains("unexpected argument"));
    assert!(msg.contains("Usage:"));
}

#[test]
fn test_command_mpd_missing () {
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--verbose"])
        .output()
        .expect("failure spawning dash-mpd-cli");
    assert!(!cli.status.success());
    let msg = String::from_utf8_lossy(&cli.stderr);
    assert!(msg.contains("following required arguments were not provided"));
    assert!(msg.contains("Usage:"));
}

#[test]
fn test_command_have_help () {
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--help"])
        .output()
        .expect("failure spawning dash-mpd-cli");
    assert!(cli.status.success());
    let msg = String::from_utf8_lossy(&cli.stdout);
    assert!(msg.lines().count() > 20);
    assert!(msg.contains("--help"));
    assert!(msg.contains("Usage:"));
}

#[test]
fn test_command_missing_file () {
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--add-root-certificate", "/missing/file",
               "https://example.org/ignored.mpd"])
        .output()
        .expect("failure spawning dash-mpd-cli");
    assert!(!cli.status.success());
    let msg = String::from_utf8_lossy(&cli.stderr);
    assert!(msg.contains("Can't read root certificate"));
    assert!(msg.contains("/missing/file"));
}

#[test]
fn test_command_missing_value () {
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--max-error-count",
               "https://example.org/ignored.mpd"])
        .output()
        .expect("failure spawning dash-mpd-cli");
    assert!(!cli.status.success());
    let msg = String::from_utf8_lossy(&cli.stderr);
    assert!(msg.contains("error: invalid value"));
    assert!(msg.contains("--help"));
}

#[test]
fn test_command_funky_source () {
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--source-address", "33.44.55.66.77",
               "https://example.org/ignored.mpd"])
        .output()
        .expect("failure spawning dash-mpd-cli");
    assert!(!cli.status.success());
    let msg = String::from_utf8_lossy(&cli.stderr);
    assert!(msg.contains("Ignoring invalid argument to --source-address"));
}

#[test]
fn test_command_bad_rate () {
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--limit-rate", "42Z",
               "https://example.org/ignored.mpd"])
        .output()
        .expect("failure spawning dash-mpd-cli");
    assert!(!cli.status.success());
    let msg = String::from_utf8_lossy(&cli.stderr);
    assert!(msg.contains("Ignoring unrecognized suffix on limit-rate"));
}


