// Testing support for sending various HTTP header (referer, user-agent).
//
//
// To run tests while enabling printing to stdout/stderr
//
//    RUST_LOG=info cargo test --test headers -- --show-output
//
// What happens in this test:
//
//   - Start an axum HTTP server that serves the manifest and our media segments.
//
//   - Fetch the associated media content using dash-mpd-cli via "cargo run" with the --referer,
//   --user-agent, -add-header and --header options. Check that both the MPD manifest and a media
//   segment are retrieved with the expected headers.


pub mod common;
use fs_err as fs;
use std::env;
use std::time::Duration;
use assert_cmd::Command;
use axum::{routing::get, Router};
use axum::response::{Response, IntoResponse};
use axum::http::header::HeaderMap;
use axum::http::{header, StatusCode};
use axum::body::Body;
use hyper_serve::Server;
use dash_mpd::{MPD, Period, AdaptationSet, Representation, SegmentTemplate};
use anyhow::Result;
use common::{generate_minimal_mp4, setup_logging};


#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_headers() -> Result<()> {
    async fn send_mpd(headers: HeaderMap) -> impl IntoResponse {
        assert_eq!(headers["user-agent"], "MyFakeUserAgent/42.0");
        assert_eq!(headers["referer"], "https://twiddles.org/");
        assert_eq!(headers["x-twizzles"], "extra");
        assert_eq!(headers["x-foobles"], "bøzzles".as_bytes());

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

    async fn send_mp4(headers: HeaderMap) -> Response {
        assert_eq!(headers["user-agent"], "MyFakeUserAgent/42.0");
        assert_eq!(headers["referer"], "https://twiddles.org/");
        assert_eq!(headers["x-twizzles"], "extra");
        assert_eq!(headers["x-foobles"], "bøzzles".as_bytes());

        let data = generate_minimal_mp4();
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "video/mp4")
            .body(Body::from(data))
            .unwrap()
    }

    setup_logging();
    let app = Router::new()
        .route("/mpd", get(send_mpd))
        .route("/media/{seg}", get(send_mp4));
    let backend = async move {
        Server::bind("127.0.0.1:6661".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap()
    };
    tokio::spawn(backend);
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let outpath = env::temp_dir().join("referer.mp4");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v", "-v", "-v",
               "--referer", "https://twiddles.org/",
               "--user-agent", "MyFakeUserAgent/42.0",
               "--add-header", "X-FOOBLES:bøzzles",
               "--header", "X-Twizzles: extra",
               "-o", outpath.to_str().unwrap(),
               "http://localhost:6661/mpd"])
        .assert()
        .success();
    assert!(fs::metadata(outpath).is_ok());
    Ok(())
}
