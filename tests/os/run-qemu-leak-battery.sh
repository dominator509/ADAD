#!/usr/bin/env sh
# Run the EP-009 on-image leak battery in QEMU through the builder container.
set -eu
cd "$(dirname "$0")/../.."

image="${1:-build/adad.img}"
builder="${ADAD_IMAGE_BUILDER:-adad-ep009-builder:local}"
workspace="$PWD"
case "$(uname -s 2>/dev/null || echo unknown)" in
  MINGW*|MSYS*|CYGWIN*)
    workspace="$(pwd -W)"
    export MSYS_NO_PATHCONV=1
    export MSYS2_ARG_CONV_EXCL='*'
    ;;
esac

docker run --rm -v "$workspace:/workspace" -w /workspace "$builder" \
  sh tests/os/run-qemu-leak-battery-inside.sh "$image"
