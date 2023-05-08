# Changelog

## [0.1.11] - 2023-05-08

- New commandline argument `--limit-rate` to throttle the network bandwidth used to download media
  segments, expressed in octets per second. The limit can be expressed with a k, M or G suffix to
  indicate kB/s, MB/s or GB/s (fractional suffixed quantities are allowed, such as `1.5M`). The
  default is not to throttle bandwidth.


## [0.1.10] - 2023-04-15

- New commandline argument `--max-error-count` to specify the maximum number of non-transient
  network errors that should be ignored before a download is aborted. This is useful in particular
  on some manifests using Time-based or Number-based SegmentLists for which the packager calculates
  a number of segments which is different to our calculation (in which case the last segment can
  generate an HTTP 404 error).
- Update to version 0.7.3 of the dash-mpd crate, which provides better handling of transient and
  non-transient network errors.
- Fix bug in the handling the value of the `--sleep-requests` commandline argument.


## [0.1.9] - 2023-03-19

- Update to version 0.7.2 of the dash-mpd crate. This provides support for downloading additional
  types of subtitles in DASH streams. This version also makes it possible to select between
  native-tls and rustls-tls TLS implementations. We build with rustls-tls in order to build static
  Linux binaries using musl-libc, and to simplify building on Android.


## [0.1.8] - 2023-01-29

- Move to async API used by version 0.7.0 of the dash-mpd crate. There should be no user-visible
  changes in this version.


## [0.1.7] - 2023-01-15

- Add commandline argument `--write-subs` to download subtitles, if they are available. Subtitles
  are downloaded to a file with the same name as the audio-video content, but a filename extension
  dependent on the subtitle format (`.vtt`, `.ttml`, `.srt`).


## [0.1.6] - 2022-11-27

- Add commandline arguments `--keep-video` and `--keep-audio` to retain the files containing video and
  audio content after muxing.
- Add commandline argument `--ignore-content-type` to disable checks that content-type of fragments
  is compatible with audio or video media (may be required for some poorly configured servers).


## [0.1.5] - 2022-10-26

- Produce release binaries for Linux/AMD64, Windows and MacOS/AMD64 using Github Actions.

- Update the version of the clap crate used to parse commandline arguments.


## [0.1.4] - 2022-09-10

- Add commandline arguments `--vlc-location=<path>` and `--mkvmerge-location=<path>` to allow
  specification of a non-standard location for the VLC and mkvmerge binaries, respectively.


## [0.1.3] - 2022-07-02

- Add commandline arguments `--audio-only` and `--video-only`, to retrieve only the audio stream, or
  only the video stream (for streams in which audio and video content are available separately).

- Add commmandline argument `--prefer-language` to allow the user to specify the preferred language
  when multiple audio streams with different languages are available. The argument must be in RFC
  5646 format (eg. "fr" or "en-AU"). If a preference is not specified and multiple audio streams are
  present, the first one listed in the DASH manifest will be downloaded.


## [0.1.2] - 2022-06-01

- Add `--sleep-requests` commandline argument, a number of seconds to sleep between network
  requests. This provides a primitive mechanism for throttling bandwidth consumption.


## [0.1.1] - 2022-03-19

- Add `--ffmpeg-location` commandline argument, to use an ffmpeg which is not in the PATH.


## [0.1.0] - 2022-01-25

- Initial release.
