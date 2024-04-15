// Test network error handling code using a fault-injecting HTTP proxy
//
// To run tests while enabling printing to stdout/stderr
//
//    cargo test --test fetch_fault_injection -- --nocapture
//
// We use the toxiproxy proxy, but could also use Noxious which is a reimplementation in Rust.
// https://github.com/oguzbilgener/noxious

pub mod common;
use fs_err as fs;
use std::env;
use test_log::test;
use anyhow::{Context, Result};
use assert_cmd::Command;
use noxious_client::{Toxic, ToxicKind, StreamDirection};
use ffprobe::ffprobe;
use file_format::FileFormat;
use common::check_file_size_approx;


#[test(tokio::test)]
async fn test_dl_resilience() -> Result<()> {
    if env::var("CI").is_ok() {
        return Ok(());
    }
    let args = vec!["run", "--rm",
                    "--net", "host",
                    "-p", "8474:8474",
                    "-p", "8001:8001",
                    "ghcr.io/shopify/toxiproxy"];
    let _cli = std::process::Command::new("podman")
        .args(args)
        .spawn()
        .expect("failed spawning podman");
    tokio::time::sleep(tokio::time::Duration::new(2, 0)).await;
    let noxious = noxious_client::Client::new("http://localhost:8474");
    let noxious_proxy = noxious.create_proxy("dash-mpd-cli", "0.0.0.0:8001", "dash.akamaized.net:80").await
        .context("creating fault injecting HTTP proxy")?;
    let toxic_timeout = Toxic {
        kind: ToxicKind::Timeout { timeout: 4000000 },
        name: "fail".to_owned(),
        toxicity: 0.3,
        direction: StreamDirection::Downstream,
    };
    let toxic_fail = Toxic {
        kind: ToxicKind::Timeout { timeout: 40 },
        name: "timeout".to_owned(),
        toxicity: 0.4,
        direction: StreamDirection::Downstream,
    };
    let toxic_limit = Toxic {
        kind: ToxicKind::LimitData { bytes: 321 },
        name: "limit_data".to_owned(),
        toxicity: 0.5,
        direction: StreamDirection::Downstream,
    };
    // We wait 5 seconds before enabling the fault injection rules, so that the initial download of
    // the MPD manifest is not perturbed.
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::new(5, 0)).await;
        noxious_proxy.add_toxic(&toxic_fail).await
            .expect("adding failure toxicity failed");
        noxious_proxy.add_toxic(&toxic_timeout).await
            .expect("adding timeout toxicity failed");
        noxious_proxy.add_toxic(&toxic_limit).await
            .expect("adding limit toxicity failed");
        println!("Noxious toxics added");
    });

    let mpd = "http://dash.akamaized.net/dash264/TestCasesMCA/dts/1/Paint_dtsc_testA.mpd";
    let tmpd = tempfile::tempdir().unwrap();
    let out = tmpd.path().join("error-resilience.mkv");
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
        .args(["-v", "-v", "-v",
               "--ignore-content-type",
               "--quality", "best",
               "--proxy", "http://127.0.0.1:8001/",
               "-o", &out.to_string_lossy(), mpd])
        .assert()
        .success();
    check_file_size_approx(&out, 35_408_884);
    let meta = ffprobe(out.clone()).unwrap();
    assert_eq!(meta.streams.len(), 2);
    let audio = meta.streams.iter()
        .find(|s| s.codec_type.eq(&Some(String::from("audio"))))
        .expect("finding audio stream");
    assert_eq!(audio.codec_name, Some(String::from("dts")));
    let format = FileFormat::from_file(out.clone()).unwrap();
    assert_eq!(format, FileFormat::MatroskaVideo);
    let entries = fs::read_dir(tmpd.path()).unwrap();
    let count = entries.count();
    assert_eq!(count, 1, "Expecting a single output file, got {count}");
    let _ = fs::remove_dir_all(tmpd);

    Ok(())
}
