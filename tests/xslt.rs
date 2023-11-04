// Dedicated tests for XSLT stylesheet processing
//
// To run only these tests while enabling printing to stdout/stderr
//
//    cargo test --test xslt -- --show-output


pub mod common;
use fs_err as fs;
use std::env;
use std::time::Duration;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use axum::{routing::get, Router};
use axum::extract::State;
use axum::response::{Response, IntoResponse};
use axum::http::{header, StatusCode};
use axum::body::{Full, Bytes};
use ffprobe::ffprobe;
use file_format::FileFormat;
use anyhow::{Context, Result};
use env_logger::Env;
use common::{check_file_size_approx, generate_minimal_mp4};


#[derive(Debug, Default)]
struct AppState {
    count_init: AtomicUsize,
    count_media: AtomicUsize,
}

impl AppState {
    fn new() -> AppState {
        AppState {
            count_init: AtomicUsize::new(0),
            count_media: AtomicUsize::new(0),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_xslt_rewrite_media() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info,reqwest=warn")).init();

    // State shared between the request handlers.
    let shared_state = Arc::new(AppState::new());


    async fn send_init(State(state): State<Arc<AppState>>) -> Response<Full<Bytes>> {
        state.count_init.fetch_add(1, Ordering::SeqCst);
        let mp4 = generate_minimal_mp4();
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "video/mp4")
            .body(Full::from(mp4))
            .unwrap()
    }

    async fn send_media(State(state): State<Arc<AppState>>) -> Response<Full<Bytes>> {
        state.count_media.fetch_add(1, Ordering::SeqCst);
        let mp4 = generate_minimal_mp4();
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "video/mp4")
            .body(Full::from(mp4))
            .unwrap()
    }
    async fn send_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        ([(header::CONTENT_TYPE, "text/plain")],
         format!("{} {}",
                 state.count_init.load(Ordering::Relaxed),
                 state.count_media.load(Ordering::Relaxed)))
    }

    let mut mpd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    mpd.push("tests");
    mpd.push("fixtures");
    mpd.push("jurassic-compact-5975");
    mpd.set_extension("mpd");
    let xml = fs::read_to_string(mpd).unwrap();
    let app = Router::new()
        .route("/mpd", get(
            || async { ([(header::CONTENT_TYPE, "application/dash+xml")], xml) }))
        .route("/media/init.mp4", get(send_init))
        .route("/media/segment-:id.mp4", get(send_media))
        .route("/status", get(send_status))
        .with_state(shared_state);
    let server_handle = axum_server::Handle::new();
    let backend_handle = server_handle.clone();
    let backend = async move {
        axum_server::bind("127.0.0.1:6668".parse().unwrap())
            .handle(backend_handle)
            .serve(app.into_make_service()).await
            .unwrap()
    };
    tokio::spawn(backend);
    tokio::time::sleep(Duration::from_millis(1000)).await;
    // Check that the initial value of our request counter is zero.
    let client = reqwest::Client::builder()
        .timeout(Duration::new(10, 0))
        .build()
        .context("creating HTTP client")?;
    let txt = client.get("http://localhost:6668/status")
        .send().await?
        .error_for_status()?
        .text().await
        .context("fetching status")?;
    assert!(txt.eq("0 0"));

    let mpd_url = "http://localhost:6668/mpd";
    let mut xslt = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xslt.push("tests");
    xslt.push("fixtures");
    xslt.push("rewrite-init-media-segments");
    xslt.set_extension("xslt");
    let v = env::temp_dir().join("xslt_video.mp4");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--xslt-stylesheet", &xslt.to_string_lossy(),
               "-o", &v.to_string_lossy(), mpd_url])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    // Check the total number of requested media segments corresponds to what we expect.
    let txt = client.get("http://localhost:6668/status")
        .send().await?
        .error_for_status()?
        .text().await
        .context("fetching status")?;
    assert!(txt.eq("1 926"), "Expecting 1 926, got {txt}");
    server_handle.shutdown();

    Ok(())
}



