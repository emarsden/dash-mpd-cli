/// Tests for functionality when using our docker/podman container
///
///
// To run tests while enabling printing to stdout/stderr
//
//    cargo test --test container -- --show-output
//
// These tests are using the dash-mpd-cli container on the Github Container Registry to download
// files. We create a directory inside the host's TMPDIR and map it to /content in the container, so for
// example when we are using "-o foo.mp4" this results in the output file being saved as
// /tmp/.tmp4BFre7/foo.mp4.
//
// Some of the the GitHub MacOS runners (M1-based processor) do not support nested virtualization,
// so are not able to run Docker/Podman (they are already running in a VM). For this reason, the
// tests that are not already disabled on the CI infrastructure are explicitly disabled when
// targeting MacOS.


#[macro_use]
extern crate lazy_static;

pub mod common;
use fs_err as fs;
use std::env;
use std::process::Command;
use std::path::{Path, PathBuf};
use ffprobe::ffprobe;
use file_format::FileFormat;
use test_log::test;
use common::check_file_size_approx;


lazy_static! {
    // A directory inside $TMPDIR that is persisted.
    static ref TMP: PathBuf = tempfile::TempDir::new().unwrap().into_path();
}


fn container_run(args: Vec<&str>) {
    let vspec = format!("{}:/content", TMP.display());
    let mut cargs = vec!["run", "--rm",
                         "--pull=newer",
                         "--volume", &vspec,
                         "ghcr.io/emarsden/dash-mpd-cli"];
    for a in &args {
        cargs.push(a);
    }
    println!("CTR> {:?}", cargs);
    let mut docker_exe = String::from("podman");
    if let Ok(docker) = env::var("DOCKER") {
        docker_exe = docker;
    }
    let cli = Command::new(docker_exe)
        .args(cargs)
        .output()
        .expect("failed spawning podman");
    if !cli.status.success() {
        let stdout = String::from_utf8_lossy(&cli.stdout);
        if stdout.len() > 0 {
            println!("Podman stdout> {stdout}");
        }
        let stderr = String::from_utf8_lossy(&cli.stderr);
        if stderr.len() > 0 {
            println!("Podman stderr> {stderr}");
        }
    }
    assert!(cli.status.success());
}

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
#[cfg(not(target_os = "macos"))]
fn test_container_mp4 () {
    let mpd = "https://cloudflarestream.com/31c9291ab41fac05471db4e73aa11717/manifest/video.mpd";
    let out = Path::new("cf.mp4");
    let outpath = TMP.join(out);
    container_run(vec!["-o", &out.to_string_lossy(), "--quality", "worst", mpd]);
    check_file_size_approx(&outpath, 325_334);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    let _ = fs::remove_file(outpath);
}

#[test]
fn test_container_mp4a () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://dash.akamaized.net/dash264/TestCases/3a/fraunhofer/aac-lc_stereo_without_video/Sintel/sintel_audio_only_aaclc_stereo_sidx.mpd";
    let out = Path::new("sintel-audio.mp4");
    let outpath = TMP.join(out);
    if outpath.exists() {
        let _ = fs::remove_file(outpath.clone());
    }
    container_run(vec!["-o", &out.to_string_lossy(), "--quality", "worst", mpd]);
    check_file_size_approx(&outpath, 7_456_334);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Audio);
    let meta = ffprobe(outpath.clone()).unwrap();
    assert_eq!(meta.streams.len(), 1);
    let audio = &meta.streams[0];
    assert_eq!(audio.codec_type, Some(String::from("audio")));
    assert_eq!(audio.codec_name, Some(String::from("aac")));
    assert!(audio.width.is_none());
    let _ = fs::remove_file(outpath);
}


