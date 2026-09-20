#!/usr/bin/env sh
# Boot build/adad.img in QEMU and assert the M2 hardening service reaches the
# serial console. Runs QEMU inside the EP-009 builder container.
set -eu
cd "$(dirname "$0")/../.."

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
  sh tests/os/boot-smoke-inside.sh
