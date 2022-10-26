# dash-mpd-cli

A commandline application for downloading media content from a DASH MPD file, as used for on-demand
replay of TV content and video streaming services like YouTube.

[![Crates.io](https://img.shields.io/crates/v/dash-mpd-cli)](https://crates.io/crates/dash-mpd-cli)
[![Released API docs](https://docs.rs/dash-mpd-cli/badge.svg)](https://docs.rs/dash-mpd-cli/)
[![CI](https://github.com/emarsden/dash-mpd-cli/workflows/build/badge.svg)](https://github.com/emarsden/dash-mpd-cli/workflows/build/badge.svg)
[![Dependency status](https://deps.rs/repo/github/emarsden/dash-mpd-cli/status.svg)](https://deps.rs/repo/github/emarsden/dash-mpd-cli)
[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)

![Terminal capture](https://risk-engineering.org/emarsden/dash-mpd-cli/terminal-capture.svg)


[DASH](https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP) (dynamic adaptive
streaming over HTTP), also called MPEG-DASH, is a technology used for media streaming over the web,
commonly used for video on demand (VOD) services. The Media Presentation Description (MPD) is a
description of the resources (manifest or “playlist”) forming a streaming service, that a DASH
client uses to determine which assets to request in order to perform adaptive streaming of the
content. DASH MPD manifests can be used both with content encoded as MPEG and as WebM. There is a
good explanation of adaptive bitrate video streaming at
[howvideo.works](https://howvideo.works/#dash).

This commandline application allows you to download content (audio or video) described by an MPD
manifest. This involves selecting the alternative with the most appropriate encoding (in terms of
bitrate, codec, etc.), fetching segments of the content using HTTP or HTTPS requests and muxing
audio and video segments together. It builds on the [dash-mpd](https://crates.io/crates/dash-mpd)
crate.


## Installation

With an [installed Rust development environment](https://www.rust-lang.org/tools/install): 

```shell
cargo install dash-mpd-cli
```


## Usage

```
dash-mpd-cli 0.1.4
Download content from an MPEG-DASH streaming media manifest

USAGE:
    dash-mpd-cli [OPTIONS] <MPD-URL>

ARGS:
    <MPD-URL>    URL of the DASH manifest to retrieve

OPTIONS:
        --add-header <extra-header>
            Add a custom HTTP header and its value, separated by a colon ':'. You can use this
            option multiple times.

        --audio-only
            If the media stream has separate audio and video streams, only download the audio stream

        --ffmpeg-location <ffmpeg-location>
            Path to the ffmpeg binary (necessary if not located in your PATH)

    -h, --help
            Print help information

        --mkvmerge-location <mkvmerge-location>
            Path to the mkvmerge binary (necessary if not located in your PATH)

        --no-progress
            Disable the progress bar

        --no-xattr
            Don't record metainformation as extended attributes in the output file

    -o, --output <output-file>
            Save media content to this file

        --prefer-language <lang>
            Preferred language when multiple audio streams with different languages are available.
            Must be in RFC 5646 format (eg. fr or en-AU). If a preference is not specified and
            multiple audio streams are present, the first one listed in the DASH manifest will be
            downloaded.

        --proxy <proxy>
            

    -q, --quiet
            

        --quality <quality>
            Prefer best quality (and highest bandwidth) representation, or lowest quality [possible
            values: best, worst]

        --sleep-requests <sleep-seconds>
            Number of seconds to sleep between network requests (default 0)

        --source-address <source-address>
            Source IP address to use for network requests, either IPv4 or IPv6. Network requests
            will be made using the version of this IP address (eg. using an IPv6 source-address will
            select IPv6 network traffic).

        --timeout <timeout>
            Timeout for network requests (from the start to the end of the request), in seconds

        --user-agent <user-agent>
            

    -v, --verbose
            Level of verbosity (can be used several times)

        --version
            

        --video-only
            If the media stream has separate audio and video streams, only download the video stream

        --vlc-location <vlc-location>
            Path to the VLC binary (necessary if not located in your PATH)
```


If your filesystem supports **extended attributes**, the application will save the following
metainformation in the output file:

- `user.xdg.origin.url`: the URL of the MPD manifest
- `user.dublincore.title`: the title, if specified in the manifest metainformation
- `user.dublincore.source`: the source, if specified in the manifest metainformation
- `user.dublincore.rights`: copyright information, if specified in the manifest metainformation

You can examine these attributes using `xattr -l` (you may need to install your distribution's
`xattr` package). Disable this feature using the `--no-xattr` commandline argument.


## Platforms

This crate is tested on the following platforms:

- Linux
- MacOS
- Microsoft Windows 10
- Android 11 on Aarch64 via termux (you’ll need to install the rust, binutils and ffmpeg packages)

The underlying library `dash-mpd-rs` has two methods for muxing audio and video streams together. If
the library feature `libav` is enabled (which is not the default configuration), muxing support is
provided by ffmpeg’s libav library, via the `ac_ffmpeg` crate. 
Otherwise, muxing is implemented by calling an external muxer, mkvmerge (from the
[MkvToolnix](https://mkvtoolnix.download/) suite), [ffmpeg](https://ffmpeg.org/) or
[vlc](https://www.videolan.org/vlc/) as a subprocess. Note that these commandline applications
implement a number of checks and workarounds to fix invalid input streams that tend to exist in the
wild. Some of these workarounds are implemented here when using libav as a library, but not all of
them, so download support tends to be more robust with the default configuration (using an external
application as a subprocess). The `libav` feature currently only works on Linux. 

The choice of external muxer depends on the filename extension of the path supplied to `--output`
or `-o` (which will be ".mp4" if you don't specify the output path explicitly):

- .mkv: call mkvmerge first, then if that fails call ffmpeg
- .mp4: call ffmpeg first, then if that fails call vlc
- other: try ffmpeg, which supports many container formats


## DASH features supported

- VOD (static) stream manifests
- Multi-period content
- XLink elements (only with actuate=onLoad semantics), including resolve-to-zero
- All forms of segment index info: SegmentBase@indexRange, SegmentTimeline,
  SegmentTemplate@duration, SegmentTemplate@index, SegmentList
- Media containers of types supported by ffmpeg or VLC (this includes ISO-BMFF / CMAF / MP4, WebM, MPEG-2 TS)


## Limitations / unsupported features

- Dynamic MPD manifests, that are used for live streaming/OTT TV
- Encrypted content using DRM such as Encrypted Media Extensions (EME) and Media Source Extension (MSE)
- Subtitles (eg. WebVTT and TTML streams)
- XLink with actuate=onRequest



## License

This project is licensed under the MIT license. For more information, see the `LICENSE-MIT` file.



## Similar tools

Similar commandline tools that are able to download content from a DASH manifest:

- `yt-dlp <MPD-URL>`

- `streamlink -o /tmp/output.mp4 <MPD-URL> worst`

- `ffmpeg -i <MPD-URL> -vcodec copy /tmp/output.mp4`

- `vlc <MPD-URL>`

- `gst-launch-1.0 playbin uri=<MPD-URL>`

This application is able to download content from certain streams that do not work with other
applications (for example xHE-AAC streams which are unsupported by ffmpeg, streamlink, VLC,
gstreamer).


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
reqwest crate and build it with rustls-tls support. 
