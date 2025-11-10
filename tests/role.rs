/// Tests for selection-by-role support
///
/// We don't run these tests on CI infrastructure, because they consume non-negligeable network
/// bandwidth.

// To run tests while enabling printing to stdout/stderr
//
//    cargo test --test role -- --show-output


pub mod common;
use fs_err as fs;
use std::env;
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::{prelude::*, TempDir};
use ffprobe::ffprobe;
use file_format::FileFormat;
use test_log::test;
use common::check_file_size_approx;


#[test]
fn test_role_alternate () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "http://dash.edgesuite.net/dash264/TestCasesIOP41/MultiTrack/alternative_content/1/manifest_alternative_content_live.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("role-alternate.mp4");
    cargo_bin_cmd!()
        .args(["-v",
               "--quality", "worst",
               "--role-preference", "alternate",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 94_921_879);
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
    // This is the discriminating test: the "main" stream has hevc content, and the alternate stream
    // has H264/AVC1, both in 1920x1080 resolution, file sizes are very similar.
    assert_eq!(video.codec_name, Some(String::from("h264")));
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}


// TODO: select per Viewpoint element with
// http://dash.edgesuite.net/dash264/TestCasesIOP41/MultiTrack/alternative_content/3/manifest_alternative_maxWidth_live.mpd

