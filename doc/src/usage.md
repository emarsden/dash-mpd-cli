# Usage

This is a commandline application, meaning it runs in a terminal (there is no graphical user interface).


## Quickstart

To download from a manifest to a file called `MyVideo.mp4`:

```shell
dash-mpd-cli -v --quality best https://example.com/manifest.mpd -o MyVideo.mp4
```

To download including Finnish subtitles (which should be written to a file named `MyVideo.srt` or
`MyVideo.vtt`, depending on the type of subtitles):

    dash-mpd-cli -v --quality best --prefer-language fi --write-subs https://example.com/manifest.mpd -o MyVideo.mp4

To know what subtitles and subtitle languages are available, first run: 

    dash-mpd-cli -v -v --simulate https://example.com/manifest.mpd

To save the output to a Matroska container using mkvmerge as a muxer:

    dash-mpd-cli --muxer-preference mkv:mkvmerge https://example.com/manifest.mpd -o MyVideo.mkv

To decrypt DRM on the media streams (assuming there are different keys for the audio and the video streams):

    dash-mpd-cli --key "43215678123412341234123412341237:12341234123412341234123412341237" \
      --key 43215678123412341234123412341236:12341234123412341234123412341236 \
       https://example.com/manifest.mpd -o MyVideo.mp4

To use ffmpeg that is installed in a non-standard location which is not in your PATH:

    dash-mpd-cli --ffmpeg-location e:/ffmpeg/ffmpeg.exe https://example.com/manifest.mpd -o MyVideo.mp4

To send necessary cookies to the web server frome Firefox (where you have logged in to the private
website):

    dash-mpd-cli --cookies-from-browser Firefox https://example.com/manifest.mpd -o MyVideo.mp4
    
If you want to interrupt a download, type `Ctrl-C` (this works at least on Linux, Windows, MacOS and
termux on Android).




## Commandline options

**Usage**: `dash-mpd-cli [OPTIONS] MPD-URL`

Options:

    -U, --user-agent <user-agent>
          
