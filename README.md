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
commonly used for video on demand (VOD) services. The Media Presentation Description (MPD) is a
description of the resources (manifest or “playlist”) forming a streaming service, that a DASH
client uses to determine which assets to request in order to perform adaptive streaming of the
content. DASH MPD manifests can be used with content encoded in different formats and containers,
including H264/MP4, HEVC/MP4 and VP9/WebM. There is a good explanation of adaptive bitrate video
streaming at [howvideo.works](https://howvideo.works/#dash).

This commandline application allows you to download content (audio or video) described by an MPD
manifest. This involves selecting the alternative with the most appropriate encoding (in terms of
bitrate, codec, etc.), fetching segments of the content using HTTP or HTTPS requests and muxing
audio and video segments together. There is also support for downloading subtitles (mostly WebVTT,
TTML and SMIL formats, with some support for wvtt format).

This application builds on the [dash-mpd](https://crates.io/crates/dash-mpd) crate.


## Features

The following features are supported: 

- VOD (static) stream manifests (this application can't download from dynamic MPD manifests, that
  are used for live streaming and OTT television).

- Multi-period content.

- The application can download content available over HTTP, HTTPS and HTTP/2. Network bandwidth can
  be throttled (see the `--limit-rate` commandline argument).

- Support for SOCKS and HTTP proxies, via the `--proxy` commandline argument. The following
  environment variables can also be used to specify the proxy at a system level: `HTTP_PROXY` or
  `http_proxy` for HTTP connections, `HTTPS_PROXY` or `https_proxy` for HTTPS connections, and
  `ALL_PROXY` or `all_proxy` for all connection types. The system proxy can be disabled using the
  `--no-proxy` commandline argument.

- Subtitles: download support for WebVTT, TTML and SMIL streams, as well as some support for the
  wvtt format.

- The application can read cookies from the Firefox, Chromium, Chrome, ChromeBeta, Safari and Edge
  browsers on Linux, Windows and MacOS, thanks to the
  [bench_scraper](https://crates.io/crates/bench_scraper) crate. See the `--cookies-from-browser`
  commandline argument.
  Browsers that support multiple profiles will have all their profiles scraped for cookies.

- XLink elements (only with actuate=onLoad semantics), including resolve-to-zero.

- All forms of segment index info: SegmentBase@indexRange, SegmentTimeline,
  SegmentTemplate@duration, SegmentTemplate@index, SegmentList.

- Media containers of types supported by mkvmerge, ffmpeg, VLC or MP4Box (this includes ISO-BMFF /
  CMAF / MP4, Matroska, WebM, MPEG-2 TS).

- Support for decrypting media streams that use MPEG Common Encryption (cenc) ContentProtection.
  This requires the `mp4decrypt` commandline application from the [Bento4
  suite](https://github.com/axiomatic-systems/Bento4/) to be installed ([binaries are
  available](https://www.bento4.com/downloads/) for common platforms). See the
  `--key` commandline argument.

The following are not supported: 

- XLink elements with actuate=onRequest semantics



## Installation

**Binary releases** are [available on GitHub](https://github.com/emarsden/dash-mpd-cli/releases) for
GNU/Linux on AMD64 (statically linked against musl libc to avoid glibc versioning problems),
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
  commandline argument if it's not installed in a standard location.

- [ffmpeg](https://ffmpeg.org/) or [vlc](https://www.videolan.org/vlc/) to download to the MP4
  container format, also for muxing audio and video streams (see the `--ffmpeg-location` and
  `--vlc-location` commandline arguments if these are installed in non-standard locations).

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


This crate is tested on the following **platforms**:

- Linux on AMD64 (x86-64) and Aarch64 architectures

- MacOS on AMD64 and Aarch64 architectures

- Microsoft Windows 10 and Windows 11 on AMD64

- Android 12 on Aarch64 via [termux](https://termux.dev/) (you'll need to install the rust, binutils
  and ffmpeg packages, and optionally the mkvtoolnix, vlc and gpac packages). You'll need to disable
  the `cookies` feature by building with `--no-default-features`.

- FreeBSD/AMD64 and OpenBSD/AMD64. You'll need to disable the `cookies` feature.



## Usage

```
Download content from an MPEG-DASH streaming media manifest

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

      --timeout <SECONDS>
          Timeout for network requests (from the start to the end of the request), in seconds.

      --sleep-requests <SECONDS>
          Number of seconds to sleep between network requests (default 0).

  -r, --limit-rate <RATE>
          Maximum network bandwidth in octets per second (default no limit), e.g. 200K, 1M.

      --max-error-count <COUNT>
          Maximum number of non-transient network errors that should be ignored before a download is aborted (default is 10).

      --source-address <source-address>
          Source IP address to use for network requests, either IPv4 or IPv6. Network requests will be made using the version of this IP address (e.g. using an IPv6 source-address will select IPv6 network traffic).

      --add-root-certificate <CERT>
          Add a root certificate (in PEM format) to be used when verifying TLS network connections.

      --client-identity-certificate <CERT>
          Client private key and certificate (in PEM format) to be used when authenticating TLS network connections.

      --quality <quality>
          Prefer best quality (and highest bandwidth) representation, or lowest quality.
          
          [possible values: best, worst]

      --prefer-language <LANG>
          Preferred language when multiple audio streams with different languages are available. Must be in RFC 5646 format (e.g. fr or en-AU). If a preference is not specified and multiple audio streams are present, the first one listed in the DASH manifest will be downloaded.

      --video-only
          If media stream has separate audio and video streams, only download the video stream.

      --audio-only
          If media stream has separate audio and video streams, only download the audio stream.

      --simulate
          Download the manifest and print diagnostic information, but do not download audio, video or subtitle content, and write nothing to disk.

      --write-subs
          Write subtitle file, if subtitles are available.

      --keep-video <VIDEO-PATH>
          Keep video stream in file specified by VIDEO-PATH.

      --keep-audio <AUDIO-PATH>
          Keep audio stream (if audio is available as a separate media stream) in file specified by AUDIO-PATH.

      --key <KID:KEY>
          Use KID:KEY to decrypt encrypted media streams. KID should be either a track id in decimal (e.g. 1), or a 128-bit keyid (32 hexadecimal characters). KEY should be 32 hexadecimal characters. Example: --key eb676abbcb345e96bbcf616630f1a3da:100b6c20940f779a4589152b57d2dacb. You can use this option multiple times.

      --save-fragments <FRAGMENTS-DIR>
          Save media fragments to this directory (will be created if it does not exist).

      --ignore-content-type
          Don't check the content-type of media fragments (may be required for some poorly configured servers).

      --add-header <NAME:VALUE>
          Add a custom HTTP header and its value, separated by a colon ':'. You can use this option multiple times.

      --referer <URL>
          Specify content of Referer HTTP header.

  -q, --quiet
          

  -v, --verbose...
          Level of verbosity (can be used several times).

      --no-progress
          Disable the progress bar

      --no-xattr
          Don't record metainformation as extended attributes in the output file.

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
- other: try ffmpeg, which supports many container formats, then try MP4Box



## License

This project is licensed under the MIT license. For more information, see the `LICENSE-MIT` file.



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
