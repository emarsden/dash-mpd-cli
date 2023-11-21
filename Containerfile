# Recipe for building a docker container image for dash-mpd-cli + help applications
#
# This Containerfile contains the recipe needed to generate a docker/podman/OCI container image
# including the dash-mpd-cli binary alongside the external helper applications that it uses for
# muxing media streams, for extracting/converting subtitle streams, and for decrypting content
# infected with DRM. These are packaged with a minimal Alpine Linux installation so that they can be
# run on any host that can run Linux/AMD64 containers (using Podman or Docker on Linux, Microsoft
# Windows and MacOS).
#
# Advantages of running in a container, instead of natively on your machine:
#
#   - Much safer, because the container isn't able to modify your host machine, except for writing
#     downloaded media to the directory you specify. This is a very good idea when running random
#     software you downloaded from the internet!
#
#   - No need to install the various helper applications (ffmpeg, mkvmerge, mp4decrypt, MP4Box),
#     which are already present in the container.
#
#   - Automatically run the latest version of dash-mpd-cli and the various helper applications (the
#     container runtime will pull the latest version for you automatically).
#
#   - Podman and Docker also allow you to set various limits on the resources allocated to the
#     container (number of CPUs, memory); see their respective documentation.
#
#
# ## Usage
#
# To run the container:
#
#    podman machine start (optionally, on Windows and MacOS)
#    podman run -ti -v .:/content ghcr.io/emarsden/dash-mpd-cli -v <MPD-URL> -o foo.mp4
#
# On the first run, this will fetch the container image (around 216 MB) from the GitHub Container
# Registry ghcr.io, and will save it for later uses. You can later delete the image using "podman image
# rm" and the image id shown by "podman images".
#
# Your current working directory (".") will be mounted in the container as "/content", which will be
# the working directory in the container. This means that an output file specified without a root
# directory, such as "foo.mp4", will be saved to your current working directory on the host machine.
#
# On Linux/AMD64, it's also possible to run the container using the gVisor container runtime runsc,
# which uses a sandbox to improve security (strong isolation, protection against privilege
# escalation). This requires installation of runsc and running as root (runsc doesn't currently
# support rootless operation).
#
#    sudo apt install runsc
#    sudo podman --runtime=rusc run --rm -ti -v .:/content ghcr.io/emarsden/dash-mpd-cli -v <MPD-URL> -o foo.mp4
#
# To build the container locally (not needed for an end user)
#
#    podman build -f Containerfile --tag dash-mpd-cli
#    podman images



FROM docker.io/rust:latest AS rustbuilder
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

WORKDIR /src
COPY ./ ./
RUN cargo update && \
    cargo build --target x86_64-unknown-linux-musl --release

# static build of MP4Box from GPAC, which is not packaged for Alpine Linux.
# https://github.com/gpac/gpac/wiki/GPAC-Build-Guide-for-Linux
FROM alpine:latest AS gpacbuilder
WORKDIR /src
RUN apk update && \
    apk upgrade && \
    apk add --no-cache musl-dev pkgconfig git g++ binutils make zlib-dev zlib-static && \
    git clone https://github.com/gpac/gpac.git && \
    cd gpac && ./configure --static-bin && \
    make && \
    ls -l bin/gcc/
# here there should be MP4Box


# Now build the final image
FROM alpine:latest
LABEL org.opencontainers.image.description="Download media content from a DASH-MPEG or DASH-WebM MPD manifest." \
    org.opencontainers.image.title="dash-mpd-cli" \
    org.opencontainers.image.url="https://github.com/emarsden/dash-mpd-cli" \
    org.opencontainers.image.source="https://github.com/emarsden/dash-mpd-cli" \
    org.opencontainers.image.version="0.2.9" \
    org.opencontainers.image.authors="eric.marsden@risk-engineering.org" \
    org.opencontainers.image.licenses="MIT,GPL-2.0-or-later"

# Install our external dependencies. Licences for the external dependencies:
#   - ffmpeg: GNU GPL v2
#   - mkvtoolnix: GNU GPL v2
#   - vlc: GNU GPL v2, not installed because it inflates image size considerably
#   - mp4decrypt (bento4): GNU GPL v2
#   - xsltproc: MIT
#   - Shaka packager: MIT
RUN apk update && \
    apk upgrade && \
    apk add --no-cache ffmpeg mkvtoolnix bento4 libxslt && \
    mkdir /content && \
    chown root.root /content && \
    chmod a=rwx,o+t /content

COPY --from=docker.io/google/shaka-packager:latest --chown=root:root \
    /usr/bin/packager /usr/local/bin/shaka-packager
COPY --from=rustbuilder --chown=root:root --chmod=755 \
    /src/target/x86_64-unknown-linux-musl/release/dash-mpd-cli /usr/local/bin
COPY --from=gpacbuilder --chown=root:root --chmod=755 \
    /src/gpac/bin/gcc/MP4Box /usr/local/bin

# This is the default location for downloaded content, writeable by all users. You should bind a
# directory from your host machine on /content so that downloaded media content is available on your
# host machine, for example with
#
#   podman run -ti -v .:/content ghcr.io/emarsden/dash-mpd-cli -v <MPD-URL>
#
# When the download finishes and the container exits, a file with a name determined from MPD-URL
# should be present in your current directory (you can specify the output file name using something
# like "-o foo.mp4").


WORKDIR /content
ENTRYPOINT ["/usr/local/bin/dash-mpd-cli"]

# Size of our container image:
#   with vlc:     331 MB
#   without vlc:  217 MB
#   static ffmpeg + shaka-packager from docker: 299 MB
#   shaka-packager from docker: 216 MB

