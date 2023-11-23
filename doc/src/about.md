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

It runs on most common **platforms**, including Linux, Microsoft Windows and MacOS.

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
  
- dash-mpd-cli is written in the Rust programming language, meaning that it's high performance and
  protected from a variety of vulnerabilities that can affect more traditional software.


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



## Licence

dash-mpd-cli is free software distributed according to the terms of the MIT licence.
