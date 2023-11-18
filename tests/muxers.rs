/// Tests for selecting muxer application (--muxer-preference commandline argument).
///

// To run tests while enabling printing to stdout/stderr
//
//    cargo test --test muxers -- --show-output


pub mod common;
use std::env;
use std::process::Command;
use std::path::Path;
use ffprobe::ffprobe;
use file_format::FileFormat;
use common::check_file_size_approx;


// Try to obtain the encoder field in the metadata with the ffprobe crate (works with certain MP4
// files), or using the mediainfo commandline tool if it is installed (works for AVI files where
// ffprobe fails).
fn container_metadata_encoder(p: &Path) -> Option<String> {
    if let Ok(meta) = ffprobe(p) {
        if let Some(tags) = meta.format.tags {
            if let Some(enc) = tags.encoder {
                return Some(enc);
            }
        }
    }
    if let Ok(minf) = Command::new("mediainfo")
        .args(["--Output=JSON", &p.to_string_lossy()])
        .output()
    {
        let json = String::from_utf8_lossy(&minf.stdout);
        if let Ok(j) = json::parse(&json) {
            if let Some(ea) = j["media"]["track"][0]["Encoded_Application"].as_str() {
                return Some(String::from(ea));
            }
        }
    }
    None
}


#[test]
fn test_muxers_mp4 () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://m.dtv.fi/dash/dasherh264/manifest.mpd";
    let outpath = env::temp_dir().join("caminandes-ffmpeg.mp4");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "-v",
               "--muxer-preference", "mp4:ffmpeg",
               "--muxer-preference", "avi:vlc",
               "--quality", "worst",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 6_975_147);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    // Check that the mp4 metadata indicates it was created with ffmpeg (libavformat which contains
    // the muxing support in ffmpeg).
    if let Some(enc) = container_metadata_encoder(&outpath) {
        assert!(enc.starts_with("Lavf"), "Unexpected encoder {enc} in mp4 metadata");
    }

    let outpath = env::temp_dir().join("caminandes-vlc.mp4");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--muxer-preference", "mp4:vlc",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 6_975_147);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    if let Some(enc) = container_metadata_encoder(&outpath) {
        assert!(enc.starts_with("vlc"), "Unexpected encoder {enc} in mp4 metadata");
    }

    let outpath = env::temp_dir().join("caminandes-mp4box.mp4");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--muxer-preference", "mp4:mp4box",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 6_975_147);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    if let Some(enc) = container_metadata_encoder(&outpath) {
        // This won't be reached because MP4Box (as of v2.2.1) does not include a Metadata box in
        // the MP4 container it creates.
        assert!(enc.starts_with("mp4box"), "Unexpected encoder {enc} in mp4 metadata");
    }
}


#[test]
fn test_muxers_mkv () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://m.dtv.fi/dash/dasherh264/manifest.mpd";
    let outpath = env::temp_dir().join("caminandes-ffmpeg.mkv");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--muxer-preference", "mkv:ffmpeg",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 6_975_147);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::MatroskaVideo);
    if let Some(enc) = container_metadata_encoder(&outpath) {
        assert!(enc.starts_with("Lavf"), "Unexpected encoder {enc} in mkv metadata");
    }

    let outpath = env::temp_dir().join("caminandes-vlc.mkv");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--muxer-preference", "mkv:vlc",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 6_975_147);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::MatroskaVideo);
    if let Some(enc) = container_metadata_encoder(&outpath) {
        assert!(enc.to_lowercase().starts_with("vlc"), "Unexpected encoder {enc} in mkv metadata");
    }

    let outpath = env::temp_dir().join("caminandes-mkvmerge.mkv");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--muxer-preference", "mkv:mkvmerge",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 6_975_147);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::MatroskaVideo);
    if let Some(enc) = container_metadata_encoder(&outpath) {
        assert!(enc.contains("libmatroska"), "Unexpected encoder {enc} in mkv metadata");
    }
}


#[test]
fn test_muxers_avi () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://m.dtv.fi/dash/dasherh264/manifest.mpd";
    let outpath = env::temp_dir().join("caminandes-ffmpeg.avi");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--muxer-preference", "avi:ffmpeg",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 7_128_748);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::AudioVideoInterleave);
    if let Some(enc) = container_metadata_encoder(&outpath) {
        assert!(enc.starts_with("Lavf"), "Unexpected encoder {enc} in avi metadata");
    }

    let outpath = env::temp_dir().join("caminandes-vlc.avi");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--muxer-preference", "avi:vlc",
               "-o", &outpath.to_string_lossy(), mpd])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&outpath, 5_520_360);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::AudioVideoInterleave);
    if let Some(enc) = container_metadata_encoder(&outpath) {
        assert!(enc.to_lowercase().starts_with("vlc"), "Unexpected encoder {enc} in avi metadata");
    }

    // mkvmerge and MP4Box can't create AVI containers.
}

