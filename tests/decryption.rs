/// Tests for decryption support using mp4decrypt
///
/// These test cases are from https://refapp.hbbtv.org/videos/. We don't run these tests on CI
/// infrastructure, because they consume non-negligeable network bandwidth.

// To run tests while enabling printing to stdout/stderr
//
//    cargo test --test decryption -- --show-output


pub mod common;
use std::env;
use std::process::Command;
use ffprobe::ffprobe;
use file_format::FileFormat;
use common::check_file_size_approx;


#[test]
fn test_decryption_widevine_cenc () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://refapp.hbbtv.org/videos/spring_h265_v8/cenc/manifest_wvcenc.mpd";
    let outpath = env::temp_dir().join("spring.mp4");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "-v",
               "--quality", "worst",
               "--key", "43215678123412341234123412341237:12341234123412341234123412341237",
               "--key", "43215678123412341234123412341236:12341234123412341234123412341236",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 33_746_341);
}

#[test]
fn test_decryption_widevine_cbcs () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://refapp.hbbtv.org/videos/tears_of_steel_h265_v8/cbcs/manifest_wvcenc.mpd";
    let outpath = env::temp_dir().join("tears-steel.mp4");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "-v",
               "--quality", "worst",
               "--key", "43215678123412341234123412341237:12341234123412341234123412341237",
               "--key", "43215678123412341234123412341236:12341234123412341234123412341236",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 79_731_116);
}


#[test]
fn test_decryption_playready_cenc () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://refapp.hbbtv.org/videos/00_llama_h264_v8_8s/cenc/manifest_prcenc.mpd";
    let outpath = env::temp_dir().join("llama.mp4");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "-v",
               "--quality", "worst",
               "--key", "43215678123412341234123412341236:12341234123412341234123412341236",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 26_420_624);
}

#[test]
fn test_decryption_marlin_cenc () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://refapp.hbbtv.org/videos/agent327_h264_v8/cenc/manifest_mlcenc.mpd";
    let outpath = env::temp_dir().join("llama.mp4");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "-v",
               "--quality", "worst",
               "--key", "43215678123412341234123412341234:12341234123412341234123412341234",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 14_357_917);
}

#[test]
fn test_decryption_marlin_cbcs () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://refapp.hbbtv.org/videos/agent327_h264_v8/cbcs/manifest_mlcenc.mpd";
    let outpath = env::temp_dir().join("llama.mp4");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "-v",
               "--quality", "worst",
               "--key", "43215678123412341234123412341234:12341234123412341234123412341234",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 14_357_925);
}


#[test]
fn test_decryption_cmaf_h265_multikey () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://media.axprod.net/TestVectors/H265/protected_cmaf_1080p_h265_multikey/manifest.mpd";
    let outpath = env::temp_dir().join("axinom-h264-multikey.mp4");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "-v",
               "--quality", "worst",
               "--key", "53dc3eaa5164410a8f4ee15113b43040:620045a34e839061ee2e9b7798fdf89b",
               "--key", "9dbace9e41034c5296aa63227dc5f773:a776f83276a107a3c322f9dbd6d4f48c",
               "--key", "a76f0ca68e7d40d08a37906f3e24dde2:2a99b42f08005ab4b57af20f4da3cc05",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 48_233_447);
}


// Small decryption test cases that we can run on the CI infrastructure.
#[test]
fn test_decryption_cenc_kaltura () {
    let mpd = "https://cdnapisec.kaltura.com/p/2433871/sp/243387100/playManifest/protocol/https//entryId/1_pgssezc1/format/mpegdash/tags/mbr/f/a.mpd";
    let outpath = env::temp_dir().join("kaltura.mp4");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--mp4decrypt-location", "mp4decrypt",
               "--key", "a07c5d499dcead0fb416fed5913967be:caee457911302478487e6680bf0b3d1b",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 1_323_079);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    let meta = ffprobe(outpath.clone()).unwrap();
    assert_eq!(meta.streams.len(), 2);
    let audio = &meta.streams[1];
    assert_eq!(audio.codec_type, Some(String::from("audio")));
    assert_eq!(audio.codec_name, Some(String::from("aac")));
    assert!(audio.width.is_none());
    let tags = audio.tags.as_ref().unwrap();
    assert_eq!(tags.language, Some(String::from("eng")));
}


#[test]
fn test_decryption_small () {
    let mpd = "https://m.dtv.fi/dash/dasherh264/drm/manifest_clearkey.mpd";
    let outpath = env::temp_dir().join("caminandes.mp4");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "-v",
               "--quality", "worst",
               "--key", "43215678123412341234123412341234:12341234123412341234123412341234",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    let msg = String::from_utf8_lossy(&cli.stdout);
    if msg.len() > 0 {
        println!("dash-mpd-cli stdout: {msg}");
    }
    let msg = String::from_utf8_lossy(&cli.stderr);
    if msg.len() > 0 {
        println!("dash-mpd-cli stderr: {msg}");
    }
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 6_975_147);
}
