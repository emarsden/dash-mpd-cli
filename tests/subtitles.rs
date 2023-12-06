/// Tests for subtitle support
///
/// We don't run these tests on CI infrastructure, because they consume non-negligeable network
/// bandwidth.

// To run tests while enabling printing to stdout/stderr, "cargo test -- --show-output" (from the
// root crate directory).


pub mod common;
use fs_err as fs;
use std::env;
use std::path::Path;
use assert_cmd::Command;
use assert_fs::{prelude::*, TempDir};
use common::check_file_size_approx;


#[test]
fn test_subtitles_wvtt () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://storage.googleapis.com/shaka-demo-assets/sintel-mp4-wvtt/dash.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("sintel.mp4");
    let mut subpath = out.to_path_buf();
    subpath.set_extension("srt");
    let subpath = Path::new(&subpath);
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v",
               "--quality", "worst",
               "--write-subs",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 25_950_458);
    assert!(fs::metadata(subpath).is_ok());
    let srt = fs::read_to_string(subpath).unwrap();
    // We didn't specify a preferred language, so the first available one in the manifest (here
    // Dutch) is downloaded.
    assert!(srt.contains("land van de poortwachters"));

    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v",
               "--quality", "worst",
               "--write-subs",
               "--prefer-language", "eng",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    let srt = fs::read_to_string(subpath).unwrap();
    // This time we requested English subtitles.
    assert!(srt.contains("land of the gatekeepers"));
}


#[test]
fn test_subtitles_ttml () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://dash.akamaized.net/dash264/TestCases/4b/qualcomm/2/TearsOfSteel_onDem5secSegSubTitles.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("tears-of-steel.mp4");
    let mut subpath = out.to_path_buf();
    subpath.set_extension("ttml");
    let subpath = Path::new(&subpath);
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v",
               "--quality", "worst",
               "--write-subs",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 46_299_053);
    assert!(fs::metadata(subpath).is_ok());
    let ttml = fs::read_to_string(subpath).unwrap();
    // We didn't specify a preferred language, so the first available one in the manifest (here
    // English) is downloaded.
    assert!(ttml.contains("You're a jerk"));

    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v",
               "--quality", "worst",
               "--write-subs",
               "--prefer-language", "de",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    let ttml = fs::read_to_string(subpath).unwrap();
    // This time we requested German subtitles.
    assert!(ttml.contains("Du bist ein Vollidiot"));
}


#[test]
fn test_subtitles_vtt () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "http://dash.edgesuite.net/akamai/test/caption_test/ElephantsDream/elephants_dream_480p_heaac5_1.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("elephants-dream.mp4");
    let mut subpath = out.to_path_buf();
    subpath.set_extension("vtt");
    let subpath = Path::new(&subpath);
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v",
               "--quality", "worst",
               "--write-subs",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 128_768_482);
    assert!(fs::metadata(subpath).is_ok());
    // This manifest contains a single subtitle track, available in VTT format via BaseURL addressing.
    let vtt = fs::read_to_string(subpath).unwrap();
    assert!(vtt.contains("Hurry Emo!"));
}