#[test]
fn test_container_audio_flac () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "http://rdmedia.bbc.co.uk/testcard/vod/manifests/radio-flac-en.mpd";
    let out = Path::new("bbcradio-flac.mp4");
    let outpath = TMP.join(out);
    if outpath.exists() {
        let _ = fs::remove_file(outpath.clone());
    }
    container_run(vec!["-o", &out.to_string_lossy(), "--quality", "worst", mpd]);
    check_file_size_approx(&outpath, 81_603_640);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Audio);
    let meta = ffprobe(outpath.clone()).unwrap();
    assert_eq!(meta.streams.len(), 1);
    let audio = &meta.streams[0];
    assert_eq!(audio.codec_type, Some(String::from("audio")));
    assert_eq!(audio.codec_name, Some(String::from("flac")));
    assert!(audio.width.is_none());
    let _ = fs::remove_file(outpath);
}


#[test]
fn test_container_subtitles_wvtt () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://storage.googleapis.com/shaka-demo-assets/sintel-mp4-wvtt/dash.mpd";
    let out = Path::new("sintel-wvtt.mp4");
    let outpath = TMP.join(out);
    if outpath.exists() {
        let _ = fs::remove_file(outpath.clone());
    }
    let mut subpath = outpath.clone();
    subpath.set_extension("srt");
    let subpath = Path::new(&subpath);
    container_run(vec!["-v", "-v",
                       "-o", &out.to_string_lossy(),
                       "--quality", "worst",
                       "--write-subs", mpd]);
    check_file_size_approx(&outpath, 25_950_458);
    assert!(fs::metadata(subpath).is_ok());
    let srt = fs::read_to_string(subpath).unwrap();
    // We didn't specify a preferred language, so the first available one in the manifest (here
    // Dutch) is downloaded.
    assert!(srt.contains("land van de poortwachters"));
    let _ = fs::remove_file(outpath.clone());

    container_run(vec!["-o", &out.to_string_lossy(),
                       "--quality", "worst",
                       "--write-subs", "--prefer-language", "eng", mpd]);
    let srt = fs::read_to_string(subpath).unwrap();
    // This time we requested English subtitles.
    assert!(srt.contains("land of the gatekeepers"));
    let _ = fs::remove_file(outpath);
}


#[test]
fn test_container_subtitles_ttml () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://dash.akamaized.net/dash264/TestCases/4b/qualcomm/2/TearsOfSteel_onDem5secSegSubTitles.mpd";
    let out = Path::new("tears-of-steel-ttml.mp4");
    let outpath = TMP.join(out);
    if outpath.exists() {
        let _ = fs::remove_file(outpath.clone());
    }
    let mut subpath = outpath.clone();
    subpath.set_extension("ttml");
    let subpath = Path::new(&subpath);
    container_run(vec!["-v",
                       "-o", &out.to_string_lossy(),
                       "--quality", "worst",
                       "--write-subs",
                       mpd]);
    check_file_size_approx(&outpath, 46_299_053);
    assert!(fs::metadata(subpath).is_ok());
    let ttml = fs::read_to_string(subpath).unwrap();
    // We didn't specify a preferred language, so the first available one in the manifest (here
    // English) is downloaded.
    assert!(ttml.contains("You're a jerk"));
    let _ = fs::remove_file(outpath.clone());
    let _ = fs::remove_file(subpath);

    container_run(vec!["-o", &out.to_string_lossy(),
                       "--quality", "worst",
                       "--write-subs",
                       "--prefer-language", "de", mpd]);
    let ttml = fs::read_to_string(subpath).unwrap();
    // This time we requested German subtitles.
    assert!(ttml.contains("Du bist ein Vollidiot"));
    let _ = fs::remove_file(outpath);
    let _ = fs::remove_file(subpath);
}


#[test]
fn test_container_subtitles_vtt () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "http://dash.edgesuite.net/akamai/test/caption_test/ElephantsDream/elephants_dream_480p_heaac5_1.mpd";
    let out = Path::new("elephants-dream.mp4");
    let outpath = TMP.join(out);
    if outpath.exists() {
        let _ = fs::remove_file(outpath.clone());
    }
    let mut subpath = outpath.clone();
    subpath.set_extension("vtt");
    let subpath = Path::new(&subpath);
    container_run(vec!["-o", &out.to_string_lossy(), "--quality", "worst", "--write-subs", mpd]);
    check_file_size_approx(&outpath, 128_768_482);
    assert!(fs::metadata(subpath).is_ok());
    // This manifest contains a single subtitle track, available in VTT format via BaseURL addressing.
    let vtt = fs::read_to_string(subpath).unwrap();
    assert!(vtt.contains("Hurry Emo!"));
    let _ = fs::remove_file(outpath);
}

