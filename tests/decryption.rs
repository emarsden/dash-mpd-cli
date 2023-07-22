/// Tests for decryption support using mp4decrypt
///
/// These test cases are from https://refapp.hbbtv.org/videos/. We don't run these tests on CI
/// infrastructure, because they consume non-negligeable network bandwidth.

// To run tests while enabling printing to stdout/stderr, "cargo test -- --show-output" (from the
// root crate directory).


use std::fs;
use std::env;
use std::process::Command;
use std::path::Path;


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
    if let Ok(meta) = fs::metadata(Path::new(&outpath)) {
        let ratio = meta.len() as f64 / 33_746_341.0;
        assert!(0.95 < ratio && ratio < 1.05);
    }
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
    if let Ok(meta) = fs::metadata(Path::new(&outpath)) {
        let ratio = meta.len() as f64 / 79_731_116.0;
        assert!(0.95 < ratio && ratio < 1.05);
    }
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
    if let Ok(meta) = fs::metadata(Path::new(&outpath)) {
        let ratio = meta.len() as f64 / 26_420_624.0;
        assert!(0.95 < ratio && ratio < 1.05);
    }
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
    if let Ok(meta) = fs::metadata(Path::new(&outpath)) {
        let ratio = meta.len() as f64 / 14_357_917.0;
        assert!(0.95 < ratio && ratio < 1.05);
    }
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
    if let Ok(meta) = fs::metadata(Path::new(&outpath)) {
        let ratio = meta.len() as f64 / 14_357_925.0;
        assert!(0.95 < ratio && ratio < 1.05);
    }
}

