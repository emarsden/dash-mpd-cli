# Build and run the container with
#
#    podman build -f Containerfile --tag dash-mpd-cli
#    podman images
#    podman run -ti -v /tmp:/tmp localhost/dash-mpd-cli dash-mpd-cli -v <MPD-URL> -o /tmp/foo.mp4
    

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
    apk add --no-cache ffmpeg mkvtoolnix bento4 libxslt

COPY --from=docker.io/google/shaka-packager:latest --chown=root:root \
    /usr/bin/packager /usr/local/bin/shaka-packager
COPY --from=rustbuilder --chown=root:root --chmod=755 \
    /src/target/x86_64-unknown-linux-musl/release/dash-mpd-cli /usr/local/bin
COPY --from=gpacbuilder --chown=root:root --chmod=755 \
    /src/gpac/bin/gcc/MP4Box /usr/local/bin

# Size of our container image:
#   with vlc:     331 MB
#   without vlc:  217 MB
#   static ffmpeg + shaka-packager from docker: 299 MB
#   shaka-packager from docker: 216 MB

