# Changelog


## [0.2.5] - 2023-09-03

- New commandline arguments `--prefer-video-width` and `--prefer-video-height` which allow the user
  to specify the video stream to be downloaded, when multiple video streams with different
  resolutions are made available. The video stream with the horizontal (respectively vertical)
  resolution closest to the specified width (respectively height) is chosen. This preference only
  concerns the video stream; use the `--quality` commandline argument to specify the preferred audio
  stream when multiple audio streams with different quality levels are available. If a preference
  for both video width and video height is provided, the preferred width is used. A width or height
  preference overrides (for the video stream) a specified quality preference.

- New value `intermediate` recognized for the `--quality` commandline argument. If the DASH manifest
  specifies several Adaptations with different bitrates or quality levels (specified by the
  `@qualityRanking` attribute in the manifest -- quality ranking may different from bandwidth
  ranking when different codecs are used), prefer the Adaptation with an intermediate bitrate
  (closest to the median value).

- New commandline arguments `--auth-username` and `--auth-password` to specify the username and
  password to be used for authentication with the server. Currently only HTTP Basic authentication
  is supported.


## [0.2.4] - 2023-08-14

- New commandline argument `--header` (alias `-H`) which is compatible with cURL. This can be
  convenient when using “Copy as cURL” functionality in Chromium DevTools. The syntax for the
  argument is slightly different from the existing `--add-header` commandline argument.

- On startup, check whether a newer version is available as a GitHub release, unless the
  `--no-version-check` commandline option is enabled.

- Improve support for multiperiod manifests. When the contents of the different periods
  can be joined into a single output container (because they share the same resolution, frame rate
  and aspect ratio), we concatenate them using ffmpeg (with reencoding in case the codecs in the
  various periods are different). If they cannot be joined, we save the content in output files
  named according to the requested output file (whose name is used for the first period). Names
  ressemble "output-p2.mp4" for the second period, and so on.

- New function `concatenate_periods` on `DashDownloader` to specify whether the concatenation using
  ffmpeg (which is very CPU-intensive due to the reencoding) of multi-period manifests should be
  attempted. The default behaviour is to concatenate when the media contents allow it.

- Improved support for certain addressing types on subtitles (AdaptationSet>SegmentList,
  Representation>SegmentList, SegmentTemplate+SegmentTimeline addressing modes).

- Significantly improved support for XLink semantics on elements (remote elements). In particular, a
  remote XLinked element may resolve to multiple elements (e.g. a Period with href pointing to a
  remote MPD fragment may resolve to three final Period elements), and a remote XLinked element may
  contain a remote XLinked element (the number of repeated resolutions is limited, to avoid DoS
  attacks).


## [0.2.3] - 2023-08-05

- New commandline argument `--referer` to specify the value of the Referer HTTP header. This is an
  alternative to the use of the `--add-header` commandline argument.

- Fix regression: restore printing of logged diagnostics.

- Add support for EIA-608 aka CEA-608 subtitles/closed captions.

- More diagnostics information is printed concerning the selected audio/video streams. In
  particular, pssh information will be printed for streams with ContentProtection whose pssh is
  embedded in the initialization segments rather than in the DASH manifest.


## [0.2.2] - 2023-07-16

- New commandline argument `--simulate` to retrieve the MPD manifest but not download any audio,
  video or subtitle content.

- Improve support for retrieving subtitles that are distributed in fragmented MP4 streams (in
  particular WebVTT/STPP formats).

- More diagnostics information is printed concerning the selected audio/video streams.


## [0.2.1] - 2023-07-08

- Support for decrypting encrypted media streams that use ContentProtection, via the Bento4
  mp4decrypt commandline application. See the `--key` commandline argument to allow kid:key pairs to
  be specified, and the `--mp4decrypt-location` commandline argument to specify a non-standard
  location for the mp4decrypt binary.

- Fix a bug in the handling of the `--add-header` commandline argument.


## [0.2.0] - 2023-06-25

- Incompatible change to the `--keep_audio` and `keep_video` commandline arguments, to allow
  the user to specify the path for the audio and video files. Instead of operating as flags, they
  allow the user to specify the filename to which the corresponding stream will be saved (and not
  deleted after muxing).

- New commandline argument `--client-identity-certificate` to provide a file containing a private
  key and certificate (both encoded in PEM format). These will be used to authenticate TLS network
  connections.

- Print information on the different media streams available (resolution, bitrate, codec) in a
  manifest when requested verbosity is non-zero.


## [0.1.14] - 2023-06-10

- New commandline argument `--add-root-certificate` to add an X.509 certificate to the list of root
  certificates used to check TLS connections to servers. Certificates should be provided in PEM format.

- New commandline argument `--no-proxy` to disable use of a proxy, even if related enviroment
  variables (`HTTP_PROXY` etc.) are set.

- Connection errors at the network level are handled as permanent, rather than transient, errors. In
  particular, TLS certificate verification errors will no longer be treated as transient errors.


## [0.1.13] - 2023-05-28

- New commandline argument `--cookies-from-browser` to load HTTP cookies from a web browser (support
  for Firefox, Chromium, Chrome, ChromeBeta, Edge and Safari on Linux, Windows and MacOS, via the
  bench_scraper crate). This support is gated by the `cookies` feature, which is enabled by default.
  Disable it (with `--no-default-features`) to build on platforms which are not supported by the
  bench_scraper crate, such as Android/Termux.


## [0.1.12] - 2023-05-12

- Add commandline argument `--save-fragments` to save media fragments (individual DASH audio and
  video segments) to the specified directory.

- Add commandline argument `--mp4box-location=<path>` to allow a non-standard location for the
  MP4Box binary (from the GPAC suite) to be specified.

- Update to version 0.9.0 of the dash-mpd crate, which is more tolerant of unexpected extensions to
  the DASH schema.


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
