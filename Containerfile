# Our container size is 331 MB.
#
# Build the container with
#
#    podman build -f Containerfile --tag dash-mpd-cli
#    podman images
#    podman run -ti -v /tmp:/tmp localhost/dash-mpd-cli dash-mpd-cli -v https://cloudflarestream.com/31c9291ab41fac05471db4e73aa11717/manifest/video.mpd -o /tmp/foo.mp4
#    

FROM docker.io/rust:latest AS rustbuilder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

ENV USER=dashmpd
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

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
LABEL description="Download media content from a DASH-MPEG or DASH-WebM MPD manifest." \
    org.opencontainers.image.title="dash-mpd-cli" \
    org.opencontainers.image.source="https://github.com/emarsden/dash-mpd-cli" \
    org.opencontainers.image.version="v0.2.9" \
    org.opencontainers.image.authors="eric.marsden@risk-engineering.org" \
    org.opencontainers.image.licenses="MIT,GPL-2.0-or-later"

COPY --from=rustbuilder /etc/passwd /etc/passwd
COPY --from=rustbuilder /etc/group /etc/group

# Install our external dependencies. Licences for the external dependencies:
#   - ffmpeg: GNU GPL v2
#   - mkvtoolnix: GNU GPL v2
#   - vlc: GNU GPL v2
#   - mp4decrypt (bento4): GNU GPL v2
#   - xsltproc: MIT
#   - Shaka packager: MIT
RUN apk update && \
    apk upgrade && \
    apk add --no-cache wget ffmpeg mkvtoolnix vlc bento4 libxslt && \
    wget -q -O /tmp/shaka-packager https://github.com/shaka-project/shaka-packager/releases/latest/download/packager-linux-x64 && \
    mv /tmp/shaka-packager /usr/local/bin && \
    chmod +x /usr/local/bin/shaka-packager

COPY --from=rustbuilder --chown=root:root --chmod=755 /src/target/x86_64-unknown-linux-musl/release/dash-mpd-cli /usr/local/bin
COPY --from=gpacbuilder --chown=root:root --chmod=755 /src/gpac/bin/gcc/MP4Box /usr/local/bin/mp4box

USER dashmpd:dashmpd