#[test]
fn test_container_decryption_playready_cenc () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://refapp.hbbtv.org/videos/00_llama_h264_v8_8s/cenc/manifest_prcenc.mpd";
    let out = Path::new("llama.mp4");
    let outpath = TMP.join(out);
    if outpath.exists() {
        let _ = fs::remove_file(outpath.clone());
    }
    container_run(vec!["-v", "-o", &out.to_string_lossy(),
                       "--quality", "worst",
                       "--key", "43215678123412341234123412341236:12341234123412341234123412341236",
                       mpd]);
    check_file_size_approx(&outpath, 26_420_624);
    let _ = fs::remove_file(outpath);
}

#[test]
fn test_container_decryption_marlin_cenc () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://refapp.hbbtv.org/videos/agent327_h264_v8/cenc/manifest_mlcenc.mpd";
    let out = Path::new("llama-cenc.mp4");
    let outpath = TMP.join(out);
    if outpath.exists() {
        let _ = fs::remove_file(outpath.clone());
    }
    container_run(vec!["-v", "-o", &out.to_string_lossy(),
                       "--quality", "worst",
                       "--key", "43215678123412341234123412341234:12341234123412341234123412341234",
                       mpd]);
    check_file_size_approx(&outpath, 14_357_917);
    let _ = fs::remove_file(outpath);
}


// Note that mp4decrypt is not able to decrypt content in a WebM container, so we use Shaka packager
// here.
#[test]
#[cfg(not(target_os = "macos"))]
fn test_container_decryption_webm() {
    let mpd = "https://storage.googleapis.com/shaka-demo-assets/angel-one-widevine/dash.mpd";
    let out = Path::new("angel.webm");
    let outpath = TMP.join(out);
    if outpath.exists() {
        let _ = fs::remove_file(outpath.clone());
    }
    container_run(vec!["-v", "-o", &out.to_string_lossy(),
                       "--quality", "worst",
                       "--decryption-application", "shaka",
                       "--key", "4d97930a3d7b55fa81d0028653f5e499:429ec76475e7a952d224d8ef867f12b6",
                       "--key", "d21373c0b8ab5ba9954742bcdfb5f48b:150a6c7d7dee6a91b74dccfce5b31928",
                       "--key", "6f1729072b4a5cd288c916e11846b89e:a84b4bd66901874556093454c075e2c6",
                       "--key", "800aacaa522958ae888062b5695db6bf:775dbf7289c4cc5847becd571f536ff2",
                       "--key", "67b30c86756f57c5a0a38a23ac8c9178:efa2878c2ccf6dd47ab349fcf90e6259",
                       mpd]);
    check_file_size_approx(&outpath, 1_331_284);
    let meta = ffprobe(outpath.clone()).unwrap();
    assert_eq!(meta.streams.len(), 2);
    // The order of audio and video streams in the output WebM container is unreliable with Shaka
    // packager, so we need to test this carefully.
    let audio = meta.streams.iter()
        .find(|s| s.codec_type.eq(&Some(String::from("audio"))))
        .expect("finding audio stream");
    assert_eq!(audio.codec_name, Some(String::from("opus")));
    let video = meta.streams.iter()
        .find(|s| s.codec_type.eq(&Some(String::from("video"))))
        .expect("finding video stream");
    assert_eq!(video.codec_name, Some(String::from("vp9")));
    assert!(video.width.is_some());
    let ffmpeg = Command::new("ffmpeg")
        .args(["-nostdin",
               "-v", "error",
               "-i", &outpath.to_string_lossy(),
               "-f", "null", "-"])
        .output()
        .expect("spawning ffmpeg");
    let msg = String::from_utf8_lossy(&ffmpeg.stderr);
    if msg.len() > 0 {
        eprintln!("FFMPEG stderr {msg}");
    }
    assert!(msg.len() == 0);
    let _ = fs::remove_file(outpath);
}


