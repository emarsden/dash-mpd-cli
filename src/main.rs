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
use std::path::Path;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;
use std::sync::Arc;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use env_logger::Env;
use reqwest::header;
use bench_scraper::{find_cookies, KnownBrowser};
use clap::{Arg, ArgAction, ValueHint};
use number_prefix::{NumberPrefix, Prefix};
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


#[tokio::main]
async fn main () -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info,reqwest=warn")).init();
    let known_browser_names = KnownBrowser::iter()
        .map(|b| format!("{b:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    let matches = clap::Command::new("dash-mpd-cli")
        .about("Download content from an MPEG-DASH streaming media manifest")
        .version(clap::crate_version!())
        .arg(Arg::new("user-agent")
             .long("user-agent")
             .short('U')
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
             .value_parser(clap::value_parser!(u8))
             .help("Number of seconds to sleep between network requests (default 0)"))
        .arg(Arg::new("limit-rate")
             .long("limit-rate")
             .short('r')
             .value_name("RATE")
             .num_args(1)
             .help("Maximum network bandwidth in octets per second (default no limit), e.g. 200K, 1M"))
        .arg(Arg::new("max-error-count")
             .long("max-error-count")
             .value_name("COUNT")
             .num_args(1)
             .value_parser(clap::value_parser!(u32))
             .help("Maximum number of non-transient network errors that should be ignored before a download is aborted (default is 10)"))
        .arg(Arg::new("source-address")
             .long("source-address")
             .num_args(1)
	     .help("Source IP address to use for network requests, either IPv4 or IPv6. Network requests will be made using the version of this IP address (e.g. using an IPv6 source-address will select IPv6 network traffic)."))
        .arg(Arg::new("quality")
             .long("quality")
             .num_args(1)
             .value_parser(["best", "worst"])
             .help("Prefer best quality (and highest bandwidth) representation, or lowest quality"))
        .arg(Arg::new("prefer-language")
             .long("prefer-language")
             .value_name("LANG")
             .num_args(1)
             .help("Preferred language when multiple audio streams with different languages are available. Must be in RFC 5646 format (e.g. fr or en-AU). If a preference is not specified and multiple audio streams are present, the first one listed in the DASH manifest will be downloaded."))
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
             .help("Write subtitle file, if subtitles are available."))
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
        .arg(Arg::new("save-fragments")
             .long("save-fragments")
             .value_name("FRAGMENTS-DIR")
             .value_hint(ValueHint::DirPath)
             .num_args(1)
             .help("Save media fragments to this directory (will be created if it does not exist)."))
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
        .arg(Arg::new("cookies-from-browser")
             .long("cookies-from-browser")
             .value_name("BROWSER")
             .num_args(1)
             .help(format!("Load cookies from BROWSER ({known_browser_names}).")))
        .arg(Arg::new("list-cookie-sources")
             .long("list-cookie-sources")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .exclusive(true)
             .help("Show valid values for the BROWSER argument to --cookies-from-browser on this computer, then exit."))
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
             .help("Level of verbosity (can be used several times)."))
        .arg(Arg::new("no-progress")
             .long("no-progress")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .help("Disable the progress bar"))
        .arg(Arg::new("no-xattr")
             .long("no-xattr")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .help("Don't record metainformation as extended attributes in the output file."))
        .arg(Arg::new("ffmpeg-location")
             .long("ffmpeg-location")
             .value_name("PATH")
             .value_hint(ValueHint::ExecutablePath)
             .num_args(1)
             .help("Path to the ffmpeg binary (necessary if not located in your PATH)."))
        .arg(Arg::new("vlc-location")
             .long("vlc-location")
             .value_name("PATH")
             .value_hint(ValueHint::ExecutablePath)
             .num_args(1)
             .help("Path to the VLC binary (necessary if not located in your PATH)."))
        .arg(Arg::new("mkvmerge-location")
             .long("mkvmerge-location")
             .value_name("PATH")
             .value_hint(ValueHint::ExecutablePath)
             .num_args(1)
             .help("Path to the mkvmerge binary (necessary if not located in your PATH)."))
        .arg(Arg::new("mp4box-location")
             .long("mp4box-location")
             .value_name("PATH")
             .value_hint(ValueHint::ExecutablePath)
             .num_args(1)
             .help("Path to the MP4Box binary (necessary if not located in your PATH)."))
        .arg(Arg::new("output-file")
             .long("output")
             .value_name("PATH")
             .value_hint(ValueHint::FilePath)
             .short('o')
             .num_args(1)
             .help("Save media content to this file."))
        .arg(Arg::new("url")
             .value_name("MPD-URL")
             .value_hint(ValueHint::Url)
             .required(true)
             .num_args(1)
             .index(1)
             .help("URL of the DASH manifest to retrieve."))
        .get_matches();
    // TODO: add --abort-on-error
    // TODO: add --fragment-retries arg
    // TODO: add --mtime arg (Last-modified header)
    if matches.get_flag("list-cookie-sources") {
        eprintln!("On this computer, cookies are available from the following browsers :");
        let browsers = find_cookies()
            .expect("reading cookies from browser");
        for b in browsers.iter() {
            eprintln!("  {:?} ({} cookies)", b.browser, b.cookies.len());
        }
        std::process::exit(3);
    }
    let verbosity = matches.get_count("verbose");
    let ua = match matches.get_one::<String>("user-agent") {
        Some(ua) => ua,
        None => concat!("dash-mpd-cli/", env!("CARGO_PKG_VERSION")),
    };
    let mut cb = reqwest::Client::builder()
        .user_agent(ua)
        .gzip(true)
        .brotli(true);
    if let Some(browser) = matches.get_one::<String>("cookies-from-browser") {
        if let Some(wanted) = match browser.as_str() {
            "Firefox" => Some(KnownBrowser::Firefox),
            "Chrome" => Some(KnownBrowser::Chrome),
            "ChromeBeta" => Some(KnownBrowser::ChromeBeta),
            "Chromium" => Some(KnownBrowser::Chromium),
            #[cfg(target_os = "windows")]
            "Edge" => Some(KnownBrowser::Edge),
            #[cfg(target_os = "macos")]
            "Safari" => Some(KnownBrowser::Safari),
            _ => None,
        } {
            let jar = reqwest::cookie::Jar::default();
            let browsers = find_cookies()
                .expect("reading cookies from browser");
            let targets = browsers.iter()
                .filter(|b| b.browser == wanted);
            let mut targets_found = false;
            for b in targets {
                targets_found = true;
                for c in &b.cookies {
                    let set_cookie = c.get_set_cookie_header();
                    if let Ok(url) = reqwest::Url::parse(&c.get_url()) {
                        jar.add_cookie_str(&set_cookie, &url);
                    }
                }
            }
            if targets_found {
                cb = cb.cookie_store(true).cookie_provider(Arc::new(jar));
            } else {
                eprintln!("Can't access cookies from {browser}.");
                eprintln!("On this computer, cookies are available from the following browsers:");
                for b in browsers.iter() {
                    eprintln!("  {:?} ({} cookies)", b.browser, b.cookies.len());
                }
            }
        } else {
            eprintln!("Ignoring unknown browser {browser}. Try one of {known_browser_names}.");
        }
    }
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
        .expect("creating HTTP client");
    let url = matches.get_one::<String>("url").unwrap();
    let mut dl = DashDownloader::new(url)
        .with_http_client(client);
    if !matches.get_flag("no-progress") && !matches.get_flag("quiet") {
        dl = dl.add_progress_observer(Arc::new(DownloadProgressBar::new()));
    }
    if let Some(seconds) = matches.get_one::<u8>("sleep-requests") {
        dl = dl.sleep_between_requests(*seconds);
    }
    if let Some(limit) = matches.get_one::<String>("limit-rate") {
        // We allow k, M, G, T suffixes, as per 100k, 1M, 0.4G
        if let Ok(np) = limit.parse::<NumberPrefix<f64>>() {
            let bps = match np {
                NumberPrefix::Standalone(bps) => bps,
                NumberPrefix::Prefixed(pfx, n) => match pfx {
                    Prefix::Kilo => n * 1024.0,
                    Prefix::Mega => n * 1024.0 * 1024.0,
                    Prefix::Giga => n * 1024.0 * 1024.0 * 1024.0,
                    Prefix::Tera => n * 1024.0 * 1024.0 * 1024.0 * 1024.0,
                    _ => {
                        eprintln!("Ignoring unrecognized suffix on limit-rate");
                        0.0
                    },
                },
            };
            if bps > 0.0 {
                dl = dl.with_rate_limit(bps as u64);
            } else {
                eprintln!("Ignoring negative value for limit-rate");
            }
        } else {
            eprintln!("Ignoring badly formed value for limit-rate");
        }
    }
    if let Some(count) = matches.get_one::<u32>("max-error-count") {
        dl = dl.max_error_count(*count);
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
    if let Some(fragments_dir) = matches.get_one::<String>("save-fragments") {
        dl = dl.save_fragments_to(Path::new(fragments_dir));
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
    if let Some(path) = matches.get_one::<String>("mp4box-location") {
        dl = dl.with_mp4box(path);
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
        if let Err(e) = dl.download_to(out).await {
            eprintln!("Download error: {e}");
        }
    } else {
        match dl.download().await {
            Ok(out) => println!("Downloaded DASH content to {out:?}"),
            Err(e) => {
                eprintln!("Download error: {e}");
                std::process::exit(2);
            },
        }
    }
    std::process::exit(0)
}
