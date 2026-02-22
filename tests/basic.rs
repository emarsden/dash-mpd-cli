//! Tests for basic download functionality.
///
/// We don't run these tests on CI infrastructure, because they consume non-negligeable network
/// bandwidth.
///
// To run tests while enabling printing to stdout/stderr
//
//    cargo test --test basic -- --show-output


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
fn test_dl_mp4 () {
    let mpd = "https://cloudflarestream.com/31c9291ab41fac05471db4e73aa11717/manifest/video.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("cf.mp4");
    cargo_bin_cmd!()
        .args(["-v",
               "--quality", "worst",
               "--write-subs",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 410_218);
    let format = FileFormat::from_file(&out).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}


#[test]
fn test_dl_mp4a () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://dash.akamaized.net/dash264/TestCases/3a/fraunhofer/aac-lc_stereo_without_video/Sintel/sintel_audio_only_aaclc_stereo_sidx.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("sintel-audio.mp4");
    cargo_bin_cmd!()
        .args(["-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 7_456_334);
    let format = FileFormat::from_file(&out).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Audio);
    let meta = ffprobe(&out).unwrap();
    assert_eq!(meta.streams.len(), 1);
    let audio = &meta.streams[0];
    assert_eq!(audio.codec_type, Some(String::from("audio")));
    assert_eq!(audio.codec_name, Some(String::from("aac")));
    assert!(audio.width.is_none());
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}


#[test]
fn test_dl_audio_flac () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "http://rdmedia.bbc.co.uk/testcard/vod/manifests/radio-flac-en.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("bbcradio-flac.mp4");
    cargo_bin_cmd!()
        .args(["-v",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 81_603_640);
    let format = FileFormat::from_file(&out).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Audio);
    let meta = ffprobe(&out).unwrap();
    assert_eq!(meta.streams.len(), 1);
    let audio = &meta.streams[0];
    assert_eq!(audio.codec_type, Some(String::from("audio")));
    assert_eq!(audio.codec_name, Some(String::from("flac")));
    assert!(audio.width.is_none());
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}

#[test]
fn test_dl_cmaf () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://cdn.theoplayer.com/video/cosmos/cmaf.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("theo-cosmos.mp4");
    cargo_bin_cmd!()
        .args(["-v",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 21_854_800);
    let format = FileFormat::from_file(&out).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    let meta = ffprobe(&out).unwrap();
    assert_eq!(meta.streams.len(), 2);
    let audio = meta.streams.iter()
        .find(|s| s.codec_type.eq(&Some(String::from("audio"))))
        .expect("finding audio stream");
    assert_eq!(audio.codec_name, Some(String::from("aac")));
    assert!(audio.width.is_none());
    let video = meta.streams.iter()
        .find(|s| s.codec_type.eq(&Some(String::from("video"))))
        .expect("finding video stream");
    assert_eq!(video.codec_name, Some(String::from("h264")));
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}

#[test]
fn test_dl_timecode () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "http://dash.edgesuite.net/dash264/AdContent_timecode/Ondemand/DuneIOSAdTimecode_ondemand.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("dune-adtime.mp4");
    cargo_bin_cmd!()
        .args(["-v",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 3_405_107);
    let format = FileFormat::from_file(&out).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    let meta = ffprobe(&out).unwrap();
    assert_eq!(meta.streams.len(), 2);
    let audio = meta.streams.iter()
        .find(|s| s.codec_type.eq(&Some(String::from("audio"))))
        .expect("finding audio stream");
    assert_eq!(audio.codec_name, Some(String::from("aac")));
    assert!(audio.width.is_none());
    let video = meta.streams.iter()
        .find(|s| s.codec_type.eq(&Some(String::from("video"))))
        .expect("finding video stream");
    assert_eq!(video.codec_name, Some(String::from("h264")));
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}

// The manifest contains minBufferTime="4S", which is an invalid format for an xs:Duration.
#[test]
fn test_parse_failure_duration () {
    cargo_bin_cmd!()
        .args(["https://dash.akamaized.net/akamai/test/manifest3.mpd"])
        .assert()
        .stderr(predicate::str::contains("Download failed"))
        .stderr(predicate::str::contains("invalid Duration"))
        .failure();
}