The value of the [user-agent
header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/User-Agent) in HTTP requests. The
default is `dash-mpd-cli/<version>`. If you want to look more like traffic from a web browser,
choose a [user agents in current use](https://www.useragents.me/).


    --proxy <URL>

The URL of a Socks or HTTP proxy (e.g. https://example.net/ or socks5://example.net/) to use for all
network requests.

    --no-proxy

Disable use of Socks or HTTP proxy even if the related environment variables are set.

    --auth-username <USER>

Username to use for authentication with the server(s) hosting the DASH manifest and the media
segments (only relevant for HTTP Basic authentication).

    --auth-password <PASSWORD>

Password to use for authentication with the server(s) hosting the DASH manifest and the media
segments (only relevant for HTTP Basic authentication).

    --auth-bearer <TOKEN>

Token to use for authentication with the server(s) hosting the DASH manifest and the media segments,
when HTTP Bearer authentication is required.

    --timeout <SECONDS>

Timeout for each network request (from the start to the end of the request), in seconds.

    --sleep-requests <SECONDS>

Number of seconds to sleep between network requests (default 0).

    --enable-live-streams

Attempt to download from a live media stream (dynamic MPD manifest). Downloading from a genuinely
live stream won't work well, because we don't implement the clock-related throttling needed to only
download media segments when they become available. However, some media sources publish pseudo-live
streams where all media segments are in fact available, which we will be able to download. You might
also have some success in combination with the `--sleep-requests` argument.

    --force-duration <SECONDS>

Specify a number of seconds (possibly floating point) to download from the media stream. This may be
necessary to download from a live stream, where the duration is often not specified in the DASH
manifest. It may also be used to download only the first part of a static stream.

    -r, --limit-rate <RATE>
    
Maximum network bandwidth in octets per second (default no limit). For example, `200K`, `1M`.

    --max-error-count <COUNT>

Maximum number of non-transient network errors that should be ignored before a download is aborted
(default is 10).

    --source-address <source-address>

Source IP address to use for network requests, either IPv4 or IPv6. Network requests will be made
using the version of this IP address (e.g. using an IPv6 source-address will select IPv6 network
traffic).

    --add-root-certificate <CERT>
    
Add a root certificate (in PEM format) to be used when verifying TLS network connections. This
option can be used multiple times.

    --client-identity-certificate <CERT>

Client private key and certificate (in PEM format) to be used when authenticating TLS network connections.

    --prefer-video-width <WIDTH>

When multiple video streams are available, choose that with horizontal resolution closest to `WIDTH`.

    --prefer-video-height <HEIGHT>

When multiple video streams are available, choose that with vertical resolution closest to `HEIGHT`.

    --quality <quality>

Prefer best quality (and highest bandwidth) representation, or lowest quality. Possible values:
`best`, `intermediate`, `worst`.

    --prefer-language <LANG>

Preferred language when multiple audio streams with different languages are available. Must be in
RFC 5646 format (e.g. fr or en-AU). If a preference is not specified and multiple audio streams are
present, the first one listed in the DASH manifest will be downloaded.

    --xslt-stylesheet <STYLESHEET>
 
XSLT stylesheet with rewrite rules to be applied to the manifest before downloading media content.
Stylesheets are applied using the xsltproc commandline application, which implements XSLT 1.0. You
can use this option multiple times. This option is currently experimental.

    --video-only

If media stream has separate audio and video streams, only download the video stream.

    --audio-only

If media stream has separate audio and video streams, only download the audio stream.

    --simulate

Download the manifest and print diagnostic information, but do not download audio, video or subtitle
content, and write nothing to disk.

    --write-subs

Download and save subtitle file, if subtitles are available.

    --keep-video <VIDEO-PATH>

Keep video stream in file specified by `VIDEO-PATH`.

    --keep-audio <AUDIO-PATH>

Keep audio stream (if audio is available as a separate media stream) in file specified by `AUDIO-PATH`.

    --no-period-concatenation

Never attempt to concatenate media from different Periods. If multiple periods are present, one
output file per Period will be saved, with names derived from the requested output filename (adding
`-p2` for the second period, `-p3` for the third period, and so on.

    --muxer-preference <CONTAINER:ORDERING>

When muxing into `CONTAINER`, try muxing applications in order `ORDERING`. You can use this option
multiple times. Examples: `mp4:mp4box,vlc` and `avi:ffmpeg`.

    --key <KID:KEY>

Use `KID:KEY` to decrypt encrypted media streams. `KID` should be either a track id in decimal (e.g.
1), or a 128-bit keyid (32 hexadecimal characters). `KEY` should be 32 hexadecimal characters.
Example: `--key eb676abbcb345e96bbcf616630f1a3da:100b6c20940f779a4589152b57d2dacb`. You can use this
option multiple times.

Please note that obtaining decryption keys is beyond the scope of this application.

    --decryption-application <APP>

Application to use to decrypt encrypted media streams (either `mp4decrypt` or `shaka`).
          

    --save-fragments <FRAGMENTS-DIR>
 
Save media fragments to this directory (will be created if it does not exist).

    --ignore-content-type

Don't check the content-type of media fragments (may be required for some poorly configured servers).

    --add-header <NAME:VALUE>

Add a custom HTTP header and its value, separated by a colon ':'. You can use this option multiple times.


    -H, --header <HEADER>
     
Add a custom HTTP header, in cURL-compatible format. You can use this option multiple times.
Example: `-H 'X-Custom: ized'`.

    --referer <URL>
 
Specify the content of the Referer HTTP header.

    -q, --quiet
          
Disable printing of diagnostics information to the terminal.

    -v, --verbose

Level of verbosity (can be used several times).

    --no-progress

Disable the progress bar.

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

Load cookies from `BROWSER` (possible values, depending on your operating system, include Firefox,
Chrome, ChromeBeta, Chromium).

    --list-cookie-sources

Show valid values for the `BROWSER` argument to `--cookies-from-browser` on this computer, then exit.

    -h, --help
  
Print help (see a summary with `-h`)

    -V, --version
  
Print version and exit.



## Relevant environment variables

You can set certain environment variables to modify the behaviour of the application: 

- On Linux and MacOS, the `TMPDIR` environment variable will determine where temporary files used
  while downloading are saved. These temporary files should be cleaned up by the application, unless
  you interrupt execution using Ctrl-C.
  
- On Microsoft Windows, the `TMP` and `TEMP` environment variables will determine where temporary files
  are saved (see the documentation of the `GetTempPathA` function in the Win32 API, or the Rust
  documentation for [`std::env::tmpdir`](https://doc.rust-lang.org/std/env/fn.temp_dir.html)). 

- The `RUST_LOG` environment variable can be used to obtain extra debugging logging (see the
  [documentation for the env_logger crate](https://docs.rs/env_logger/latest/env_logger/)). For
  example, you can ask for voluminous logging using

```shell
RUST_LOG=trace dash-mpd-cli -o foo.mp4 https://example.com/manifest.mpd
```

  or if [running in a container](container.html)
  
```shell
podman run --env RUST_LOG=trace \
   -v .:/content \ 
   ghcr.io/emarsden/dash-mpd-cli \
   https://example.com/manifest.mpd -o foo.mp4
```


## Recording metadata

If your filesystem supports **extended attributes**, the application will save the following
metainformation in the output file:

- `user.xdg.origin.url`: the URL of the MPD manifest
- `user.dublincore.title`: the title, if specified in the manifest metainformation
- `user.dublincore.source`: the source, if specified in the manifest metainformation
- `user.dublincore.rights`: copyright information, if specified in the manifest metainformation

You can examine these attributes using `xattr -l` (you may need to install your distribution's
`xattr` package). Disable this feature using the `--no-xattr` commandline argument.
