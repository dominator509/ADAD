#!/usr/bin/env sh
# Non-destructive EP-010 rollback drill. Copies the image artifact inside the
# builder container to simulate selecting a prior image, then boots that copy.
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
  sh tests/os/rollback-drill-inside.sh
