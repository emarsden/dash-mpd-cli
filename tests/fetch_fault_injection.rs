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
use serde_json::json;
use assert_cmd::Command;
use ffprobe::ffprobe;
use file_format::FileFormat;
use common::check_file_size_approx;


#[test(tokio::test)]
async fn test_dl_resilience() -> Result<()> {
    if env::var("CI").is_ok() {
        return Ok(());
    }
    // https://github.com/Shopify/toxiproxy
    //
    // Pull the container image before the run command, so that when we later run the container it
    // starts up within a repeatable timeframe.
    let pull = Command::new("podman")
        .args(["pull", "ghcr.io/shopify/toxiproxy"])
        .output()
        .expect("failed spawning podman");
    if !pull.status.success() {
        let stdout = String::from_utf8_lossy(&pull.stdout);
        if stdout.len() > 0 {
            println!("Podman stdout> {stdout}");
        }
        let stderr = String::from_utf8_lossy(&pull.stderr);
        if stderr.len() > 0 {
            println!("Podman stderr> {stderr}");
        }
    }
    assert!(pull.status.success());
    let args = vec!["run", "--rm",
                    "--name", "toxiproxy",
                    "--net", "host",
                    "-p", "8474:8474",
                    "-p", "8001:8001",
                    "ghcr.io/shopify/toxiproxy"];
    let _cli = std::process::Command::new("podman")
        .args(args)
        .spawn()
        .expect("failed spawning podman");
    tokio::time::sleep(tokio::time::Duration::new(2, 0)).await;
    let txclient = reqwest::Client::new();
    // Enable the toxiproxy proxy.
    txclient.post("http://localhost:8474/proxies")
        .json(&json!({
            "name": "dash-mpd-rs",
            "listen": "0.0.0.0:8001",
            // "upstream": "dash.akamaized.net:80",
            "upstream": "ftp.itec.aau.at:80",
            "enabled": true
        }))
        .send().await?;
    // Add a timeout Toxic with a very large timeout (amounts to a failure).
    txclient.post("http://localhost:8474/proxies/dash-mpd-rs/toxics")
        .json(&json!({
            "type": "timeout",
            "name": "fail",
            "toxicity": 0.3,
            "attributes": { "timeout": 4000000 },
        }))
        .send().await
        .expect("creating timeout toxic");
    // Add a data rate limitation Toxic.
    txclient.post("http://localhost:8474/proxies/dash-mpd-rs/toxics")
        .json(&json!({
            "type": "limit_data",
            "toxicity": 0.5,
            "attributes": { "bytes": 321 },
        }))
        .send().await
        .expect("creating timeout toxic");

    // We wait 5 seconds before enabling the fault injection rules, so that the initial download of
    // the MPD manifest is not perturbed.
    let _configer = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::new(0, 5000)).await;
        println!("Injecting toxics");
        let txfail = json!({
            "type": "timeout",
            "toxicity": 0.3,
            "attributes": { "timeout": 4000000 },
        });
        txclient.post("http://localhost:8474/proxies/dash-mpd-rs/toxics")
            .json(&txfail)
            .send().await
            .expect("creating timeout toxic");
        let txlimit = json!({
            "type": "limit_data",
            "toxicity": 0.5,
            "attributes": { "bytes": 321 },
        });
        txclient.post("http://localhost:8474/proxies/dash-mpd-rs/toxics")
            .json(&txlimit)
            .send().await
            .expect("creating timeout toxic");
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

    let _stop = Command::new("podman")
        .args(["stop", "toxiproxy"])
        .output()
        .expect("failed to spawn podman");

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