// This MPD manifest includes two AdaptationSets, one for the video streams and one for the audio
// stream. The rewrite-drop-audio.xslt stylesheet rewrites the XML manifest to remove the audio
// AdaptationSet. We check that the resulting media container only contains a video track.
#[test]
fn test_xslt_drop_audio() {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd_url = "http://dash.edgesuite.net/envivio/dashpr/clear/Manifest.mpd";
    let out = env::temp_dir().join("envivio-dropped-audio.mp4");
    let mut xslt = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xslt.push("tests");
    xslt.push("fixtures");
    xslt.push("rewrite-drop-audio");
    xslt.set_extension("xslt");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "-v",
               "--xslt-stylesheet", &xslt.to_string_lossy(),
               "-o", &out.to_string_lossy(), mpd_url])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&out, 11_005_923);
    let format = FileFormat::from_file(out.clone()).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    let meta = ffprobe(out.clone()).unwrap();
    assert_eq!(meta.streams.len(), 1);
    let video = &meta.streams[0];
    assert_eq!(video.codec_type, Some(String::from("video")));
    assert_eq!(video.codec_name, Some(String::from("h264")));
    assert_eq!(video.width, Some(320));
}


// This XSLT stylesheet replaces @media and @initialization attributes to point to a beloved media
// segment.
#[test]
fn test_xslt_rick() {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd_url = "https://dash.akamaized.net/akamai/test/index3-original.mpd";
    let out = env::temp_dir().join("ricked.mp4");
    let mut xslt = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xslt.push("tests");
    xslt.push("fixtures");
    xslt.push("rewrite-rickroll");
    xslt.set_extension("xslt");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "-v",
               "--xslt-stylesheet", &xslt.to_string_lossy(),
               "-o", &out.to_string_lossy(), mpd_url])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&out, 538_262_020);
    let format = FileFormat::from_file(out.clone()).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    let meta = ffprobe(out.clone()).unwrap();
    assert_eq!(meta.streams.len(), 1);
    let video = &meta.streams[0];
    assert_eq!(video.codec_type, Some(String::from("video")));
    assert_eq!(video.codec_name, Some(String::from("h264")));
    assert_eq!(video.width, Some(320));
}


#[test]
fn test_xslt_multiple_stylesheets() {
    if env::var("CI").is_ok() {
        return;
    }
    let mpd_url = "http://dash.edgesuite.net/envivio/dashpr/clear/Manifest.mpd";
    let out = env::temp_dir().join("ricked-cleaned.mp4");
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
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "-v",
               "--xslt-stylesheet", &xslt_rick.to_string_lossy(),
               "--xslt-stylesheet", &xslt_clean.to_string_lossy(),
               "-o", &out.to_string_lossy(), mpd_url])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    check_file_size_approx(&out, 12_975_377);
    let format = FileFormat::from_file(out.clone()).unwrap();
    assert_eq!(format, FileFormat::Mpeg4Part14Video);
    let meta = ffprobe(out.clone()).unwrap();
    assert_eq!(meta.streams.len(), 2);
    let video = &meta.streams[0];
    assert_eq!(video.codec_type, Some(String::from("video")));
    assert_eq!(video.codec_name, Some(String::from("h264")));
    assert_eq!(video.width, Some(320));
}


// Note that the error message is structured differently on Unix and Microsoft Windows platforms
// ("exit code" vs "exit status").
#[test]
fn test_xslt_stylesheet_error() {
    let mpd_url = "https://dash.akamaized.net/akamai/test/index3-original.mpd";
    let out = env::temp_dir().join("unexist.mp4");
    let mut xslt = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xslt.push("tests");
    xslt.push("fixtures");
    xslt.push("rewrite-stylesheet-error");
    xslt.set_extension("xslt");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "-v",
               "--xslt-stylesheet", &xslt.to_string_lossy(),
               "-o", &out.to_string_lossy(), mpd_url])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    let stderr = String::from_utf8_lossy(&cli.stderr);
    println!("STDERR> {stderr}");
    assert!(stderr.contains("xsltproc returned exit"));
}


// Note that the error message is structured differently on Unix and Microsoft Windows platforms.
#[test]
fn test_xslt_stylesheet_missing() {
    let mpd_url = "https://dash.akamaized.net/akamai/test/index3-original.mpd";
    let out = env::temp_dir().join("unexist.mp4");
    let xslt = env::temp_dir().join("missing.xslt");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "-v",
               "--xslt-stylesheet", &xslt.to_string_lossy(),
               "-o", &out.to_string_lossy(), mpd_url])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    // We would like to access the return code from running our bin, but "cargo run" doesn't return that.
    let stderr = String::from_utf8_lossy(&cli.stderr);
    assert!(stderr.contains("failed to load external entity"));
}

