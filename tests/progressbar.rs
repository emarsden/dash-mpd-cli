//! Tests for progressbar and LDJSON logging functionality
///
///
// To run tests while enabling printing to stdout/stderr
//
//    cargo test --test progressbar -- --show-output


pub mod common;
use fs_err as fs;
use std::env;
use ffprobe::ffprobe;
use file_format::FileFormat;
use predicates::prelude::*;
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::{prelude::*, TempDir};
use test_log::test;
use common::check_file_size_approx;



#[test]
fn test_progressbar_indicatif () {
    let mpd = "https://cloudflarestream.com/31c9291ab41fac05471db4e73aa11717/manifest/video.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("cf.mp4");
    cargo_bin_cmd!()
        .args(["-v",
               "--quality", "worst",
               "--progress=bar",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 410_218);
    let format = FileFormat::from_file(&out).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    let meta = ffprobe(&out).unwrap();
    assert_eq!(meta.streams.len(), 1);
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}


#[test]
fn test_progressbar_json () {
    let mpd = "https://cloudflarestream.com/31c9291ab41fac05471db4e73aa11717/manifest/video.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("cf.mp4");
    cargo_bin_cmd!()
        .args(["-v",
               "--quality", "worst",
               "--progress=json",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .stderr(predicate::str::contains("\"type\": \"progress\""))
        .stderr(predicate::str::contains("\"percent\": 100"))
        .success();
    check_file_size_approx(&out, 410_218);
    let format = FileFormat::from_file(&out).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}

