//! dash-mpd-cli
//!
//! A commandline application for downloading media content from a DASH MPD file, as used for on-demand
//! replay of TV content and video streaming services like YouTube.
//!
//! [DASH](https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP) (dynamic adaptive
//! streaming over HTTP), also called MPEG-DASH, is a technology used for media streaming over the web,
//! commonly used for video on demand (VOD) services. The Media Presentation Description (MPD) is a
//! description of the resources (manifest or “playlist”) forming a streaming service, that a DASH
//! client uses to determine which assets to request in order to perform adaptive streaming of the
//! content. DASH MPD manifests can be used both with content encoded as MPEG and as WebM.
//!
//! This commandline application allows you to download content (audio or video) described by an MPD
//! manifest. This involves selecting the alternative with the most appropriate encoding (in terms of
//! bitrate, codec, etc.), fetching segments of the content using HTTP or HTTPS requests and muxing
//! audio and video segments together. It builds on the [dash-mpd](https://crates.io/crates/dash-mpd)
//! crate.
//
//
// Example usage: dash-mpd-cli --timeout 5 --output=/tmp/foo.mp4 https://v.redd.it/zv89llsvexdz/DASHPlaylist.mpd

use std::env;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;
use std::sync::Arc;
use std::collections::HashMap;
use env_logger::Env;
use reqwest::header;
use clap::Arg;
use indicatif::{ProgressBar, ProgressStyle};
use anyhow::Result;
use dash_mpd::fetch::DashDownloader;
use dash_mpd::fetch::ProgressObserver;


struct DownloadProgressBar {
    bar: ProgressBar,
}

impl DownloadProgressBar {
    pub fn new() -> Self {
        let b = ProgressBar::new(100)
            .with_style(ProgressStyle::default_bar()
                        .template("[{elapsed}] [{bar:50.cyan/blue}] {wide_msg}")
                        .progress_chars("#>-"));
        Self { bar: b }
    }
}

impl ProgressObserver for DownloadProgressBar {
    fn update(&self, percent: u32, message: &str) {
        if percent <= 100 {
            self.bar.set_position(percent.into());
            self.bar.set_message(message.to_string());
        }
        if percent == 100 {
            self.bar.finish_with_message("Done");
        }
    }
}


