#!/usr/bin/env bash

set -e

pushd () {
    command pushd "$@" > /dev/null
}

popd () {
    command popd > /dev/null
}

APP="self-assessment"

SCRIPT_PATH=$( cd "$(dirname "$0")" ; pwd -P )

rustup target add aarch64-apple-darwin x86_64-apple-darwin

pushd "$SCRIPT_PATH/.."

VERSION=$(grep '^version =' Cargo.toml | sed 's/version = //g' | sed 's/"//g')

# Some targets do not support async code so we only target ARM and x86
ALL_TRIPLES=$(rustup target list --installed | grep -E '^(aarch64|x86)')

for TRIPLE in $ALL_TRIPLES; do
    echo ''
    echo "=== Creating release for $TRIPLE ==="
    cargo build --release --quiet --target="$TRIPLE"

    pushd "target/$TRIPLE/release"

    TAR_NAME="${APP}_${TRIPLE}_v${VERSION}.tar.gz"

    tar -czf "$TAR_NAME" "$APP"

    echo "The following is the SHA256 sum for the '$TAR_NAME' bundle:"
    shasum -a 256 "$TAR_NAME"

    popd
done

popd