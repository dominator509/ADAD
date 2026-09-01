#!/usr/bin/env sh
# Verify the EP-009 containerized builder has the host tools the image recipe
# needs. This does not write a real device; it only runs the repo install check
# inside the builder image.
set -eu
cd "$(dirname "$0")/.."

workspace="$PWD"
case "$(uname -s 2>/dev/null || echo unknown)" in
  MINGW*|MSYS*|CYGWIN*)
    workspace="$(pwd -W)"
    export MSYS_NO_PATHCONV=1
    export MSYS2_ARG_CONV_EXCL='*'
    ;;
esac

docker run --rm -v "$workspace:/workspace" -w /workspace adad-ep009-builder:local \
  sh -lc 'export PATH=/usr/local/cargo/bin:$PATH; scripts/install.sh'
echo "image builder check: ok"