#[test]
#[cfg(not(target_os = "macos"))]
fn test_container_decryption_small_shaka () {
    let mpd = "https://m.dtv.fi/dash/dasherh264/drm/manifest_clearkey.mpd";
    let out = Path::new("caminandes.mp4");
    let outpath = TMP.join(out);
    if outpath.exists() {
        let _ = fs::remove_file(outpath.clone());
    }
    container_run(vec!["-v", "-o", &out.to_string_lossy(),
                       "--quality", "worst",
                       "--decryption-application", "shaka",
                       "--key", "43215678123412341234123412341234:12341234123412341234123412341234",
                       mpd]);
    check_file_size_approx(&outpath, 6_975_147);
    // There are unexpected ffmpeg errors shown on CI machines for this output file
    // assert!(ffmpeg_approval(&outpath));
    let _ = fs::remove_file(outpath);
}

#[test]
fn test_container_muxers_mkv () {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://m.dtv.fi/dash/dasherh264/manifest.mpd";
    let out = Path::new("caminandes-ffmpeg.mkv");
    let outpath = TMP.join(out);
    if outpath.exists() {
        let _ = fs::remove_file(outpath.clone());
    }
    container_run(vec!["-v", "-o", &out.to_string_lossy(),
                       "--quality", "worst",
                       "--muxer-preference", "mkv:ffmpeg",
                       mpd]);
    check_file_size_approx(&outpath, 6_975_147);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::MatroskaVideo);
    if let Some(enc) = container_metadata_encoder(&outpath) {
        assert!(enc.starts_with("Lavf"), "Unexpected encoder {enc} in mkv metadata");
    }
    let _ = fs::remove_file(outpath);

    let out = Path::new("caminandes-mkvmerge.mkv");
    let outpath = TMP.join(out);
    container_run(vec!["-v", "-o", &out.to_string_lossy(),
                       "--quality", "worst",
                       "--muxer-preference", "mkv:mkvmerge",
                       mpd]);
    check_file_size_approx(&outpath, 6_975_147);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::MatroskaVideo);
    if let Some(enc) = container_metadata_encoder(&outpath) {
        assert!(enc.contains("libmatroska"), "Unexpected encoder {enc} in mkv metadata");
    }
    let _ = fs::remove_file(outpath);
}


#[test]
fn test_container_xslt_multiple_stylesheets() {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "http://dash.edgesuite.net/envivio/dashpr/clear/Manifest.mpd";
    let out = Path::new("ricked-cleaned.mp4");
    let outpath = TMP.join(out);
    if outpath.exists() {
        let _ = fs::remove_file(outpath.clone());
    }
    let mut xslt_rick = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xslt_rick.push("tests");
    xslt_rick.push("fixtures");
    xslt_rick.push("rewrite-rickroll");
    xslt_rick.set_extension("xslt");
    let mut xslt_clean = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xslt_clean.push("tests");
    xslt_clean.push("fixtures");
    xslt_clean.push("rewrite-drop-dai");
    xslt_clean.set_extension("xslt");
    // For xsltproc running in the container to access the stylesheet files, we copy them to the
    // /tmp/ directory, which is made available inside the container.
    let s1 = TMP.join("s1.xslt");
    let s2 = TMP.join("s2.xslt");
    fs::copy(xslt_rick, s1.clone()).unwrap();
    fs::copy(xslt_clean, s2.clone()).unwrap();
    container_run(vec!["-v", "-o", &out.to_string_lossy(),
                       "--quality", "worst",
                       "--xslt-stylesheet", "s1.xslt",
                       "--xslt-stylesheet", "s2.xslt",
                       mpd]);
    check_file_size_approx(&outpath, 12_975_377);
    let format = FileFormat::from_file(outpath.clone()).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    let meta = ffprobe(outpath.clone()).unwrap();
    assert_eq!(meta.streams.len(), 2);
    let video = &meta.streams[0];
    assert_eq!(video.codec_type, Some(String::from("video")));
    assert_eq!(video.codec_name, Some(String::from("h264")));
    assert_eq!(video.width, Some(320));
    let _ = fs::remove_file(outpath);
}
