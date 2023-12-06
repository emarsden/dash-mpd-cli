/// Tests for the conformity checking functionality in the dash-mpd crate
///
// To run tests while enabling printing to stdout/stderr
//
//    cargo test --test conformity -- --show-output


use fs_err as fs;
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use predicates::prelude::*;
use assert_cmd::Command;
use axum::{routing::get, Router};
use axum::http::header;


#[test]
fn test_conformity_empty_period() {
    // This manifest contains an empty Period. Periods should have at least one AdaptationSet.
    let mpd = "http://download.tsi.telecom-paristech.fr/gpac/DASH_CONFORMANCE/TelecomParisTech/advanced/invalid_empty_period.mpd";
    let outpath = env::temp_dir().join("empty.mp4");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["--simulate",
               "-o", &outpath.to_string_lossy(), mpd])
        .assert()
        .stderr(predicate::str::contains("contains no AdaptationSet elements"))
        // This conformity check only generates a warning
        .success();
}

#[test]
fn test_conformity_maxheight () {
    let mpd = "http://download.tsi.telecom-paristech.fr/gpac/DASH_CONFORMANCE/TelecomParisTech/advanced/invalid_maxHeight.mpd";
    let outpath = env::temp_dir().join("empty.mp4");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["--simulate",
               "-o", &outpath.to_string_lossy(), mpd])
        .assert()
        .stderr(predicate::str::contains("invalid @maxHeight on AdaptationSet"))
        // This conformity check only generates a warning
        .success();
}


#[test]
fn test_conformity_invalid_maxwidth() {
    let mpd = "http://download.tsi.telecom-paristech.fr/gpac/DASH_CONFORMANCE/TelecomParisTech/advanced/invalid_maxWidth.mpd";
    let outpath = env::temp_dir().join("empty.mp4");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["--simulate",
               "-o", &outpath.to_string_lossy(), mpd])
        .assert()
        .stderr(predicate::str::contains("invalid @maxWidth on AdaptationSet"))
        // This conformity check only generates a warning
        .success();
}


// This DASH manifest is not spec compliant: it specifies a @maxHeight attribute on an AdaptationSet
// which is lower than the @height attribute on one of the child Representation elements.
#[test]
fn test_conformity_invalid_maxheight() {
    let mpd = "https://vod.infiniteplatform.tv/dash/vod-clear/ElephantsDream/default.mpd";
    let outpath = env::temp_dir().join("empty.mp4");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["--simulate",
               "-o", &outpath.to_string_lossy(), mpd])
        .assert()
        .stderr(predicate::str::contains("invalid @maxHeight on AdaptationSet"))
        // This conformity check only generates a warning
        .success();
}


#[test]
fn test_conformity_invalid_segment_duration() {
    let mpd = "http://download.tsi.telecom-paristech.fr/gpac/DASH_CONFORMANCE/TelecomParisTech/advanced/invalid_segmenttimeline_maxsegduration.mpd";
    let outpath = env::temp_dir().join("empty.mp4");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["--simulate",
               "-o", &outpath.to_string_lossy(), mpd])
        .assert()
        .stderr(predicate::str::contains("segment@d > @maxSegmentDuration"))
        // This conformity check only generates a warning
        .success();
}


// This an exemple DASH manifest from a commercial ad management platform which is not spec
// compliant. The MPD specifies maxSegmentDuration="PT2S", but the SegmentTimeline contains segments
// of duration 132300 / 44100 (3 seconds).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_conformity_invalid_maxsegmentduration() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("admanager");
    path.set_extension("xml");
    let xml = fs::read_to_string(path).unwrap();

    let app = Router::new()
        .route("/mpd", get(|| async { ([(header::CONTENT_TYPE, "application/dash+xml")], xml) }));
    let server_handle = axum_server::Handle::new();
    let backend_handle = server_handle.clone();
    let backend = async move {
        axum_server::bind("127.0.0.1:6666".parse().unwrap())
            .handle(backend_handle)
            .serve(app.into_make_service()).await
            .unwrap()
    };
    tokio::spawn(backend);
    tokio::time::sleep(Duration::from_millis(500)).await;

    let mpd = "http://localhost:6666/mpd";
    let outpath = env::temp_dir().join("empty.mp4");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["--simulate",
               "-o", &outpath.to_string_lossy(), mpd])
        .assert()
        .stderr(predicate::str::contains("segment@d > @maxSegmentDuration"))
        // This conformity check only generates a warning, but the download fails because it's a dynamic
        // manifest.
        .failure();
    server_handle.shutdown();
}

/*
   These tests moved from dash-mpd-rs now need to be run with a check of stderr for the warning message.
   Will need to use the local HTTP server method.

    let err1 = r#"<MPD><Period id="1">
       <AdaptationSet group="1">
         <Representation mimeType='video/mp4' width="320" height="240">
           <SegmentList duration="10">
             <Initialization sourceURL="httpunexist://example.com/segment.mp4"/>
             <SegmentURL media="seg1.mp4"/>
           </SegmentList>
         </Representation>
       </AdaptationSet>
     </Period></MPD>"#;
    assert!(parse(err1).is_err());

    let err2 = r#"<MPD><Period id="1">
       <AdaptationSet group="1">
         <Representation mimeType="video/mp4" width="320" height="240">
           <SegmentList duration="10">
             <SegmentURL media="https://example.com:-1/segment.mp4"/>
           </SegmentList>
         </Representation>
       </AdaptationSet>
     </Period></MPD>"#;
    assert!(parse(err2).is_err());

    let err3 = r#"<MPD><ProgramInformation moreInformationURL="https://192.168.1.2.3/segment.mp4" /></MPD>"#;
    assert!(parse(err3).is_err());

*/
