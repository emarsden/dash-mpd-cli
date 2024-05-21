# For use with the just command runner, https://just.systems/

export LLVM_PROFILE_FILE := 'coverage/cargo-test-%p-%m.profraw'

default:
  @just --list

version := `git describe --tags`

grcov:
  @echo 'Running tests for coverage with grcov'
  rm -rf ${CARGO_TARGET_DIR}/coverage
  CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' cargo test
  grcov . --binary-path ${CARGO_TARGET_DIR}/debug/deps/ \
    -s . -t html \
    --branch \
    --ignore-not-existing --ignore '../*' --ignore "/*" \
    -o ${CARGO_TARGET_DIR}/coverage
  @echo grcov report in file://${CARGO_TARGET_DIR}/coverage/index.html


coverage-tarpaulin:
  @echo 'Running tests for coverage with tarpaulin'
  mkdir /tmp/tarpaulin
  cargo tarpaulin --engine llvm --line --out html --output-dir /tmp/tarpaulin



setup-coverage-tools:
  rustup component add llvm-tools-preview
  cargo install grcov
  cargo install cargo-tarpaulin
    

termux:
    cargo update
    cargo test --no-default-features -- --show-output

# Build with the UCRT64 enviroment of MSYS, using a restricted PATH to limit the possibility of
# conflicts with non-mingw applications (in particular for autoconf and the C compiler that are
# needed to build the protobuf-src crate).
mingw:
    PATH=/ucrt64/bin:/usr/local/bin:/usr/bin:/bin:/c/windows/System32:/c/ProgramData/chocolatey/bin cargo build --release

podman:
    podman run -ti -v /tmp:/tmp -v .:/content ghcr.io/emarsden/dash-mpd-cli "$@"


# Build our container for Linux/AMD64, Linux/arm64, Linux/arm/v7 and Linux/riscv64 and push to the
# GitHub Container Registry.
#
# Needs the package qemu-user-static installed to cross-build the various architectures.
podman-build-multiarch:
    #!/usr/bin/env bash
    echo First need to "podman login ghcr.io"
    podman manifest create \
      --annotation org.opencontainers.image.description="Download media content from a DASH-MPEG or DASH-WebM MPD manifest." \
      --annotation org.opencontainers.image.title="dash-mpd-cli" \
      --annotation org.opencontainers.image.url="https://github.com/emarsden/dash-mpd-cli" \
      --annotation org.opencontainers.image.source="https://github.com/emarsden/dash-mpd-cli" \
      --annotation org.opencontainers.image.version={{version}} \
      --annotation org.opencontainers.image.authors="eric.marsden@risk-engineering.org" \
      --annotation org.opencontainers.image.licenses="MIT" \
      dash-mpd-cli
    echo === Build container for AMD64
    podman build -f etc/Containerfile_linux_amd64 --arch amd64 --tag dash-mpd-cli-linux-amd64 --manifest dash-mpd-cli .
    podman manifest push --format oci localhost/dash-mpd-cli-linux-amd64 ghcr.io/emarsden/dash-mpd-cli
    echo === Build container for ARM64
    podman build -f etc/Containerfile_linux_aarch64 --arch arm64 --tag dash-mpd-cli-linux-aarch64 --manifest dash-mpd-cli .
    podman manifest push --format oci localhost/dash-mpd-cli-linux-aarch64 ghcr.io/emarsden/dash-mpd-cli

    echo === Build container for ARMv7
    podman build -f etc/Containerfile_linux_armv7 --arch arm/v7 --tag dash-mpd-cli-linux-armv7 --manifest dash-mpd-cli .
    podman manifest push --format oci localhost/dash-mpd-cli-linux-armv7 ghcr.io/emarsden/dash-mpd-cli
    echo === Build container for RISCV64
    podman build -f etc/Containerfile_linux_riscv64 --arch riscv64 --tag dash-mpd-cli-linux-riscv64 --manifest dash-mpd-cli .
    podman manifest push --format oci localhost/dash-mpd-cli-linux-riscv64 ghcr.io/emarsden/dash-mpd-cli
    echo === Build container for PPC64LE
    podman build -f etc/Containerfile_linux_ppc64le --arch ppc64le --tag dash-mpd-cli-linux-ppc64le --manifest dash-mpd-cli .
    echo === Push container to registry
    podman manifest push --all localhost/dash-mpd-cli ghcr.io/emarsden/dash-mpd-cli


test-annotate-manifest:
    DIGEST=`podman manifest create test-annotation`
    podman build -f etc/Containerfile_linux_amd64 --arch amd64 --tag test-annotation-linux-amd64 --manifest test-annotation .
    podman manifest push --format oci --all localhost/test-annotation ttl.sh/test-annotation


list-docker-platforms:
    podman run --rm docker.io/mplatform/mquery ghcr.io/emarsden/dash-mpd-cli:latest


# Run a trivy vulnerability scan of our container image
# https://github.com/aquasecurity/trivy
trivy-container:
    podman run --rm docker.io/aquasec/trivy image ghcr.io/emarsden/dash-mpd-cli:latest

trivy-repository:
    podman run --rm -v $PWD:/myapp docker.io/aquasec/trivy fs --scanners vuln,secret,misconfig .


# Run a grype vulnerability scan of our container image
# https://github.com/anchore/grype
grype-container:
    podman run --rm -it docker.io/anchore/grype ghcr.io/emarsden/dash-mpd-cli:latest


# Using cargo-zigbuild from https://github.com/rust-cross/cargo-zigbuild
macos-AMD64:
    podman run --rm -it \
       -v $(pwd):/io \
       -w /io docker.io/messense/cargo-zigbuild \
       cargo zigbuild --release --target x86_64-apple-darwin

macos-universal:
    podman run --rm -it \
       -v $(pwd):/io \
       -w /io docker.io/messense/cargo-zigbuild \
       cargo zigbuild --release --target universal2-apple-darwin


publish:
  cargo test
  cargo publish
