#!/usr/bin/env sh
# Build the containerized Debian/Ubuntu-compatible image builder used when the
# host cannot provide live-build/QEMU tools directly.
set -eu
cd "$(dirname "$0")/.."

docker build -f live-build/builder/Dockerfile -t adad-ep009-builder:local .
echo "image builder: ok"
