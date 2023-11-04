// Testing support for Basic HTTP authorization.
//
//
// To run tests while enabling printing to stdout/stderr
//
//    RUST_LOG=info cargo test --test basic_auth -- --show-output
//
// What happens in this test:
//
//   - Start an axum HTTP server that serves the manifest and our media segments. The server is
//   configured to require HTTP Basic authorization.
//
//   - Fetch the associated media content using dash-mpd-cli via "cargo run", and check that each of
//   the expected remote elements is retrieved.


pub mod common;
use fs_err as fs;
use std::env;
use std::time::Duration;
use std::process::Command;
use std::io::Cursor;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use axum::{routing::get, Router};
use axum::extract::State;
use axum::response::{Response, IntoResponse};
use axum::http::{header, StatusCode};
use axum::body::{Full, Bytes};
use axum_auth::AuthBasic;
use dash_mpd::{MPD, Period, AdaptationSet, Representation, SegmentTemplate};
use anyhow::{Context, Result};
use env_logger::Env;
use log::info;
use common::generate_minimal_mp4;


#[derive(Debug, Default)]
struct AppState {
    counter: AtomicUsize,
}

impl AppState {
    fn new() -> AppState {
        AppState { counter: AtomicUsize::new(0) }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_basic_auth() -> Result<()> {
    // State shared between the request handlers. We are simply maintaining a counter of the number
    // of requests for media segments made.
    let shared_state = Arc::new(AppState::new());

    async fn send_mpd(AuthBasic((id, password)): AuthBasic) -> impl IntoResponse {
        info!("mpd request: auth {id:?}:{password:?}");
        let segment_template = SegmentTemplate {
            initialization: Some("/media/f1.mp4".to_string()),
            ..Default::default()
        };
        let rep = Representation {
            id: Some("1".to_string()),
            mimeType: Some("video/mp4".to_string()),
            codecs: Some("avc1.640028".to_string()),
            width: Some(1920),
            height: Some(800),
            bandwidth: Some(1980081),
            SegmentTemplate: Some(segment_template),
            ..Default::default()
        };
        let adapt = AdaptationSet {
            id: Some("1".to_string()),
            contentType: Some("video".to_string()),
            representations: vec!(rep),
            ..Default::default()
        };
        let period = Period {
            id: Some("1".to_string()),
            duration: Some(Duration::new(5, 0)),
            adaptations: vec!(adapt.clone()),
            ..Default::default()
        };
        let mpd = MPD {
            mpdtype: Some("static".to_string()),
            periods: vec!(period),
            ..Default::default()
        };
        let xml = quick_xml::se::to_string(&mpd).unwrap();
        ([(header::CONTENT_TYPE, "application/dash+xml")], xml)
    }

    // Create a minimal sufficiently-valid MP4 file.
    async fn send_mp4(AuthBasic((id, password)): AuthBasic, State(state): State<Arc<AppState>>) -> Response<Full<Bytes>> {
        info!("segment request: auth {id:?}:{password:?}");
        state.counter.fetch_add(1, Ordering::SeqCst);
        /*
        let config = mp4::Mp4Config {
            major_brand: str::parse("isom").unwrap(),
            minor_version: 512,
            compatible_brands: vec![
                str::parse("isom").unwrap(),
                str::parse("iso2").unwrap(),
                str::parse("avc1").unwrap(),
                str::parse("mp41").unwrap(),
            ],
            timescale: 1000,
        };
        let data = Cursor::new(Vec::<u8>::new());
        let mut writer = mp4::Mp4Writer::write_start(data, &config).unwrap();
        let media_conf = mp4::MediaConfig::AvcConfig(mp4::AvcConfig {
            width: 10,
            height: 10,
            // from https://github.com/ISSOtm/gb-packing-visualizer/blob/1954066537b373f2ddcd5768131bdb5595734a85/src/render.rs#L260
            seq_param_set: vec![
                0, // ???
                0, // avc_profile_indication
                0, // profile_compatibility
                0, // avc_level_indication
            ],
            pic_param_set: vec![],
        });
        let track_conf = mp4::TrackConfig {
            track_type: mp4::TrackType::Video,
            timescale: 1000,
            language: "und".to_string(),
            media_conf,
        };
        writer.add_track(&track_conf).unwrap();
        let sample = mp4::Mp4Sample {
            start_time: 0,
            duration: 2,
            rendering_offset: 0,
            is_sync: true,
            bytes: mp4::Bytes::from(vec![0x0u8; 751]),
        };
        writer.write_sample(1, &sample).unwrap();
        writer.write_end().unwrap();
        let data: Vec<u8> = writer.into_writer().into_inner();
        */
        let data = generate_minimal_mp4();
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "video/mp4")
            .body(Full::from(data))
            .unwrap()
    }

    // Status requests don't require authentication.
    async fn send_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        ([(header::CONTENT_TYPE, "text/plain")], format!("{}", state.counter.load(Ordering::Relaxed)))
    }