fn main () -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info,reqwest=warn")).init();
    let matches = clap::Command::new("dash-mpd-cli")
        .about("Download content from a DASH streaming media manifest")
        .version("0.1.2")
        .arg(Arg::new("user-agent")
             .long("user-agent")
             .takes_value(true))
        .arg(Arg::new("proxy")
             .long("proxy")
             .takes_value(true))
        .arg(Arg::new("timeout")
             .long("timeout")
             .takes_value(true)
             .help("Timeout for network requests (from the start to the end of the request), in seconds"))
        .arg(Arg::new("sleep-requests")
             .long("sleep-requests")
             .takes_value(true)
             .help("Number of seconds to sleep between network requests (default 0)"))
        .arg(Arg::new("source-address")
             .long("source-address")
             .takes_value(true)
	     .help("Source IP address to use for network requests, either IPv4 or IPv6. Network requests will be made using the version of this IP address (eg. using an IPv6 source-address will select IPv6 network traffic)."))
        .arg(Arg::new("quality")
             .long("quality")
             .takes_value(true)
             .possible_value("best")
             .possible_value("worst")
             .help("Prefer best quality (and highest bandwidth) representation, or lowest quality"))
        .arg(Arg::new("prefer-language")
             .long("prefer-language")
             .takes_value(true)
             .help("Preferred language when multiple audio streams with different languages are available. Must be in RFC 5646 format (eg. fr or en-AU). If a preference is not specified and multiple audio streams are present, the first one listed in the DASH manifest will be downloaded."))
        .arg(Arg::new("video-only")
             .long("video-only")
             .takes_value(false)
             .conflicts_with("audio-only")
             .help("If the media stream has separate audio and video streams, only download the video stream"))
        .arg(Arg::new("audio-only")
             .long("audio-only")
             .takes_value(false)
             .conflicts_with("video-only")
             .help("If the media stream has separate audio and video streams, only download the audio stream"))
        .arg(Arg::new("add-header")
             .long("add-header")
             .takes_value(true)
             .multiple_occurrences(true)
             .help("Add a custom HTTP header and its value, separated by a colon ':'. You can use this option multiple times."))
        .arg(Arg::new("quiet")
             .short('q')
             .long("quiet")
             .conflicts_with("verbose")
             .takes_value(false))
        .arg(Arg::new("verbose")
             .short('v')
             .long("verbose")
             .takes_value(false)
             .multiple_occurrences(true)
             .help("Level of verbosity (can be used several times)"))
        .arg(Arg::new("no-progress")
             .long("no-progress")
             .takes_value(false)
             .help("Disable the progress bar"))
        .arg(Arg::new("no-xattr")
             .long("no-xattr")
             .takes_value(false)
             .help("Don't record metainformation as extended attributes in the output file"))
        .arg(Arg::new("ffmpeg-location")
             .long("ffmpeg-location")
             .takes_value(true)
             .help("Path to the ffmpeg binary (necessary if not located in your PATH)"))
        .arg(Arg::new("version")
             .long("version")
             .takes_value(false))
        .arg(Arg::new("output")
             .long("output")
             .short('o')
             .takes_value(true)
             .help("Save media content to this file"))
        .arg(Arg::new("url")
             .takes_value(true)
             .value_name("MPD-URL")
             .required(true)
             .index(1)
             .help("URL of the DASH manifest to retrieve"))
        .get_matches();
    // TODO: add --abort-on-error
    // TODO: add --fragment-retries arg
    // TODO: add --mtime arg (Last-modified header)
    // TODO: limit download rate once reqwest crate can do so
    let verbosity = matches.occurrences_of("verbose") as u8;
    let ua = matches.value_of("user-agent")
        .unwrap_or(concat!("dash-mpd-cli/", env!("CARGO_PKG_VERSION")));
    let mut cb = reqwest::blocking::Client::builder()
        .user_agent(ua)
	.tcp_nodelay(true)
        .gzip(true)
        .brotli(true);
    if verbosity > 2 {
       cb = cb.connection_verbose(true);
    }
    if let Some(p) = matches.value_of("proxy") {
        let proxy = reqwest::Proxy::http(p)
            .expect("Can't connect to HTTP proxy");
        cb = cb.proxy(proxy);
    }
    if let Some(src) = matches.value_of("source-address") {
       if let Ok(local_addr) = IpAddr::from_str(src) {
          cb = cb.local_address(local_addr);
       } else {
          eprintln!("Ignoring invalid argument to --source-address: {}", src);
       }
    }
    if let Some(seconds) = matches.value_of("timeout") {
        if let Ok(secs) = seconds.parse::<u64>() {
            cb = cb.timeout(Duration::new(secs, 0));
        } else {
            eprintln!("Ignoring invalid value for --timeout: {}", seconds);
        }
    } else {
        cb = cb.timeout(Duration::new(30, 0));
    }
    if let Some(hvs) = matches.values_of("add-header") {
        let mut headers = HashMap::new();
        for hv in hvs.collect::<Vec<_>>() {
            if let Some(pos) = hv.find(':') {
                let (h, v) = hv.split_at(pos);
                headers.insert(h.to_string(), v.to_string());
            } else {
                eprintln!("Ignoring badly formed header:value argument to --add-header");
            }
        }
        let hmap: header::HeaderMap = (&headers).try_into()
            .expect("valid HTTP headers");
        cb = cb.default_headers(hmap);
    }

    let client = cb.build()
        .expect("Couldn't create reqwest HTTP client");
    let url = matches.value_of("url").unwrap();
    let mut dl = DashDownloader::new(url)
        .with_http_client(client);
    if !matches.is_present("no-progress") && !matches.is_present("quiet") {
        dl = dl.add_progress_observer(Arc::new(DownloadProgressBar::new()));
    }
    if let Some(seconds) = matches.value_of("sleep-requests") {
        if let Ok(secs) = seconds.parse::<u8>() {
            dl = dl.sleep_between_requests(secs);
        } else {
            eprintln!("Ignoring invalid value for --sleep-requests: {}", seconds);
        }
    }
    if matches.is_present("audio-only") {
        dl = dl.audio_only();
    }
    if matches.is_present("video-only") {
        dl = dl.video_only();
    }
    if matches.is_present("no-xattr") {
        dl = dl.record_metainformation(false);
    }
    if let Some(ffmpeg_path) = matches.value_of("ffmpeg-location") {
        dl = dl.with_ffmpeg(ffmpeg_path);
    }
    if let Some(q) = matches.value_of("quality") {
        if q.eq("best") {
            // DashDownloader defaults to worst quality
            dl = dl.best_quality();
        }
    }
    if let Some(lang) = matches.value_of("prefer-language") {
        dl = dl.prefer_language(lang.to_string());
    }
    dl = dl.verbosity(verbosity);
    if let Some(out) = matches.value_of("output") {
        dl.download_to(out)?;
    } else {
        let out = dl.download()?;
        println!("Downloaded DASH content to {:?}", out);
    }
    std::process::exit(0)
}
