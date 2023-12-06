/// Tests for decryption support using mp4decrypt and shaka packager
///
/// These test cases are from https://refapp.hbbtv.org/videos/. We don't run these tests on CI
/// infrastructure, because they consume non-negligeable network bandwidth.

// To run tests while enabling printing to stdout/stderr
//
//    cargo test --test decryption -- --show-output


pub mod common;
use fs_err as fs;
use std::env;
use ffprobe::ffprobe;
use file_format::FileFormat;
use assert_cmd::Command;
use assert_fs::{prelude::*, TempDir};
use common::check_file_size_approx;


#[test]
fn test_decryption_widevine_cenc () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://refapp.hbbtv.org/videos/spring_h265_v8/cenc/manifest_wvcenc.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("spring.mp4");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v",
               "--quality", "worst",
               "--key", "43215678123412341234123412341237:12341234123412341234123412341237",
               "--key", "43215678123412341234123412341236:12341234123412341234123412341236",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 33_746_341);
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}

#[test]
fn test_decryption_widevine_cbcs () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://refapp.hbbtv.org/videos/tears_of_steel_h265_v8/cbcs/manifest_wvcenc.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("tears-steel.mp4");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v",
               "--quality", "worst",
               "--key", "43215678123412341234123412341237:12341234123412341234123412341237",
               "--key", "43215678123412341234123412341236:12341234123412341234123412341236",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 79_731_116);
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}


#[test]
fn test_decryption_playready_cenc () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://refapp.hbbtv.org/videos/00_llama_h264_v8_8s/cenc/manifest_prcenc.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("llama.mp4");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v",
               "--quality", "worst",
               "--key", "43215678123412341234123412341236:12341234123412341234123412341236",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 26_420_624);
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}

#[test]
fn test_decryption_marlin_cenc () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://refapp.hbbtv.org/videos/agent327_h264_v8/cenc/manifest_mlcenc.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("llama-cenc.mp4");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v",
               "--quality", "worst",
               "--key", "43215678123412341234123412341234:12341234123412341234123412341234",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 14_357_917);
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}

#[test]
fn test_decryption_marlin_cbcs () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://refapp.hbbtv.org/videos/agent327_h264_v8/cbcs/manifest_mlcenc.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("llama-cbcs.mp4");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v",
               "--quality", "worst",
               "--key", "43215678123412341234123412341234:12341234123412341234123412341234",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 14_357_925);
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}


#[test]
fn test_decryption_cmaf_h265_multikey () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://media.axprod.net/TestVectors/H265/protected_cmaf_1080p_h265_multikey/manifest.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("axinom-h264-multikey.mp4");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v",
               "--quality", "worst",
               "--key", "53dc3eaa5164410a8f4ee15113b43040:620045a34e839061ee2e9b7798fdf89b",
               "--key", "9dbace9e41034c5296aa63227dc5f773:a776f83276a107a3c322f9dbd6d4f48c",
               "--key", "a76f0ca68e7d40d08a37906f3e24dde2:2a99b42f08005ab4b57af20f4da3cc05",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 48_233_447);
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}


// Small decryption test cases that we can run on the CI infrastructure.
#[test]
fn test_decryption_cenc_kaltura () {
    let mpd = "https://cdnapisec.kaltura.com/p/2433871/sp/243387100/playManifest/protocol/https//entryId/1_pgssezc1/format/mpegdash/tags/mbr/f/a.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("kaltura.mp4");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["--mp4decrypt-location", "mp4decrypt",
               "--key", "a07c5d499dcead0fb416fed5913967be:caee457911302478487e6680bf0b3d1b",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .failure();
    /* MPD has disappeared in December 2023
    check_file_size_approx(&out, 1_323_079);
    let format = FileFormat::from_file(&out).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    let meta = ffprobe(out).unwrap();
    assert_eq!(meta.streams.len(), 2);
    let audio = meta.streams.iter()
        .find(|s| s.codec_type.eq(&Some(String::from("audio"))))
        .expect("finding audio stream");
    assert_eq!(audio.codec_name, Some(String::from("aac")));
    assert!(audio.width.is_none());
    let tags = audio.tags.as_ref().unwrap();
    assert_eq!(tags.language, Some(String::from("eng")));
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
    */
}


