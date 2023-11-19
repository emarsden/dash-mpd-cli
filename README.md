# dash-mpd-cli

A commandline application for downloading media content from a DASH MPD file, as used for on-demand
replay of TV content and video streaming services like YouTube.

[![Crates.io](https://img.shields.io/crates/v/dash-mpd-cli)](https://crates.io/crates/dash-mpd-cli)
[![CI](https://github.com/emarsden/dash-mpd-cli/workflows/build/badge.svg)](https://github.com/emarsden/dash-mpd-cli/actions/)
[![Dependency status](https://deps.rs/repo/github/emarsden/dash-mpd-cli/status.svg)](https://deps.rs/repo/github/emarsden/dash-mpd-cli)
[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)

![Terminal capture](https://risk-engineering.org/emarsden/dash-mpd-cli/terminal-capture.svg)


[DASH](https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP) (dynamic adaptive
streaming over HTTP), also called MPEG-DASH, is a technology used for media streaming over the web,
commonly used for video on demand (VOD) and “replay/catch-up TV” services. The Media Presentation
Description (MPD) is an XML document that lists the resources (manifest or “playlist”) forming a
streaming service. A DASH client uses the manifest to determine which assets to request in order to
perform adaptive streaming of the content. DASH MPD manifests can be used with content using
different codecs (including H264, HEVC, AV1, AAC, VP9, MP4A, MP3) and containers (MP4, WebM,
Matroska, AVI). There is a good explanation of adaptive bitrate video streaming at
[howvideo.works](https://howvideo.works/#dash).

This commandline application allows you to download content (audio or video) described by an MPD
manifest. This involves selecting the alternative with the most appropriate encoding (in terms of
bitrate, codec, etc.), fetching segments of the content using HTTP or HTTPS requests and muxing
audio and video segments together. There is also support for downloading subtitles (mostly WebVTT,
TTML, SRT, tx3g and SMIL formats, with some support for wvtt format).

This application builds on the [dash-mpd](https://crates.io/crates/dash-mpd) crate.


## Features

The following features are supported: 

- Multi-period content. The media in the different streams will be saved in a single media container
  if the formats are compatible (same resolution, codecs, bitrate and so on) and the
  `--no-period-concatenation` commandline option is not provided, and otherwise in separate media
  containers.

- The application can download content available over HTTP, HTTPS and HTTP/2. Network bandwidth can
  be throttled (see the `--limit-rate` commandline argument).

- Support for SOCKS and HTTP proxies, via the `--proxy` commandline argument. The following
  environment variables can also be used to specify the proxy at a system level: `HTTP_PROXY` or
  `http_proxy` for HTTP connections, `HTTPS_PROXY` or `https_proxy` for HTTPS connections, and
  `ALL_PROXY` or `all_proxy` for all connection types. The system proxy can be disabled using the
  `--no-proxy` commandline argument.

- Support for HTTP Basic authentication (see the `--auth-username` and `--auth-password` commandline
  arguments) and for Bearer authentation (see the `--auth-bearer` commandline argument). This
  authentication information is sent both to the server which hosts the DASH manifest, and to the
  server that hosts the media segments (the latter often being a CDN).

- Subtitles: download support for WebVTT, TTML, SRT, tx3g and SMIL streams, as well as some support
  for the wvtt format. We support both subtitles published as a complete file and segmented
  subtitles made available in media fragments.

- The application can read cookies from the Firefox, Chromium, Chrome, ChromeBeta, Safari and Edge
  browsers on Linux, Windows and MacOS, thanks to the
  [bench_scraper](https://crates.io/crates/bench_scraper) crate. See the `--cookies-from-browser`
  commandline argument.
  Browsers that support multiple profiles will have all their profiles scraped for cookies.

- XLink elements (only with actuate=onLoad semantics), including resolve-to-zero.

- All forms of segment index info: SegmentBase@indexRange, SegmentTimeline,
  SegmentTemplate@duration, SegmentTemplate@index, SegmentList.

- Media containers of types supported by mkvmerge, ffmpeg, VLC or MP4Box (this includes ISO-BMFF /
  CMAF / MP4, Matroska, WebM, MPEG-2 TS, AVI), and all the codecs supported by these applications.

- Support for decrypting media streams that use ContentProtection (DRM). This requires either the
  `mp4decrypt` or `shaka-packager` commandline application to be installed. mp4decrypt is available
  from the [Bento4 suite](https://github.com/axiomatic-systems/Bento4/) ([binaries are
  available](https://www.bento4.com/downloads/) for common platforms), and [shaka-packager
  binaries](https://github.com/shaka-project/shaka-packager) are available from Google for common
  platforms (see the Releases section on their GitHub page). See the `--key` commandline argument to
  specify a decryption key (can be used several times if different keys are used for different media
  streams). See the `--decryption-application` commandline argument to specify which decryption
  application to use. Shaka packager is able to decrypt more types of media streams (including in
  particular WebM containers and more encryption formats), whereas mp4decrypt mostly works with MPEG
  Common Encryption.

- In practice, all features used by real streaming services and on-demand TV. Our test suite
  includes test streams published by industry groups such as HbbTV and the DASH Industry Forum, and
  comprises a wide variety of DASH streams using different publishing software, including GPAC (used
  by Netflix and other services), Amazon MediaTailor, Google’s Shaka packager, Microsoft’s Azure
  Media Services, and Unified Streaming. Test content is served by different CDNs including Akamai
  and various telecom providers.

The following are not supported: 

- **Live streams** (dynamic MPD manifests), that are used for live streaming/OTT TV are not really
  supported. This is because we don’t implement the clock-related throttling that is needed to only
  download media segments when they become available. However, some media sources publish
  “pseudo-live” streams where all media segments are in fact available; they simply don’t update the
  manifest once the live is complete. We are able to download these streams using the
  `--enable-live-streams` commandline argument. You might also have some success with a live stream
  in combination with the `--sleep-requests` commandline argument. The VLC application is a better
  choice for watching live streams.

- XLink elements with actuate=onRequest semantics.


## Installation

**Binary releases** are [available on GitHub](https://github.com/emarsden/dash-mpd-cli/releases) for
GNU/Linux on AMD64 (statically linked against Musl Libc to avoid glibc versioning problems),
Microsoft Windows on AMD64 and MacOS on aarch64 (“Apple Silicon”) and AMD64. These are built
automatically on the GitHub continuous integration infrastructure.

You can also **build from source** using an [installed Rust development
environment](https://www.rust-lang.org/tools/install):

```shell
cargo install dash-mpd-cli
```

This installs the binary to your installation root's `bin` directory, which is typically
`$HOME/.cargo/bin`.

You should also install the following **dependencies**:

- the mkvmerge commandline utility from the [MkvToolnix](https://mkvtoolnix.download/) suite, if you
  download to the Matroska container format (`.mkv` filename extension). mkvmerge is used as a
  subprocess for muxing (combining) audio and video streams. See the `--mkvmerge-location`
  commandline argument if it’s not installed in a standard location (not on your PATH).

- [ffmpeg](https://ffmpeg.org/) or [vlc](https://www.videolan.org/vlc/) to download to the MP4
  container format, also for muxing audio and video streams (see the `--ffmpeg-location` and
  `--vlc-location` commandline arguments if these are installed in non-standard locations). See the
  `--muxer-preference` commandline argument to specify which muxing application to prefer for
  different container types.

- the MP4Box commandline utility from the [GPAC](https://gpac.wp.imt.fr/) project, if you want to
  test the preliminary support for retrieving subtitles in wvtt format. If it's installed, MP4Box
  will be used to convert the wvtt stream to the more widely recognized SRT format. MP4Box can also
  be used for muxing audio and video streams to an MP4 container, as a fallback if ffmpeg and vlc
  are not available. See the `--mp4box-location` commandline argument if this is installed in a
  non-standard location.

- the mp4decrypt commandline application from the [Bento4
  suite](https://github.com/axiomatic-systems/Bento4/), if you need to fetch encrypted content.
  [Binaries are available](https://www.bento4.com/downloads/) for common platforms. See the
  `--mp4decrypt-location` commandline argument if this is installed in a non-standard location.

- for some types of streams that the mp4decrypt application is not able to decrypt (for example
  content in WebM containers), you should install the [Shaka packager
  application](https://github.com/shaka-project/shaka-packager) developed by Google. See the
  `--decryption-application` commandline option to specify the choice of decryption application, and
  the `--shaka-packager-location` commandline argument if it is installed in a non-standard location.


This crate is tested on the following **platforms**:

- Linux on AMD64 (x86-64) and Aarch64 architectures

- MacOS on AMD64 and Aarch64 architectures

- Microsoft Windows 10 and Windows 11 on AMD64

- Android 12 on Aarch64 via [termux](https://termux.dev/) (you'll need to install the rust, binutils
  and ffmpeg packages, and optionally the mkvtoolnix, vlc and gpac packages). You'll need to disable
  the `cookies` feature by building with `--no-default-features`.

- FreeBSD/AMD64 and OpenBSD/AMD64. You'll need to disable the `cookies` feature. Some of the
  external applications we depend on (e.g. mp4decrypt, Shaka packager) are poorly supported on OpenBSD.



## Usage

```
Download content from an MPEG-DASH streaming media manifest.

Usage: dash-mpd-cli [OPTIONS] <MPD-URL>

Arguments:
  <MPD-URL>
          URL of the DASH manifest to retrieve.

Options:
  -U, --user-agent <user-agent>
          

      --proxy <URL>
          URL of Socks or HTTP proxy (e.g. https://example.net/ or socks5://example.net/).

      --no-proxy
          Disable use of Socks or HTTP proxy even if related environment variables are set.

      --auth-username <USER>
          Username to use for authentication with the server(s) hosting the DASH manifest and the media segments (HTTP
          Basic authentication only).

      --auth-password <PASSWORD>
          Password to use for authentication with the server(s) hosting the DASH manifest and the media segments (HTTP
          Basic authentication only).

      --auth-bearer <TOKEN>
          Token to use for authentication with the server(s) hosting the DASH manifest and the media segments, when HTTP
          Bearer authentication is required.

      --timeout <SECONDS>
          Timeout for network requests (from the start to the end of the request), in seconds.

      --sleep-requests <SECONDS>
          Number of seconds to sleep between network requests (default 0).

      --enable-live-streams
          Attempt to download from a live media stream (dynamic MPD manifest). Downloading from a genuinely live stream
          won't work well, because we don't implement the clock-related throttling needed to only download media segments
          when they become available. However, some media sources publish pseudo-live streams where all media segments are
          in fact available, which we will be able to download. You might also have some success in combination with the
          --sleep-requests argument.

      --force-duration <SECONDS>
          Specify a number of seconds (possibly floating point) to download from the media stream. This may be necessary to
          download from a live stream, where the duration is often not specified in the DASH manifest. It may also be used
          to download only the first part of a static stream.

  -r, --limit-rate <RATE>
          Maximum network bandwidth in octets per second (default no limit), e.g. 200K, 1M.

      --max-error-count <COUNT>
          Maximum number of non-transient network errors that should be ignored before a download is aborted (default is
          10).

      --source-address <source-address>
          Source IP address to use for network requests, either IPv4 or IPv6. Network requests will be made using the
          version of this IP address (e.g. using an IPv6 source-address will select IPv6 network traffic).

      --add-root-certificate <CERT>
          Add a root certificate (in PEM format) to be used when verifying TLS network connections.

      --client-identity-certificate <CERT>
          Client private key and certificate (in PEM format) to be used when authenticating TLS network connections.

      --prefer-video-width <WIDTH>
          When multiple video streams are available, choose that with horizontal resolution closest to WIDTH.

      --prefer-video-height <HEIGHT>
          When multiple video streams are available, choose that with vertical resolution closest to HEIGHT.

      --quality <quality>
          Prefer best quality (and highest bandwidth) representation, or lowest quality.
          
          [possible values: best, intermediate, worst]

      --prefer-language <LANG>
          Preferred language when multiple audio streams with different languages are available. Must be in RFC 5646 format
          (e.g. fr or en-AU). If a preference is not specified and multiple audio streams are present, the first one listed
          in the DASH manifest will be downloaded.

      --xslt-stylesheet <STYLESHEET>
          XSLT stylesheet with rewrite rules to be applied to the manifest before downloading media content. Stylesheets
          are applied using the xsltproc commandline application, which implements XSLT 1.0. You can use this option
          multiple times. This option is currently experimental.

      --video-only
          If media stream has separate audio and video streams, only download the video stream.

      --audio-only
          If media stream has separate audio and video streams, only download the audio stream.

      --simulate
          Download the manifest and print diagnostic information, but do not download audio, video or subtitle content, and
          write nothing to disk.

      --write-subs
          Download and save subtitle file, if subtitles are available.

      --keep-video <VIDEO-PATH>
          Keep video stream in file specified by VIDEO-PATH.

      --keep-audio <AUDIO-PATH>
          Keep audio stream (if audio is available as a separate media stream) in file specified by AUDIO-PATH.

      --no-period-concatenation
          Never attempt to concatenate media from different Periods (keep one output file per Period).

      --muxer-preference <CONTAINER:ORDERING>
          When muxing into CONTAINER, try muxing applications in order ORDERING. You can use this option multiple times.

      --key <KID:KEY>
          Use KID:KEY to decrypt encrypted media streams. KID should be either a track id in decimal (e.g. 1), or a 128-bit
          keyid (32 hexadecimal characters). KEY should be 32 hexadecimal characters. Example: --key
          eb676abbcb345e96bbcf616630f1a3da:100b6c20940f779a4589152b57d2dacb. You can use this option multiple times.

      --decryption-application <APP>
          Application to use to decrypt encrypted media streams (either mp4decrypt or shaka).
          
          [possible values: mp4decrypt, shaka]

      --save-fragments <FRAGMENTS-DIR>
          Save media fragments to this directory (will be created if it does not exist).

      --ignore-content-type
          Don't check the content-type of media fragments (may be required for some poorly configured servers).

      --add-header <NAME:VALUE>
          Add a custom HTTP header and its value, separated by a colon ':'. You can use this option multiple times.

  -H, --header <HEADER>
          Add a custom HTTP header, in cURL-compatible format. You can use this option multiple times.

      --referer <URL>
          Specify content of Referer HTTP header.

  -q, --quiet
          

  -v, --verbose...
          Level of verbosity (can be used several times).

      --no-progress
          Disable the progress bar

      --no-xattr
          Don't record metainformation as extended attributes in the output file.

      --no-version-check
          Disable the check for availability of a more recent version on startup.

      --ffmpeg-location <PATH>
          Path to the ffmpeg binary (necessary if not located in your PATH).

      --vlc-location <PATH>
          Path to the VLC binary (necessary if not located in your PATH).

      --mkvmerge-location <PATH>
          Path to the mkvmerge binary (necessary if not located in your PATH).

      --mp4box-location <PATH>
          Path to the MP4Box binary (necessary if not located in your PATH).

      --mp4decrypt-location <PATH>
          Path to the mp4decrypt binary (necessary if not located in your PATH).

      --shaka-packager-location <PATH>
          Path to the shaka-packager binary (necessary if not located in your PATH).

  -o, --output <PATH>
          Save media content to this file.

      --cookies-from-browser <BROWSER>
          Load cookies from BROWSER (Firefox, Chrome, ChromeBeta, Chromium).

      --list-cookie-sources
          Show valid values for BROWSER argument to --cookies-from-browser on this computer, then exit.

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```


If your filesystem supports **extended attributes**, the application will save the following
metainformation in the output file:

- `user.xdg.origin.url`: the URL of the MPD manifest
- `user.dublincore.title`: the title, if specified in the manifest metainformation
- `user.dublincore.source`: the source, if specified in the manifest metainformation
- `user.dublincore.rights`: copyright information, if specified in the manifest metainformation

You can examine these attributes using `xattr -l` (you may need to install your distribution's
`xattr` package). Disable this feature using the `--no-xattr` commandline argument.


## Muxing

The underlying library `dash-mpd-rs` has two methods for muxing audio and video streams together. If
the library feature `libav` is enabled (which is not the default configuration), muxing support is
provided by ffmpeg’s libav library, via the `ac_ffmpeg` crate. Otherwise, muxing is implemented by
calling an external muxer, mkvmerge (from the [MkvToolnix](https://mkvtoolnix.download/) suite),
[ffmpeg](https://ffmpeg.org/), [vlc](https://www.videolan.org/vlc/) or
[MP4Box](https://github.com/gpac/gpac/wiki/MP4Box) as a subprocess. Note that these commandline
applications implement a number of checks and workarounds to fix invalid input streams that tend to
exist in the wild. Some of these workarounds are implemented here when using libav as a library, but
not all of them, so download support tends to be more robust with the default configuration (using
an external application as a subprocess). The `libav` feature currently only works on Linux.

The choice of external muxer depends on the filename extension of the path supplied to `--output`
or `-o` (which will be ".mp4" if you don't specify the output path explicitly):

- `.mkv`: call mkvmerge first, then if that fails call ffmpeg, then try MP4Box
- `.mp4`: call ffmpeg first, then if that fails call vlc, then try MP4Box
- `.webm`: call vlc, then if that fails ffmpeg
- other: try ffmpeg, which supports many container formats, then try MP4Box

You can specify a different order of preference for muxing applications using the
`--muxer-preference` commandline option. For example, `--muxer-preference avi:vlc,ffmpeg` means that
for an AVI media container the external muxer vlc will be tried first, then ffmpeg in case of
failure. This commandline option can be used multiple times to specify options for different
container types.



## Similar tools

Similar commandline tools that are able to download content from a DASH manifest:

- `yt-dlp <MPD-URL>`

- `N_m3u8DL-RE <MPD-URL>`

- `streamlink -o /tmp/output.mp4 <MPD-URL> worst`

- `ffmpeg -i <MPD-URL> -vcodec copy /tmp/output.mp4`

- `vlc <MPD-URL>`

- `gst-launch-1.0 playbin uri=<MPD-URL>`

However, dash-mpd-cli (this application) is able to download content from certain streams that do
not work with other applications:

- streams using xHE-AAC codecs are currently unsupported by ffmpeg, streamlink, VLC, and gstreamer
- streams in multi-period manifests
- streams using XLink elements


## Building

```
$ git clone https://github.com/emarsden/dash-mpd-cli
$ cd dash-mpd-cli
$ cargo build --release
$ target/release/dash-mpd-cli --help
```

The application can also be built statically with the musl-libc target on Linux. First install the
[MUSL C standard library](https://musl.libc.org/) on your system. Add linux-musl as a target to your
Rust toolchain, then rebuild for the relevant target:

```
$ sudo apt install musl-dev
$ rustup target add x86_64-unknown-linux-musl
$ cargo build --release --target x86_64-unknown-linux-musl
```

Static musl-libc builds don’t work with OpenSSL, which is why we disable default features on the
dash-mpd crate and build it with [rustls](https://github.com/rustls/rustls) support (a Rust TLS
stack). You may encounter some situations where rustls fails to connect (handshake errors, for
example) but other applications on your system can connect. These differences in behaviour are
typically due to different configurations for the set of root certificates. If you prefer to use
your machine’s native TLS stack, replace both instances of `rustls-tls` by `native-tls` in
`Cargo.toml` and rebuild.


## Why?

The [dash-mpd-rs](https://github.com/emarsden/dash-mpd-rs) library at the core of this application
was developed to allow the author to watch a news programme produced by a public media broadcaster
whilst at the gym. The programme is published as a DASH stream on the broadcaster’s “replay”
service, but network service at the gym is sometimes poor. First world problems!

The author is not the morality police nor a lawyer, but please note that redistributing media
content that you have not produced may, depending on the publication licence, be a breach of
intellectual property laws. Also, circumventing DRM may be prohibited in some countries.


## License

This project is licensed under the MIT license. For more information, see the `LICENSE-MIT` file.
