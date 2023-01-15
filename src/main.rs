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
use clap::{Arg, ArgAction};
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
                        .expect("building progress bar")
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
        .about("Download content from an MPEG-DASH streaming media manifest")
        .version(clap::crate_version!())
        .arg(Arg::new("user-agent")
             .long("user-agent")
             .num_args(1))
        .arg(Arg::new("proxy")
             .long("proxy")
             .value_name("URL")
             .num_args(1)
             .help("URL of Socks or HTTP proxy (eg. https://example.net/ or socks5://example.net/)"))
        .arg(Arg::new("timeout")
             .long("timeout")
             .value_name("SECONDS")
             .num_args(1)
             .help("Timeout for network requests (from the start to the end of the request), in seconds"))
        .arg(Arg::new("sleep-requests")
             .long("sleep-requests")
             .value_name("SECONDS")
             .num_args(1)
             .help("Number of seconds to sleep between network requests (default 0)"))
        .arg(Arg::new("source-address")
             .long("source-address")
             .num_args(1)
	     .help("Source IP address to use for network requests, either IPv4 or IPv6. Network requests will be made using the version of this IP address (eg. using an IPv6 source-address will select IPv6 network traffic)."))
        .arg(Arg::new("quality")
             .long("quality")
             .num_args(1)
             .value_parser(["best", "worst"])
             .help("Prefer best quality (and highest bandwidth) representation, or lowest quality"))
        .arg(Arg::new("prefer-language")
             .long("prefer-language")
             .value_name("LANG")
             .num_args(1)
             .help("Preferred language when multiple audio streams with different languages are available. Must be in RFC 5646 format (eg. fr or en-AU). If a preference is not specified and multiple audio streams are present, the first one listed in the DASH manifest will be downloaded."))
        .arg(Arg::new("video-only")
             .long("video-only")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .conflicts_with("audio-only")
             .help("If the media stream has separate audio and video streams, only download the video stream"))
        .arg(Arg::new("audio-only")
             .long("audio-only")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .conflicts_with("video-only")
             .help("If the media stream has separate audio and video streams, only download the audio stream"))
        .arg(Arg::new("write-subs")
             .long("write-subs")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .help("Write subtitle file, if subtitles are available"))
        .arg(Arg::new("keep-video")
             .long("keep-video")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .help("Don't delete the file containing video once muxing is complete."))
        .arg(Arg::new("keep-audio")
             .long("keep-audio")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .help("Don't delete the file containing audio once muxing is complete."))
        .arg(Arg::new("ignore-content-type")
             .long("ignore-content-type")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .help("Don't check the content-type of media fragments (may be required for some poorly configured servers)"))
        .arg(Arg::new("add-header")
             .long("add-header")
             .value_name("NAME:VALUE")
             .num_args(1)
             .action(clap::ArgAction::Append)
             .help("Add a custom HTTP header and its value, separated by a colon ':'. You can use this option multiple times."))
        .arg(Arg::new("quiet")
             .short('q')
             .long("quiet")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .conflicts_with("verbose"))
        .arg(Arg::new("verbose")
             .short('v')
             .long("verbose")
             .action(clap::ArgAction::Count)
             .help("Level of verbosity (can be used several times)"))
        .arg(Arg::new("no-progress")
             .long("no-progress")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .help("Disable the progress bar"))
        .arg(Arg::new("no-xattr")
             .long("no-xattr")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .help("Don't record metainformation as extended attributes in the output file"))
        .arg(Arg::new("ffmpeg-location")
             .long("ffmpeg-location")
             .value_name("PATH")
             .num_args(1)
             .help("Path to the ffmpeg binary (necessary if not located in your PATH)"))
        .arg(Arg::new("vlc-location")
             .long("vlc-location")
             .value_name("PATH")
             .num_args(1)
             .help("Path to the VLC binary (necessary if not located in your PATH)"))
        .arg(Arg::new("mkvmerge-location")
             .long("mkvmerge-location")
             .value_name("PATH")
             .num_args(1)
             .help("Path to the mkvmerge binary (necessary if not located in your PATH)"))
        .arg(Arg::new("output-file")
             .long("output")
             .value_name("PATH")
             .short('o')
             .num_args(1)
             .help("Save media content to this file"))
        .arg(Arg::new("url")
             .value_name("MPD-URL")
             .required(true)
             .num_args(1)
             .index(1)
             .help("URL of the DASH manifest to retrieve"))
        .get_matches();
    // TODO: add --abort-on-error
    // TODO: add --fragment-retries arg
    // TODO: add --mtime arg (Last-modified header)
    // TODO: limit download rate once reqwest crate can do so
    let verbosity = matches.get_count("verbose");
    let ua = match matches.get_one::<String>("user-agent") {
        Some(ua) => ua,
        None => concat!("dash-mpd-cli/", env!("CARGO_PKG_VERSION")),
    };
    let mut cb = reqwest::blocking::Client::builder()
        .user_agent(ua)
        .gzip(true)
        .brotli(true);
    if verbosity > 2 {
       cb = cb.connection_verbose(true);
    }
    if let Some(p) = matches.get_one::<String>("proxy") {
        let proxy = reqwest::Proxy::all(p)
            .expect("connecting to HTTP proxy");
        cb = cb.proxy(proxy);
    }
    if let Some(src) = matches.get_one::<String>("source-address") {
       if let Ok(local_addr) = IpAddr::from_str(src) {
          cb = cb.local_address(local_addr);
       } else {
          eprintln!("Ignoring invalid argument to --source-address: {src}");
       }
    }
    if let Some(seconds) = matches.get_one::<String>("timeout") {
        if let Ok(secs) = seconds.parse::<u64>() {
            cb = cb.timeout(Duration::new(secs, 0));
        } else {
            eprintln!("Ignoring invalid value for --timeout: {seconds}");
        }
    } else {
        cb = cb.timeout(Duration::new(30, 0));
    }
    if let Some(hvs) = matches.get_many::<String>("add-header") {
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
        .expect("creating reqwest HTTP client");
    let url = matches.get_one::<String>("url").unwrap();
    let mut dl = DashDownloader::new(url)
        .with_http_client(client);
    if !matches.get_flag("no-progress") && !matches.get_flag("quiet") {
        dl = dl.add_progress_observer(Arc::new(DownloadProgressBar::new()));
    }
    if let Some(seconds) = matches.get_one::<u8>("sleep-requests") {
        dl = dl.sleep_between_requests(*seconds);
    }
    if matches.get_flag("audio-only") {
        dl = dl.audio_only();
    }
    if matches.get_flag("video-only") {
        dl = dl.video_only();
    }
    if matches.get_flag("keep-video") {
        dl = dl.keep_video();
    }
    if matches.get_flag("keep-audio") {
        dl = dl.keep_audio();
    }
    if matches.get_flag("write-subs") {
        dl = dl.fetch_subtitles();
    }
    if matches.get_flag("ignore-content-type") {
        dl = dl.without_content_type_checks();
    }
    if matches.get_flag("no-xattr") {
        dl = dl.record_metainformation(false);
    }
    if let Some(ffmpeg_path) = matches.get_one::<String>("ffmpeg-location") {
        dl = dl.with_ffmpeg(ffmpeg_path);
    }
    if let Some(path) = matches.get_one::<String>("vlc-location") {
        dl = dl.with_vlc(path);
    }
    if let Some(path) = matches.get_one::<String>("mkvmerge-location") {
        dl = dl.with_mkvmerge(path);
    }
    if let Some(q) = matches.get_one::<String>("quality") {
        if q.eq("best") {
            // DashDownloader defaults to worst quality
            dl = dl.best_quality();
        }
    }
    if let Some(lang) = matches.get_one::<String>("prefer-language") {
        dl = dl.prefer_language(lang.to_string());
    }
    dl = dl.verbosity(verbosity);
    if let Some(out) = matches.get_one::<String>("output-file") {
        if let Err(e) = dl.download_to(out) {
            eprintln!("Download error: {e:?}");
        }
    } else {
        match dl.download() {
            Ok(out) => println!("Downloaded DASH content to {out:?}"),
            Err(e) => {
                eprintln!("Download error: {e}");
                std::process::exit(2);
            },
        }
    }
    std::process::exit(0)
}
