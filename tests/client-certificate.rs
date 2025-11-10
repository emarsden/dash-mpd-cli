// Test scaffolding to verify the --client-identity-certificate functionality.
//
// To run tests while enabling printing to stdout/stderr, "cargo test -- --show-output" (from the
// root crate directory).
//
// What happens in this test:
//
//   - Start an axum https server using a certificate signed by our own (private) certificate
//   authority and valid for localhost. The server is configured only to allow requests from
//   authenticated clients (ones that present a TLS client certificated signed by our certificate
//   authority).
//
//   - The axum server serves a MPD file, an MP4 segment (at "/init.mp4") and a status counter (at
//   "/status"). The MPD file refers to the MP4 segment. The status counter is initially zero and
//   increments for each request to download the MP4 segment.
//
//   - You can check that curl can connect to the server when our certificate authority is specified
//   and a client certificate provided using (with an extra sleep at the end of the test):
//
//     curl --cacert tests/fixtures/root-CA.crt --cert tests/fixtures/client-id.pem https://localhost:6666/mpd
//
//   - Check that the initial status counter is zero, using a reqwest client configured with our
//   certificate authority as a root certificate and with the client identity specified.
//
//   - Run dash-mpd-cli (this crate using "cargo run") on the MPD URL, with our certificate
//   authority as a root certificate but without a client certificate, and check that the request
//   fails.
//
//   - Run dash-mpd-cli on the MPD URL, this time adding our certificate authority as a root
//   certificate and specifying the client identity. This should make a request for the MP4 segment
//   and increment the status counter.
//
//   - Check that the status counter has been incremented to one.
//


use fs_err as fs;
use std::net::SocketAddr;
use std::time::Duration;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use assert_cmd::cargo::cargo_bin_cmd;
use axum::{routing::get, Router};
use axum::extract::State;
use axum::response::{Response, IntoResponse};
use axum::http::{header, StatusCode};
use axum::body::Body;
use hyper_serve::tls_rustls::RustlsConfig;
use rustls::RootCertStore;
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use dash_mpd::{MPD, Period, AdaptationSet, Representation, BaseURL};
use anyhow::{Context, Result};
use test_log::test;


#[derive(Debug, Default)]
struct AppState {
    counter: AtomicUsize,
}

impl AppState {
    fn new() -> AppState {
        AppState { counter: AtomicUsize::new(0) }
    }
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 2))]
async fn test_add_client_identity() -> Result<(), anyhow::Error> {
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

    async fn send_mp4(State(state): State<Arc<AppState>>) -> Response {
        state.counter.fetch_add(1, Ordering::SeqCst);
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "video/mp4")
            .body(Body::from(include_bytes!("fixtures/minimal-valid.mp4").as_slice()))
            .unwrap()
    }

    async fn send_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
        ([(header::CONTENT_TYPE, "text/plain")], format!("{}", state.counter.load(Ordering::Relaxed)))
    }

    rustls::crypto::aws_lc_rs::default_provider().install_default().unwrap();
    let app = Router::new()
        .route("/mpd", get(|| async { ([(header::CONTENT_TYPE, "application/dash+xml")], xml) }))
        .route("/init.mp4", get(send_mp4))
        .route("/status", get(send_status))
        .with_state(shared_state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 6666));
    let mut client_auth_roots = RootCertStore::empty();
    let root_cert = fs::File::open("tests/fixtures/root-CA.crt")?;
    for maybe_cert in rustls_pemfile::certs(&mut BufReader::new(root_cert)) {
        client_auth_roots.add(maybe_cert.unwrap()).unwrap();
    }
    let client_verifier = WebPkiClientVerifier::builder(client_auth_roots.into())
        .build()
        .unwrap();
    /* this convenience method doesn't allow us to specify client verifier
    let config = RustlsConfig::from_pem_file(
        "tests/fixtures/localhost-cert.crt",
        "tests/fixtures/localhost-cert.key")
        .await
        .unwrap()
     */
    let cert_file = fs::File::open("tests/fixtures/localhost-cert.crt")?;
    let mut certificates: Vec<CertificateDer<'static>> = Vec::new();
    for maybe_cert in rustls_pemfile::certs(&mut BufReader::new(cert_file)) {
        certificates.push(maybe_cert.unwrap());
    }
    let key_file = fs::File::open("tests/fixtures/localhost-cert.key")?;
    let mut keys: Vec<PrivateKeyDer> = Vec::new();
    for maybe_key in rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(key_file)) {
        keys.push(PrivateKeyDer::Pkcs8(maybe_key.unwrap()));
    }
    keys.reverse();
    let key = keys.pop().expect("no private key");
    let mut config = ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(certificates, key)
        .expect("bad server certificate/key");
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let backend = async move {
        hyper_serve::tls_rustls::bind_rustls(addr, RustlsConfig::from_config(config.into()))
            .serve(app.into_make_service()).await
            .unwrap()
    };
    tokio::spawn(backend);
    tokio::time::sleep(Duration::from_millis(1000)).await;
    // Check that the initial value of our request counter is zero.
    let client_id = fs::read("tests/fixtures/client-id.pem")?;
    let id = reqwest::Identity::from_pem(&client_id)
        .context("reading client identity from certificate")?;
    let crt = fs::read("tests/fixtures/root-CA.crt")?;
    let root_cert = reqwest::Certificate::from_pem(&crt)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::new(30, 0))
        .identity(id)
        .add_root_certificate(root_cert)
        .build()
        .context("creating HTTP client")?;
    let txt = client.get("https://localhost:6666/status")
        .send().await?
        .error_for_status()?
        .text().await
        .context("fetching status")?;
    // The initial request count should be zero.
    assert!(txt.eq("0"));

    // Without the --client-identity-certificate, should see an error from dash-mpd-cli due to the
    // server refusing the connection (with rustls, is "channel closed").
    cargo_bin_cmd!()
        .args(["--add-root-certificate", "tests/fixtures/root-CA.crt",
               "https://localhost:6666/mpd"])
        .assert()
        .failure();
    cargo_bin_cmd!()
        .args(["-v", "-v", "-v",
               "--add-root-certificate", "tests/fixtures/root-CA.crt",
               "--client-identity-certificate", "tests/fixtures/client-id.pem",
               "https://localhost:6666/mpd"])
        .assert()
        .success();
    /* TODO: turn this into a predicate::str 
    let msg = String::from_utf8_lossy(&cli.stderr);
    if msg.len() > 0 {
        eprintln!("cli stderr: {msg}");
    }
    */

    // Check that the init.mp4 segment was fetched: request counter should be 1.
    let txt = client.get("https://localhost:6666/status")
        .send().await?
        .error_for_status()?
        .text().await
        .context("fetching status")?;
    assert!(txt.eq("1"));

    // allow test connection using another HTTP client such as curl
    // tokio::time::sleep(Duration::from_millis(10000)).await;

    Ok(())
}
