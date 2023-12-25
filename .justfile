# For use with the just command runner, https://just.systems/

export LLVM_PROFILE_FILE := 'coverage/cargo-test-%p-%m.profraw'

default:
  @just --list


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


podman:
    podman run -ti -v /tmp:/tmp ghcr.io/emarsden/dash-mpd-cli dash-mpd-cli "$@"


# Build our container for Linux/AMD64, Linux/arm64, Linux/arm/v7 and Linux/riscv64 and push to the
# GitHub Container Registry.
#
# Needs the package qemu-user-static installed to cross build the various architectures.
podman-build-multiarch:
    podman manifest create dash-mpd-cli
    podman build -f etc/Containerfile_linux_amd64 --arch amd64 --tag dash-mpd-cli-linux-amd64 --manifest dash-mpd-cli .
    podman build -f etc/Containerfile_linux_aarch64 --arch arm64 --tag dash-mpd-cli-linux-aarch64 --manifest dash-mpd-cli .
    podman build -f etc/Containerfile_linux_armv7 --arch arm/v7 --tag dash-mpd-cli-linux-armv7 --manifest dash-mpd-cli .
    podman build -f etc/Containerfile_linux_riscv64 --arch riscv64 --tag dash-mpd-cli-linux-riscv64 --manifest dash-mpd-cli .
    podman manifest push --all ghcr.io/emarsden/dash-mpd-cli



publish:
  cargo test
  cargo publish
