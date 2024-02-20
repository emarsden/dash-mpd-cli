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
    echo First need to "podman login ghcr.io"
    podman manifest create dash-mpd-cli
    podman manifest annotate --annotation=org.opencontainers.image.description="Download media content from a DASH-MPEG or DASH-WebM MPD manifest."
    podman manifest annotate --annotation=org.opencontainers.image.title="dash-mpd-cli"
    podman manifest annotate --annotation=org.opencontainers.image.url="https://github.com/emarsden/dash-mpd-cli"
    podman manifest annotate --annotation=org.opencontainers.image.source="https://github.com/emarsden/dash-mpd-cli"
    podman manifest annotate --annotation=org.opencontainers.image.version={{version}}
    podman manifest annotate --annotation=org.opencontainers.image.authors="eric.marsden@risk-engineering.org"
    podman manifest annotate --annotation=org.opencontainers.image.licenses="MIT,GPL-2.0-or-later"
    podman build -f etc/Containerfile_linux_amd64 --arch amd64 --tag dash-mpd-cli-linux-amd64 --manifest dash-mpd-cli .
    podman build -f etc/Containerfile_linux_aarch64 --arch arm64 --tag dash-mpd-cli-linux-aarch64 --manifest dash-mpd-cli .
    podman build -f etc/Containerfile_linux_armv7 --arch arm/v7 --tag dash-mpd-cli-linux-armv7 --manifest dash-mpd-cli .
    podman build -f etc/Containerfile_linux_riscv64 --arch riscv64 --tag dash-mpd-cli-linux-riscv64 --manifest dash-mpd-cli .
    podman manifest push --all localhost/dash-mpd-cli ghcr.io/emarsden/dash-mpd-cli


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
    podman run -it --rm docker.io/anchore/grype ghcr.io/emarsden/dash-mpd-cli:latest


publish:
  cargo test
  cargo publish