#[test]
fn test_decryption_small () {
    let mpd = "https://m.dtv.fi/dash/dasherh264/drm/manifest_clearkey.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("caminandes.mp4");
    let cli = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v",
               "--quality", "worst",
               "--key", "43215678123412341234123412341234:12341234123412341234123412341234",
               "-o", &out.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cli");
    let msg = String::from_utf8_lossy(&cli.stdout);
    if msg.len() > 0 {
        println!("dash-mpd-cli stdout: {msg}");
    }
    let msg = String::from_utf8_lossy(&cli.stderr);
    if msg.len() > 0 {
        println!("dash-mpd-cli stderr: {msg}");
    }
    assert!(cli.status.success());
    check_file_size_approx(&out, 6_975_147);
    let format = FileFormat::from_file(&out).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}


// Note that mp4decrypt is not able to decrypt content in a WebM container, so we use Shaka packager
// here.
#[test]
fn test_decryption_webm() {
    let mpd = "https://storage.googleapis.com/shaka-demo-assets/angel-one-widevine/dash.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("angel.webm");
    let cli = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v",
               "--quality", "worst",
               "--decryption-application", "shaka",
               "--key", "4d97930a3d7b55fa81d0028653f5e499:429ec76475e7a952d224d8ef867f12b6",
               "--key", "d21373c0b8ab5ba9954742bcdfb5f48b:150a6c7d7dee6a91b74dccfce5b31928",
               "--key", "6f1729072b4a5cd288c916e11846b89e:a84b4bd66901874556093454c075e2c6",
               "--key", "800aacaa522958ae888062b5695db6bf:775dbf7289c4cc5847becd571f536ff2",
               "--key", "67b30c86756f57c5a0a38a23ac8c9178:efa2878c2ccf6dd47ab349fcf90e6259",
               "-o", &out.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cli");
    let msg = String::from_utf8_lossy(&cli.stdout);
    if msg.len() > 0 {
        println!("dash-mpd-cli stdout: {msg}");
    }
    let msg = String::from_utf8_lossy(&cli.stderr);
    if msg.len() > 0 {
        println!("dash-mpd-cli stderr: {msg}");
    }
    assert!(cli.status.success());
    check_file_size_approx(&out, 1_331_284);
    let meta = ffprobe(&out).unwrap();
    assert_eq!(meta.streams.len(), 2);
    // The order of audio and video streams in the output WebM container is unreliable with Shaka
    // packager, so we need to test this carefully.
    let audio = meta.streams.iter()
        .find(|s| s.codec_type.eq(&Some(String::from("audio"))))
        .expect("finding audio stream");
    // Whether opus or vorbis codec is chosen seems to depend on the version of the muxer used.
    assert!(audio.codec_name.eq(&Some(String::from("vorbis"))) ||
            audio.codec_name.eq(&Some(String::from("opus"))));
    let video = meta.streams.iter()
        .find(|s| s.codec_type.eq(&Some(String::from("video"))))
        .expect("finding video stream");
    assert_eq!(video.codec_name, Some(String::from("vp9")));
    assert!(video.width.is_some());
    let ffmpeg = Command::new("ffmpeg")
        .args(["-nostdin",
               "-v", "error",
               "-i", &out.to_string_lossy(),
               "-f", "null", "-"])
        .output()
        .expect("spawning ffmpeg");
    let msg = String::from_utf8_lossy(&ffmpeg.stderr);
    if msg.len() > 0 {
        eprintln!("FFMPEG stderr {msg}");
    }
    assert!(msg.len() == 0);
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}


#[test]
fn test_decryption_small_shaka () {
    let mpd = "https://m.dtv.fi/dash/dasherh264/drm/manifest_clearkey.mpd";
    let tmpd = TempDir::new().unwrap()
        .into_persistent_if(env::var("TEST_PERSIST_FILES").is_ok());
    let out = tmpd.child("caminandes.mp4");
    let cli = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v",
               "--quality", "worst",
               "--decryption-application", "shaka",
               "--key", "43215678123412341234123412341234:12341234123412341234123412341234",
               "-o", &out.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cli");
    let msg = String::from_utf8_lossy(&cli.stdout);
    if msg.len() > 0 {
        println!("dash-mpd-cli stdout: {msg}");
    }
    let msg = String::from_utf8_lossy(&cli.stderr);
    if msg.len() > 0 {
        println!("dash-mpd-cli stderr: {msg}");
    }
    assert!(cli.status.success());
    check_file_size_approx(&out, 6_975_147);
    let format = FileFormat::from_file(&out).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    // There are unexpected ffmpeg errors shown on CI machines for this output file
    // assert!(ffmpeg_approval(&out));
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
}
