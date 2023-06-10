// Test scaffolding to verify the --add-root-certificate functionality.
//
// To run tests while enabling printing to stdout/stderr, "cargo test -- --show-output" (from the
// root crate directory).
//
// What happens in this test:
//
//   - Start an axum https server using a certificate signed by our own (non-recognized) certificate
//   authority and valid for localhost.
//
//   - The axum server serves a MPD file, an MP4 segment (at "/init.mp4") and a status counter (at
//   "/status"). The MPD file refers to the MP4 segment. The status counter is initially zero and
//   increments for each request to download the MP4 segment.
//
//   - You can check that curl can connect to the server when our certificate authority is specified using 
//     curl --cacert tests/fixtures/root-CA.crt https://localhost:6666/mp
//
//   - Check that the initial status counter is zero, using a reqwest client configured with our
//   certificate authority as a root certificate.
//
//   - Run dash-mpd-cli (this crate using "cargo run") on the MPD URL, without adding our
//   certificate authority as a root certificate, and check that the request fails.
//
//   - Run dash-mpd-cli on the MPD URL, this time adding our certificate authority as a root
//   certificate. This should make a request for the MP4 segment and increment the status counter.
//
//   - Check that the status counter has been incremented to one.
//
//
// The steps needed to create a CA root certificate and a server certificate using openssl (note
// that rustls is finicky, requiring the subjectAltName field to be present):
//
//   openssl genrsa -out root-CA.key 4096
//   openssl req -x509 -new -nodes -subj "/C=FR/L=Toulouse/O=Test" -addext "basicConstraints=CA:true" -key root-CA.key -sha256 -days 1024 -out root-CA.crt
//   openssl genrsa -out localhost-cert.key 2048
//   openssl req -new -sha256 -key localhost-cert.key -subj "/C=FR/L=Toulouse/O=Test/CN=localhost" -addext 'subjectAltName = DNS:localhost' -out localhost-cert.csr
//   openssl x509 -req -in localhost-cert.csr -CA root-CA.crt -CAkey root-CA.key -CAcreateserial -out localhost-cert.crt -days 1500 -sha256 -copy_extensions copy


use fs_err as fs;
use std::net::SocketAddr;
use std::time::Duration;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use axum::{routing::get, Router};
use axum::extract::State;
use axum::response::{Response, IntoResponse};
use axum::http::{header, StatusCode};
use axum::body::{Full, Bytes};
use axum_server::tls_rustls::RustlsConfig;
use dash_mpd::{MPD, Period, AdaptationSet, Representation, BaseURL};
use anyhow::{Context, Result};


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
async fn test_add_root_cert() -> Result<(), anyhow::Error> {
    let base = BaseURL {
        base: "https://localhost:6666/init.mp4".to_string(),
        ..Default::default()
    };
    let rep = Representation {
        id: Some("1".to_string()),
        mimeType: Some("video/mp4".to_string()),
        codecs: Some("avc1.640028".to_string()),
        width: Some(1920),
        height: Some(800),
        bandwidth: Some(1980081),
        BaseURL: vec!(base),
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
        adaptations: vec!(adapt),
        ..Default::default()
    };
    let mpd = MPD {
        mpdtype: Some("static".to_string()),
        periods: vec!(period),
        ..Default::default()
    };
    let xml = quick_xml::se::to_string(&mpd)
        .context("serializing MPD struct")?;

    // State shared between the request handlers. We are simply maintaining a counter of the number
    // of requests to "/init.mp4", to check (via the "/status" route) that dash-mpd-cli has parsed the
    // MPD and requested the video segment.
    let shared_state = Arc::new(AppState::new());

    async fn send_mp4(State(state): State<Arc<AppState>>) -> Response<Full<Bytes>> {
        state.counter.fetch_add(1, Ordering::SeqCst);
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "video/mp4")
            .body(Full::from(vec![1, 2, 3, 4]))
            .unwrap()
    }

    async fn send_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        ([(header::CONTENT_TYPE, "text/plain")], format!("{}", state.counter.load(Ordering::Relaxed)))
    }

    let app = Router::new()
        .route("/mpd", get(|| async { ([(header::CONTENT_TYPE, "application/dash+xml")], xml) }))
        .route("/init.mp4", get(send_mp4))
        .route("/status", get(send_status))
        .with_state(shared_state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 6666));
    let config = RustlsConfig::from_pem_file(
        "tests/fixtures/localhost-cert.crt",
        "tests/fixtures/localhost-cert.key",
    )
        .await
        .context("rustls configuration")?;
    let backend = async move {
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await
            .unwrap()
    };
    tokio::spawn(backend);
    tokio::time::sleep(Duration::from_millis(1000)).await;
    // Check that the initial value of our request counter is zero.
    let crt = fs::read("tests/fixtures/root-CA.crt")?;
    let cert = reqwest::Certificate::from_pem(&crt)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::new(30, 0))
        .add_root_certificate(cert)
        .build()
        .context("creating HTTP client")?;
    let txt = client.get("https://localhost:6666/status")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await
        .context("fetching status")?;
    if ! txt.eq("0") {
        panic!("initial request count should be zero");
    }

    // Without the --add-root-certificate, should see an error from dash-mpd-cli "invalid peer
    // certificate: UnknownIssuer".
    if let Ok(failed) = Command::new("cargo")
        .args(["run", "--", "https://localhost:6666/mpd"])
        .output()
    {
        assert!(!failed.status.success());
        let stderr = String::from_utf8_lossy(&failed.stderr);
        // we are assuming that we build reqwest with rustls here, rather than with native-tls
        assert!(stderr.contains("UnknownIssuer"));
    } else {
        panic!("cargo run failure");
    }
    let cli = Command::new("cargo")
        .args(["run", "--",
               "-v", "-v", "-v",
               "--add-root-certificate",
               "tests/fixtures/root-CA.crt",
               "https://localhost:6666/mpd"])
        .output()
        .expect("failed spawning dash-mpd-cli");
    assert!(cli.status.success());

    // Check that the init.mp4 segment was fetched: request counter should be 1.
    let txt = client.get("https://localhost:6666/status")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await
        .context("fetching status")?;
    if ! txt.eq("1") {
        panic!("request count should be one");
    }
    Ok(())
}
