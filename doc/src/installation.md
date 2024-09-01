# Installation

The recommended way of running dash-mpd-cli is conveniently sandboxed in a [Podman
container](container.html). If you prefer to install the software and its dependencies on your
computer in the traditional way, you can download a prebuilt binary or build from source yourself.

**Binary releases** are [available on GitHub](https://github.com/emarsden/dash-mpd-cli/releases) for
GNU/Linux on AMD64 (statically linked against Musl Libc to avoid glibc versioning problems),
Microsoft Windows on AMD64 and MacOS on aarch64 (“Apple Silicon”) and AMD64. These are built
automatically on the GitHub continuous integration infrastructure.

You can also **build from source** using an [installed Rust development
environment](https://www.rust-lang.org/tools/install):

```shell
cargo install dash-mpd-cli
```

This installs the binary to your installation root’s `bin` directory, which is typically
`$HOME/.cargo/bin`.


## External dependencies

You should also install the following **dependencies**:

- the mkvmerge commandline utility from the [MkvToolnix](https://mkvtoolnix.download/) suite, if you
  download to the Matroska container format (`.mkv` filename extension). mkvmerge is used as a
  subprocess for muxing (combining) audio and video streams. See the `--mkvmerge-location`
  commandline argument if it’s not installed in a standard location (not on your PATH).

- [ffmpeg](https://ffmpeg.org/) for muxing audio and video streams and for concatenating streams
  from a multi-period manifest. The ffprobe binary (distributed with ffmpeg) is required alongside
  the ffmpeg binary. See the `--ffmpeg-location` commandline argument if this is installed in a
  non-standard location.
  
- [vlc](https://www.videolan.org/vlc/) as an alternative application for muxing audio and video
  streams (sometimes VLC is able to mux certain streams that ffmpeg doesn’t support). See the
  `--vlc-location` commandline argument if this is installed in a non-standard location. Also see
  the `--muxer-preference` commandline argument to specify which muxing application to prefer for
  different container types.

- the MP4Box commandline utility from the [GPAC](https://gpac.wp.imt.fr/) project, to help with
  subtitles in wvtt format. If it’s installed, MP4Box will be used to convert the wvtt stream to the
  more widely recognized SRT format. MP4Box can also be used for muxing audio and video streams to
  an MP4 container, as a fallback if ffmpeg and vlc are not available. See the `--mp4box-location`
  commandline argument if this is installed in a non-standard location.

- the mp4decrypt commandline application from the [Bento4
  suite](https://github.com/axiomatic-systems/Bento4/), if you need to fetch encrypted content.
  [Binaries are available](https://www.bento4.com/downloads/) for common platforms. See the
  `--mp4decrypt-location` commandline argument if this is installed in a non-standard location.

- for some types of streams that the mp4decrypt application is not able to decrypt (for example
  content in WebM containers), you should install the [Shaka packager
  application](https://github.com/shaka-project/shaka-packager) developed by Google. See the
  `--decryption-application` commandline option to specify the choice of decryption application, and
  the `--shaka-packager-location` commandline argument if it is installed in a non-standard location.

- the xsltproc commandline utility packaged with libxslt, which is used for the MPD rewriting
  functionality (the `--drop-elements` and `--xslt-stylesheet` commandline options).



## Supported platforms

This crate is tested on the following **platforms**:

- Our container images are tested using Podman on Linux and Windows

- Linux on AMD64 (x86-64) and Aarch64 architectures

- MacOS on AMD64 and Aarch64 architectures

- Microsoft Windows 10 and Windows 11 on AMD64

- Android 12 on Aarch64 via [termux](https://termux.dev/) (you’ll need to install the rust, binutils
  and ffmpeg packages, and optionally the mkvtoolnix, vlc and gpac packages). You’ll need to disable
  the `cookies` feature by building with `--no-default-features`.

- FreeBSD/AMD64 and OpenBSD/AMD64. You’ll need to disable the `cookies` feature. Some of the
  external applications we depend on (e.g. mp4decrypt, Shaka packager) are poorly supported on OpenBSD.

It should also work on more obscure platforms, such as ppc64le and RISC-V, as long as you can
install a recent Rust toolchain and have ffmpeg working.
