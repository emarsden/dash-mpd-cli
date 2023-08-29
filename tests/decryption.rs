/// Tests for decryption support using mp4decrypt
///
/// These test cases are from https://refapp.hbbtv.org/videos/. We don't run these tests on CI
/// infrastructure, because they consume non-negligeable network bandwidth.

// To run tests while enabling printing to stdout/stderr
//
//    cargo test --test decryption -- --show-output


use fs_err as fs;
use std::env;
use std::process::Command;
use std::path::PathBuf;


// We tolerate significant differences in final output file size, because as encoder performance
// changes in newer versions of ffmpeg, the resulting file size when reencoding may change
// significantly.
fn check_file_size_approx(p: &PathBuf, expected: u64) {
    let meta = fs::metadata(p).unwrap();
    let ratio = meta.len() as f64 / expected as f64;
    assert!(0.9 < ratio && ratio < 1.1);
}

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



// A small decryption test case that we can run on the CI infrastructure.
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
