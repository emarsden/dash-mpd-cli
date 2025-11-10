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
use url::Url;
use fs_err as fs;
use reqwest::header;
use clap::{Arg, ArgAction, ValueHint};
use number_prefix::{NumberPrefix, Prefix};
use indicatif::{ProgressBar, ProgressStyle};
use anyhow::{Result, Context};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::prelude::*;
use tracing::{info, warn, error, Level};
use dash_mpd::fetch::DashDownloader;
use dash_mpd::fetch::ProgressObserver;
#[cfg(feature = "cookies")]
use strum::IntoEnumIterator;

#[cfg(feature = "cookies")]
use decrypt_cookies::{Browser, ChromiumBuilder, FirefoxBuilder};



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


// Check whether a newer release is available on GitHub.
async fn check_newer_version() -> Result<()> {
    use versions::Versioning;

    let api = "https://api.github.com/repos/emarsden/dash-mpd-cli/releases/latest";
    let gh = reqwest::Client::builder()
        .gzip(true)
        .build()?
        .get(api)
        .header("Accept", "application/vnd.github+json")
        .header("User-agent", concat!("dash-mpd-cli/", env!("CARGO_PKG_VERSION")))
        .send().await?
        .json::<serde_json::Value>().await?;
    if let Some(gh_release) = gh["name"].as_str() {
        if let Some(gh_version) = Versioning::new(gh_release) {
            if let Some(this_version) = Versioning::new(env!("CARGO_PKG_VERSION")) {
                if gh_version > this_version {
                    info!("dash-mpd-cli {}", env!("CARGO_PKG_VERSION"));
                    info!("A newer version ({gh_release}) is available from https://github.com/emarsden/dash-mpd-cli.");
                }
            }
        }
    }
    Ok(())
}


