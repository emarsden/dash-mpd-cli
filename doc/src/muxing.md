# Muxing and concatenating media streams


DASH streams are often published with separate audio and video streams. This allows the publisher to
offer audio which is dubbed into different languages, or which is encoded using more basic or more
sophisticated codecs. When they are published separately, this application merges the selected audio
and video streams into a single output media container (such as MPEG-4 with a `.mp4` filename
extension, or Matroska with a `.mkv` filename extension), a process called “muxing”.

Unless you compiled dash-mpd-cli yourself and enabled the `libav` feature, the muxing is implemented
by an external application.


## Selecting a muxing application

The application has support for several muxing applications: mkvmerge (from the
[MkvToolnix](https://mkvtoolnix.download/) suite), [ffmpeg](https://ffmpeg.org/),
[vlc](https://www.videolan.org/vlc/) and [MP4Box](https://github.com/gpac/gpac/wiki/MP4Box). These
must be installed separately on your computer (they are not distributed with dash-mpd-cli), unless
you are using our Docker container, where these applications (except for VLC which is a little
large) are preinstalled. If these applications are installed to a non-standard location which is not
present on your `PATH`, you will need to specify their location using one of the commandline options
`--mkvmerge-location`, `--ffmpeg-location`, `--vlc-location` and `--mp4box-location`.

The choice of external muxer depends on the filename extension of the path supplied to `--output`
or `-o`: 

- `.mkv`: call mkvmerge first, then if that fails call ffmpeg, then try MP4Box
- `.mp4`: call ffmpeg first, then if that fails call vlc, then try MP4Box
- `.webm`: call vlc, then if that fails ffmpeg
- other: try ffmpeg, which supports many container formats, then try MP4Box

If you don’t specify the output path, the filename extension will be `.mp4`.

You can specify a different **order of preference** for muxing applications using the
`--muxer-preference` commandline option. For example, `--muxer-preference avi:vlc,ffmpeg` means that
for an AVI media container the external muxer vlc will be tried first, then ffmpeg in case of
failure. This commandline option can be used multiple times to specify options for different
container types.



## Concatenating multi-period streams

Certain DASH streams are split up into multiple periods, represented by different `Period` elements
in the XML manifest. You can think of these as chapters on a DVD, movements in a piece of classical
music, or segments between advertising breaks (a common reason for multi-period manifests). Some
information on the periods present (identifier, codec, resolution and so on) will be printed if
you enable the `--verbose` commandline option.

If you use the `--no-period-concatenation` commandline option, each period will be saved into a
separate media container, whose name has `-pN` appended. For example, if you used the commandline
argument `-o concerto.mp4` and the stream contains three periods, they will be saved to three files:

- `concerto.mp4` for the first period
- `concerto-p2.mp4` for the second period
- `concerto-p3.mp4` for the third period

If the `--no-period-concatenation` commandline option is not used, and the media in the different
periods is compatible (same video resolution, codecs, framerate and so on), then dash-mpd-cli will
attempt to concatenate them into a single output file. This concatenation process uses either
mkvmerge or the [concat filter of ffmpeg](https://ffmpeg.org/ffmpeg-filters.html#concat)
(irrespective of the selected muxing application); see the the `--concat-preference` commandline
argument). Concatenation can be slow, because it may require re-encoding of the different media
streams. If the concatenation fails, the periods will be retained as separate files, as specified
above.