    env_logger::Builder::from_env(Env::default().default_filter_or("info,reqwest=warn")).init();
    let app = Router::new()
        .route("/mpd", get(send_mpd))
        .route("/media/:seg", get(send_mp4))
        .route("/status", get(send_status))
        .with_state(shared_state);
    let backend = async move {
        axum::Server::bind(&"127.0.0.1:6666".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap()
    };
    tokio::spawn(backend);
    tokio::time::sleep(Duration::from_millis(1000)).await;
    // Check that the initial value of our request counter is zero.
    let client = reqwest::Client::builder()
        .timeout(Duration::new(10, 0))
        .build()
        .context("creating HTTP client")?;
    let txt = client.get("http://localhost:6666/status")
        .send().await?
        .error_for_status()?
        .text().await
        .context("fetching status")?;
    assert!(txt.eq("0"), "Expecting 0 fragment requests, saw {txt}");

    // Check that the manifest and media segments both require authentication
    let mpd_fail = client.get("http://localhost:6666/mpd")
        .send().await
        .expect("unauthenticated manifest request");
    assert_eq!(mpd_fail.status(), StatusCode::BAD_REQUEST);
    let segment_fail = client.get("http://localhost:6666/media/foo.mp4")
        .send().await
        .expect("unauthenticated segment request");
    assert_eq!(segment_fail.status(), StatusCode::BAD_REQUEST);

    // Now download the media content from the MPD and check that the expected number of segments
    // were requested. Since we are in verbose mode, we make two requests for the init fragment, the
    // second request being to attempt to extract and print the PSSH.
    let outpath = env::temp_dir().join("basic_auth.mp4");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "-v", "-v", "-v",
               "--auth-username", "myuser", "--auth-password", "mypassword",
               "-o", outpath.to_str().unwrap(),
               "http://localhost:6666/mpd"])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    assert!(fs::metadata(outpath).is_ok());
    let txt = client.get("http://localhost:6666/status")
        .send().await?
        .error_for_status()?
        .text().await
        .context("fetching status")?;
    assert!(txt.eq("2"), "Expecting 2 fragment requests, saw {txt}");

    // This time we make the request in quiet mode and we should see only a single additional
    // request to the init segment.
    let outpath = env::temp_dir().join("basic_auth2.mp4");
    let cli = Command::new("cargo")
        .args(["run", "--no-default-features", "--",
               "--quiet",
               "--auth-username", "myuser", "--auth-password", "mypassword",
               "-o", outpath.to_str().unwrap(),
               "http://localhost:6666/mpd"])
        .output()
        .expect("failed spawning cargo run / dash-mpd-cli");
    assert!(cli.status.success());
    assert!(fs::metadata(outpath).is_ok());
    let txt = client.get("http://localhost:6666/status")
        .send().await?
        .error_for_status()?
        .text().await
        .context("fetching status")?;
    assert!(txt.eq("3"), "Expecting 3 fragment requests, saw {txt}");

    Ok(())
}