#[tokio::main]
async fn main () -> Result<()> {
    let time_fmt = time::format_description::parse("[hour]:[minute]:[second]").unwrap();
    let time_offset = time::UtcOffset::current_local_offset()
        .unwrap_or(time::UtcOffset::UTC);
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(time_offset, time_fmt);
    // Logs of level >= INFO go to stdout, otherwise (warnings and errors) to stderr.
    let stderr = std::io::stderr.with_max_level(Level::WARN);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .map_writer(move |w| stderr.or_else(w))
        .compact()
        .with_target(false)
        .with_timer(timer);
    let filter_layer = EnvFilter::try_from_default_env()
        // The sqlx crate is used by the decrypt-cookies crate
        .or_else(|_| EnvFilter::try_new("info,reqwest=warn,hyper=warn,h2=warn,sqlx=warn"))
        .expect("initializing logging");
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    #[allow(unused_mut)]
    let mut clap = clap::Command::new("dash-mpd-cli")
        .about("Download content from an MPEG-DASH streaming media manifest.")
        .version(clap::crate_version!())
        .arg(Arg::new("user-agent")
             .long("user-agent")
             .short('U')
             .num_args(1))
        .arg(Arg::new("proxy")
             .long("proxy")
             .value_name("URL")
             .num_args(1)
             .help("URL of Socks or HTTP proxy (e.g. https://example.net/ or socks5://example.net/)."))
        .arg(Arg::new("no-proxy")
             .long("no-proxy")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .conflicts_with("proxy")
             .help("Disable use of Socks or HTTP proxy even if related environment variables are set."))
        .arg(Arg::new("auth-username")
             .long("auth-username")
             .value_name("USER")
             .help("Username to use for authentication with the server(s) hosting the DASH manifest and the media segments (HTTP Basic authentication only)."))
        .arg(Arg::new("auth-password")
             .long("auth-password")
             .value_name("PASSWORD")
             .help("Password to use for authentication with the server(s) hosting the DASH manifest and the media segments (HTTP Basic authentication only)."))
        .arg(Arg::new("auth-bearer")
             .long("auth-bearer")
             .value_name("TOKEN")
             .help("Token to use for authentication with the server(s) hosting the DASH manifest and the media segments, when HTTP Bearer authentication is required."))
        .arg(Arg::new("timeout")
             .long("timeout")
             .value_name("SECONDS")
             .num_args(1)
             .help("Timeout for network requests (from the start to the end of the request), in seconds."))
        .arg(Arg::new("sleep-requests")
             .long("sleep-requests")
             .value_name("SECONDS")
             .num_args(1)
             .value_parser(clap::value_parser!(u8))
             .help("Number of seconds to sleep between network requests (default 0)."))
        .arg(Arg::new("enable-live-streams")
             .long("enable-live-streams")
             .num_args(0)
             .help("Attempt to download from a live media stream (dynamic MPD manifest). Downloading from a genuinely live stream won't work well, because we don't implement the clock-related throttling needed to only download media segments when they become available. However, some media sources publish pseudo-live streams where all media segments are in fact available, which we will be able to download. You might also have some success in combination with the --sleep-requests argument."))
        .arg(Arg::new("force-duration")
             .long("force-duration")
             .value_name("SECONDS")
             .num_args(1)
             .value_parser(clap::value_parser!(f64))
             .help("Specify a number of seconds (possibly floating point) to download from the media stream. This may be necessary to download from a live stream, where the duration is often not specified in the DASH manifest. It may also be used to download only the first part of a static stream."))
        .arg(Arg::new("base-url")
            .long("base-url")
            .value_name("URL")
            .num_args(1)
            .help("Base URL to use for all segment downloads. This overrides any BaseURL element in the MPD."))
        .arg(Arg::new("limit-rate")
             .long("limit-rate")
             .short('r')
             .value_name("RATE")
             .num_args(1)
             .help("Maximum network bandwidth in octets per second (default no limit), e.g. 200K, 1M."))
        .arg(Arg::new("fragment-retries")
             .long("fragment-retries")
             .value_name("COUNT")
             .num_args(1)
             .value_parser(clap::value_parser!(u32))
             .help("Number of times to retry fragment network requests on error.")
             .long_help("Maximum number of non-transient network errors to ignore for each media framgent (default is 10)."))
        .arg(Arg::new("max-error-count")
             .long("max-error-count")
             .value_name("COUNT")
             .num_args(1)
             .value_parser(clap::value_parser!(u32))
             .help("Abort after COUNT non-transient network errors.")
             .long_help("Maximum number of non-transient network errors that should be ignored before a download is aborted (default is 30)."))
        .arg(Arg::new("source-address")
             .long("source-address")
             .num_args(1)
	     .long_help("Source IP address to use for network requests, either IPv4 or IPv6. Network requests will be made using the version of this IP address (e.g. using an IPv6 source-address will select IPv6 network traffic)."))
        .arg(Arg::new("add-root-certificate")
             .long("add-root-certificate")
             .value_name("CERT")
             .num_args(1)
             .value_hint(ValueHint::FilePath)
             .help("Add a root certificate (in PEM format) to be used when verifying TLS network connections."))
        .arg(Arg::new("client-identity-certificate")
             .long("client-identity-certificate")
             .value_name("CERT")
             .num_args(1)
             .value_hint(ValueHint::FilePath)
             .help("Client private key and certificate (in PEM format) to be used when authenticating TLS network connections."))
        .arg(Arg::new("prefer-video-width")
             .long("prefer-video-width")
             .value_name("WIDTH")
             .value_parser(clap::value_parser!(u64))
             .num_args(1)
             .help("When multiple video streams are available, choose that with horizontal resolution closest to WIDTH."))
        .arg(Arg::new("prefer-video-height")
             .long("prefer-video-height")
             .value_name("HEIGHT")
             .value_parser(clap::value_parser!(u64))
             .num_args(1)
             .help("When multiple video streams are available, choose that with vertical resolution closest to HEIGHT."))
        .arg(Arg::new("quality")
             .long("quality")
             .num_args(1)
             .value_parser(["best", "intermediate", "worst"])
             .help("Prefer best quality (and highest bandwidth) representation, or lowest quality."))
        .arg(Arg::new("prefer-language")
             .long("prefer-language")
             .value_name("LANG")
             .num_args(1)
             .long_help("Preferred language when multiple audio streams with different languages are available. Must be in RFC 5646 format (e.g. fr or en-AU). If a preference is not specified and multiple audio streams are present, the first one listed in the DASH manifest will be downloaded."))
        .arg(Arg::new("xslt-stylesheet")
             .long("xslt-stylesheet")
             .value_name("STYLESHEET")
             .action(ArgAction::Append)
             .num_args(1)
             .value_hint(ValueHint::FilePath)
             .long_help("XSLT stylesheet with rewrite rules to be applied to the manifest before downloading media content. Stylesheets are applied using the xsltproc commandline application, which implements XSLT 1.0. You can use this option multiple times. This option is currently experimental."))
        .arg(Arg::new("drop-elements")
             .long("drop-elements")
             .value_name("XPATH")
             .action(ArgAction::Append)
             .num_args(1)
             .long_help("XML elements in the MPD manifest that match this XPATH expression will be removed before downloading proceeds. You can use this option multiple times. This option is currently experimental."))
        .arg(Arg::new("minimum-period-duration")
             .long("minimum-period-duration")
             .value_name("SECONDS")
             .value_parser(clap::value_parser!(u64))
             .num_args(1)
             .help("Do not download periods whose duration is less than this value."))
        .arg(Arg::new("video-only")
             .long("video-only")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .conflicts_with("audio-only")
             .help("If media stream has separate audio and video streams, only download the video stream."))
        .arg(Arg::new("audio-only")
             .long("audio-only")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .conflicts_with("video-only")
             .help("If media stream has separate audio and video streams, only download the audio stream."))
        .arg(Arg::new("simulate")
             .long("simulate")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .conflicts_with("write-subs")
             .conflicts_with("keep-video")
             .conflicts_with("keep-audio")
             .help("Download the manifest and print diagnostic information, but do not download audio, video or subtitle content, and write nothing to disk."))
        .arg(Arg::new("write-subs")
             .long("write-subs")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .help("Download and save subtitle file, if subtitles are available."))
        .arg(Arg::new("keep-video")
             .long("keep-video")
             .value_name("VIDEO-PATH")
             .num_args(1)
             .value_hint(ValueHint::FilePath)
             .help("Keep video stream in file specified by VIDEO-PATH."))
        .arg(Arg::new("keep-audio")
             .long("keep-audio")
             .value_name("AUDIO-PATH")
             .num_args(1)
             .value_hint(ValueHint::FilePath)
             .help("Keep audio stream (if audio is available as a separate media stream) in file specified by AUDIO-PATH."))
        .arg(Arg::new("no-period-concatenation")
             .long("no-period-concatenation")
             .num_args(0)
             .action(ArgAction::SetTrue)
             .help("Never attempt to concatenate media from different Periods (keep one output file per Period)."))
        .arg(Arg::new("muxer-preference")
             .long("muxer-preference")
             .value_name("CONTAINER:ORDERING")
             .num_args(1)
             .action(ArgAction::Append)
             .help("When muxing into CONTAINER, try muxing applications in order ORDERING. You can use this option multiple times."))
        .arg(Arg::new("concat-preference")
             .long("concat-preference")
             .value_name("CONTAINER:ORDERING")
             .num_args(1)
             .action(ArgAction::Append)
             .help("When concatenating media streams into CONTAINER, try concat helper applications in order ORDERING. You can use this option multiple times."))
        .arg(Arg::new("role-preference")
            .long("role-preference")
            .value_name("ORDERING")
            .num_args(1)
            .action(ArgAction::Append)
            .help("Preference order for streams based on the value of the Role element in an AdaptationSet. Streaming services sometimes publish additional streams marked with roles such as alternate or supplementary, in addition to the main stream which is generalled labelled main. A value such as \"alternate,main\" means to download the alternate stream instead of the main stream (the default ordering will prefer the stream that is labelled as \"main\"). The role preference is applied after any language preference that is specified and before any specified width/height/quality preference."))
        .arg(Arg::new("key")
             .long("key")
             .value_name("KID:KEY")
             .num_args(1)
             .action(ArgAction::Append)
             .long_help("Use KID:KEY to decrypt encrypted media streams. KID should be either a track id in decimal (e.g. 1), or a 128-bit keyid (32 hexadecimal characters). KEY should be 32 hexadecimal characters. Example: --key eb676abbcb345e96bbcf616630f1a3da:100b6c20940f779a4589152b57d2dacb. You can use this option multiple times."))
        .arg(Arg::new("decryption-application")
             .long("decryption-application")
             .value_name("APP")
             .num_args(1)
             .value_parser(["mp4decrypt", "shaka"])
             .help("Application to use to decrypt encrypted media streams (either mp4decrypt or shaka)."))
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
             .help("Don't check the content-type of media fragments (may be required for some poorly configured servers)."))
        .arg(Arg::new("add-header")
             .long("add-header")
             .value_name("NAME:VALUE")
             .num_args(1)
             .action(ArgAction::Append)
             .long_help("Add a custom HTTP header and its value, separated by a colon ':'. You can use this option multiple times."))
        .arg(Arg::new("header")
             .long("header")
             .short('H')
             .value_name("HEADER")
             .num_args(1)
             .action(ArgAction::Append)
             .long_help("Add a custom HTTP header, in cURL-compatible format. You can use this option multiple times."))
        .arg(Arg::new("referer")
             .long("referer")
             .alias("referrer")
             .value_name("URL")
             .num_args(1)
             .help("Specify content of Referer HTTP header."))
        .arg(Arg::new("quiet")
             .short('q')
             .long("quiet")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .conflicts_with("verbose"))
        .arg(Arg::new("verbose")
             .short('v')
             .long("verbose")
             .action(ArgAction::Count)
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
        .arg(Arg::new("no-version-check")
             .long("no-version-check")
             .action(ArgAction::SetTrue)
             .num_args(0)
             .help("Disable the check for availability of a more recent version on startup."))
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
        .arg(Arg::new("mp4decrypt-location")
             .long("mp4decrypt-location")
             .value_name("PATH")
             .value_hint(ValueHint::ExecutablePath)
             .num_args(1)
             .help("Path to the mp4decrypt binary (necessary if not located in your PATH)."))
        .arg(Arg::new("shaka-packager-location")
             .long("shaka-packager-location")
             .value_name("PATH")
             .value_hint(ValueHint::ExecutablePath)
             .num_args(1)
             .help("Path to the shaka-packager binary (necessary if not located in your PATH)."))
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
             .help("URL of the DASH manifest to retrieve."));
    #[cfg(feature = "cookies")] {
        clap = clap
            .arg(Arg::new("cookies-from-browser")
                 .long("cookies-from-browser")
                 .value_name("BROWSER")
                 .num_args(1)
                 .help("Load cookies from BROWSER (see --list-cookie-sources for possible browsers)."))
            .arg(Arg::new("list-cookie-sources")
                 .long("list-cookie-sources")
                 .action(ArgAction::SetTrue)
                 .num_args(0)
                 .exclusive(true)
                 .help("Show valid values for BROWSER argument to --cookies-from-browser on this computer, then exit."));
    }
    // TODO: add --abort-on-error
    // TODO: add --mtime arg (Last-modified header)
    let matches = clap.get_matches();

    if ! matches.get_flag("no-version-check") {
        let _ = check_newer_version().await;
    }
    #[cfg(feature = "cookies")]
    if matches.get_flag("list-cookie-sources") {
        info!("On this computer, cookies are available from the following browsers:");
        for browser in Browser::iter() {
            if browser.is_chromium_base() {
                if let Ok(browser) = ChromiumBuilder::new(browser).build().await {
                    if let Ok(cookies) = browser.get_cookies_all().await {
                        info!("  {:?} ({} cookies)", browser.browser(), cookies.len());
                    }
                }
            }
            if browser.is_firefox_base() {
                if let Ok(browser) = FirefoxBuilder::new(browser).build().await {
                    if let Ok(cookies) = browser.get_cookies_all().await {
                        info!("  {:?} ({} cookies)", browser.browser(), cookies.len());
                    }
                }
            }
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
        .cookie_store(true)
        .gzip(true);
    #[cfg(feature = "cookies")]
    if let Some(browser_name) = matches.get_one::<String>("cookies-from-browser") {
        let browser = Browser::from_str(browser_name)
            .unwrap_or_else(|_| panic!("unknown browser {browser_name} in --cookies-from-browser"));
        if browser.is_chromium_base() {
            let browser = ChromiumBuilder::new(browser).build().await
                .unwrap_or_else(|_| panic!("couldn't extract cookies from browser {browser_name}"));
            let cookies = browser.get_cookies_all().await
                .unwrap_or_else(|_| panic!("couldn't extract cookies from browser {browser_name}"));
            if verbosity > 1 {
                info!("  Extracted {} cookies from browser {}", cookies.len(), browser_name);
            }
            let jar: reqwest::cookie::Jar = cookies.into_iter().collect();
            cb = cb.cookie_store(true).cookie_provider(Arc::new(jar));
        }
        if browser.is_firefox_base() {
            let browser = FirefoxBuilder::new(browser).build().await
                .unwrap_or_else(|_| panic!("couldn't extract cookies from browser {browser_name}"));
            let cookies = browser.get_cookies_all().await
                .unwrap_or_else(|_| panic!("couldn't extract cookies from browser {browser_name}"));
            if verbosity > 1 {
                info!("  Extracted {} cookies from browser {}", cookies.len(), browser_name);
            }
            let jar: reqwest::cookie::Jar = cookies.into_iter().collect();
            cb = cb.cookie_store(true).cookie_provider(Arc::new(jar));
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
    if matches.get_flag("no-proxy") {
        cb = cb.no_proxy();
    }
    if let Some(src) = matches.get_one::<String>("source-address") {
       if let Ok(local_addr) = IpAddr::from_str(src) {
          cb = cb.local_address(local_addr);
       } else {
          warn!("Ignoring invalid argument to --source-address: {src}");
       }
    }
    if let Some(seconds) = matches.get_one::<String>("timeout") {
        if let Ok(secs) = seconds.parse::<u64>() {
            cb = cb.timeout(Duration::new(secs, 0));
        } else {
            warn!("Ignoring invalid value for --timeout: {seconds}");
        }
    } else {
        cb = cb.timeout(Duration::new(30, 0));
    }
    let mut headers = HashMap::new();
    if let Some(hvs) = matches.get_many::<String>("header") {
        for hv in hvs.collect::<Vec<_>>() {
            if let Some((h, v)) = hv.split_once(':') {
                headers.insert(h.to_string(), v.trim_start().to_string());
            } else {
                warn!("Ignoring badly formed header:value argument to --header");
            }
        }
    }
    if let Some(hvs) = matches.get_many::<String>("add-header") {
        for hv in hvs.collect::<Vec<_>>() {
            if let Some((h, v)) = hv.split_once(':') {
                headers.insert(h.to_string(), v.to_string());
            } else {
                warn!("Ignoring badly formed header:value argument to --add-header");
            }
        }
    }
    if !headers.is_empty() {
        let hmap: header::HeaderMap = (&headers).try_into()
            .expect("valid HTTP headers");
        cb = cb.default_headers(hmap);
    }
    if let Some(rcs) = matches.get_many::<String>("add-root-certificate") {
        for rc in rcs {
            match fs::read(rc) {
                Ok(pem) => {
                    match reqwest::Certificate::from_pem(&pem) {
                        Ok(cert) => {
                            cb = cb.add_root_certificate(cert);
                        },
                        Err(e) => {
                            error!("Can't decode root certificate: {e}");
                            std::process::exit(6);
                        },
                    }
                },
                Err(e) => {
                    error!("Can't read root certificate: {e}");
                    std::process::exit(5);
                },
            }
        }
    }
    if let Some(cc) = matches.get_one::<String>("client-identity-certificate") {
        match fs::read(cc) {
            Ok(pem) => {
                match reqwest::Identity::from_pem(&pem) {
                    Ok(id) => {
                        cb = cb.identity(id);
                    },
                    Err(e) => {
                        error!("Can't decode client certificate: {e}");
                        std::process::exit(8);
                    },
                }
            },
            Err(e) => {
                error!("Can't read client certificate: {e}");
                std::process::exit(7);
            },
        }
    }
    let client = cb.build()
        .expect("creating HTTP client");
    let url = matches.get_one::<String>("url").unwrap();
    let mut dl = DashDownloader::new(url)
        .with_http_client(client);
    if let Some(url) = matches.get_one::<String>("referer") {
        dl = dl.with_referer(url.to_string());
    }
    if !matches.get_flag("no-progress") && !matches.get_flag("quiet") {
        dl = dl.add_progress_observer(Arc::new(DownloadProgressBar::new()));
    }
    if let Some(seconds) = matches.get_one::<u8>("sleep-requests") {
        dl = dl.sleep_between_requests(*seconds);
    }
    if matches.get_flag("enable-live-streams") {
        dl = dl.allow_live_streams(true);
    }
    if let Some(seconds) = matches.get_one::<f64>("force-duration") {
        dl = dl.force_duration(*seconds);
    }
    if let Some(bu) = matches.get_one::<String>("base-url") {
        if let Err(e) = Url::parse(bu) {
            error!("Invalid URL for --base-url: {e}");
            std::process::exit(9);
        }
        dl = dl.with_base_url(String::from(bu));
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
                        warn!("Ignoring unrecognized suffix on limit-rate");
                        0.0
                    },
                },
            };
            if bps > 0.0 {
                dl = dl.with_rate_limit(bps as u64);
            } else {
                warn!("Ignoring negative value for limit-rate");
            }
        } else {
            warn!("Ignoring invalid value for limit-rate");
        }
    }
    if let Some(count) = matches.get_one::<u32>("fragment-retries") {
        dl = dl.fragment_retry_count(*count);
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
    if matches.get_flag("simulate") {
        dl = dl.fetch_audio(false)
            .fetch_video(false)
            .fetch_subtitles(false);
    }
    if let Some(path) = matches.get_one::<String>("keep-video") {
        dl = dl.keep_video_as(path);
    }
    if let Some(path) = matches.get_one::<String>("keep-audio") {
        dl = dl.keep_audio_as(path);
    }
    if matches.get_flag("no-period-concatenation") {
        dl = dl.concatenate_periods(false);
    } else {
        dl = dl.concatenate_periods(true);
    }
    if let Some(mps) = matches.get_many::<String>("muxer-preference") {
        for mp in mps.collect::<Vec<_>>() {
            if let Some((container, ordering)) = mp.split_once(':') {
                dl = dl.with_muxer_preference(container, ordering);
            } else {
                warn!("Ignoring badly formatted container:ordering argument to --muxer-preference");
            }
        }
    }
    if let Some(mps) = matches.get_many::<String>("concat-preference") {
        for mp in mps.collect::<Vec<_>>() {
            if let Some((container, ordering)) = mp.split_once(':') {
                dl = dl.with_concat_preference(container, ordering);
            } else {
                warn!("Ignoring badly formatted container:ordering argument to --concat-preference");
            }
        }
    }
    if let Some(rp) = matches.get_one::<String>("role-preference") {
        let ordering: Vec<String> = rp.split(',')
            .map(str::to_string)
            .collect();
        if !ordering.is_empty() {
            dl = dl.prefer_roles(ordering);
        } else {
            warn!("Ignoring badly formatted role1,role2,role3 argument to --role-preference");
        }
    }
    if let Some(kvs) = matches.get_many::<String>("key") {
        for kv in kvs.collect::<Vec<_>>() {
            if let Some((kid, key)) = kv.split_once(':') {
                if key.len() != 32 {
                    warn!("Ignoring invalid format for KEY (should be 32 hex digits)");
                } else {
                    dl = dl.add_decryption_key(String::from(kid), String::from(key));
                }
            } else {
                warn!("Ignoring badly formed KID:KEY argument to --key");
            }
        }
    }
    if let Some(app) = matches.get_one::<String>("decryption-application") {
        dl = dl.with_decryptor_preference(app);
    }
    if let Some(fragments_dir) = matches.get_one::<String>("save-fragments") {
        dl = dl.save_fragments_to(Path::new(fragments_dir));
    }
    if matches.get_flag("write-subs") {
        dl = dl.fetch_subtitles(true);
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
    if let Some(path) = matches.get_one::<String>("mp4decrypt-location") {
        dl = dl.with_mp4decrypt(path);
    }
    if let Some(path) = matches.get_one::<String>("shaka-packager-location") {
        dl = dl.with_shaka_packager(path);
    }
    if let Some(w) = matches.get_one::<u64>("prefer-video-width") {
        dl = dl.prefer_video_width(*w);
    }
    if let Some(h) = matches.get_one::<u64>("prefer-video-height") {
        dl = dl.prefer_video_height(*h);
    }
    // It's possible to specify both prefer-video-width/height and quality. The former is not
    // relevant concerning the audio stream, where the quality preference will be used. For the
    // choice of video stream if both are specified, preference will be given to the preferred
    // width, then height, then quality.
    if let Some(q) = matches.get_one::<String>("quality") {
        // DashDownloader defaults to worst quality
        if q.eq("best") {
            dl = dl.best_quality();
        } else if q.eq("intermediate") {
            dl = dl.intermediate_quality();
        }
    }
    if let Some(lang) = matches.get_one::<String>("prefer-language") {
        dl = dl.prefer_language(lang.to_string());
    }
    if let Some(stylesheets) = matches.get_many::<String>("xslt-stylesheet") {
        for stylesheet in stylesheets {
            dl = dl.with_xslt_stylesheet(stylesheet);
        }
    }
    if let Some(xpaths) = matches.get_many::<String>("drop-elements") {
        for xpath in xpaths {
            let xslt = format!(r#"<?xml version="1.0" encoding="utf-8"?>
  <xsl:stylesheet version="1.0"
     xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
     xmlns:mpd="urn:mpeg:dash:schema:mpd:2011">
  <xsl:template match="@*|node()">
    <xsl:copy><xsl:apply-templates select="@*|node()"/></xsl:copy></xsl:template>
  <xsl:template match="{xpath}" />
</xsl:stylesheet>"#);
            let stylesheet = tempfile::Builder::new()
                .suffix(".xslt")
                .rand_bytes(7)
                .tempfile()
                .context("creating temporary XSLT stylesheet")?;
            fs::write(&stylesheet, xslt)
                .context("writing XSLT to temporary stylesheet file")?;
            let (_, stylesheet_path) = stylesheet.keep()?;
            dl = dl.with_xslt_stylesheet(stylesheet_path);
        }
    }
    if let Some(secs) = matches.get_one::<u64>("minimum-period-duration") {
        dl = dl.minimum_period_duration(Duration::from_secs(*secs));
    }
    if let Some(user) = matches.get_one::<String>("auth-username") {
        if let Some(password) = matches.get_one::<String>("auth-password") {
            dl = dl.with_authentication(user.to_string(), password.to_string());
        }
    }
    if let Some(token) = matches.get_one::<String>("auth-bearer") {
        dl = dl.with_auth_bearer(token.to_string());
    }
    dl = dl.verbosity(verbosity);
    if let Some(out) = matches.get_one::<String>("output-file") {
        if let Err(e) = dl.download_to(out).await {
            error!("Download failed");
            std::process::exit(2);
        }
    } else {
        match dl.download().await {
            Ok(out) => {
                if !matches.get_flag("simulate") {
                    info!("Downloaded DASH content to {out:?}");
                }
            },
            Err(e) => {
                error!("Download failed");
                if e.to_string().contains("how to download dynamic MPD") {
                    info!("See the help for the --enable-live-streams commandline option.");
                }
                // TODO we could return different exit codes for different error types
                std::process::exit(2);
            },
        }
    }
    std::process::exit(0)
}
