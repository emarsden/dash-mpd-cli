# Changelog


## [0.1.4] - 2022-09-10

- Add commandline arguments --vlc-location=<path> and --mkvmerge-location=<path> to allow specification
  of a non-standard location for the VLC and mkvmerge binaries, respectively.


## [0.1.3] - 2022-07-02

- Add commandline arguments --audio-only and --video-only, to retrieve only the audio stream, or
  only the video stream (for streams in which audio and video content are available separately).

- Add commmandline argument --prefer-language to allow the user to specify the preferred language
  when multiple audio streams with different languages are available. The argument must be in RFC
  5646 format (eg. "fr" or "en-AU"). If a preference is not specified and multiple audio streams are
  present, the first one listed in the DASH manifest will be downloaded.

## [0.1.2] - 2022-06-01

- Add --sleep-requests commandline argument, a number of seconds to sleep between network requests.
  This provides a primitive mechanism for throttling bandwidth consumption.


## [0.1.1] - 2022-03-19

- Add --ffmpeg-location commandline argument, to use an ffmpeg which is not in the PATH.


## [0.1.0] - 2022-01-25

- Initial release.
