#!/usr/bin/env sh
# Build the EP-009 Debian-Live image inside the prepared containerized builder.
# The container writes only the final image artifact back to build/adad.img.
set -eu
cd "$(dirname "$0")/.."

builder="${ADAD_IMAGE_BUILDER:-adad-ep009-builder:local}"
workspace="$PWD"
case "$(uname -s 2>/dev/null || echo unknown)" in
  MINGW*|MSYS*|CYGWIN*)
    workspace="$(pwd -W)"
    export MSYS_NO_PATHCONV=1
    export MSYS2_ARG_CONV_EXCL='*'
    ;;
esac

docker run --rm \
  --cap-add SYS_ADMIN \
  --security-opt apparmor=unconfined \
  --security-opt seccomp=unconfined \
  -v "$workspace:/workspace" \
  -w /workspace \
  "$builder" \
  sh scripts/build-image-inside.sh
