#!/usr/bin/env sh
# Fast smoke test: the core binaries run and report version/health without a
# full OS boot. Confirms the workspace produced runnable artifacts.
set -eu
cd "$(dirname "$0")/.."
BINDIR=target/x86_64-unknown-linux-musl/release
if [ ! -d "$BINDIR" ]; then
  echo "ERROR: no release binaries. Run scripts/build.sh first (EP-001+)." >&2
  exit 1
fi
# Each core tool must answer --version with exit 0 once it exists. Tools not yet
# built by the current phase are skipped, not failed.
for tool in forge leakguard agent-coding xmr-wallet vps-deploy persona metafuse git-spoof; do
  bin="$BINDIR/$tool"
  if [ -x "$bin" ]; then
    "$bin" --version >/dev/null || { echo "smoke: $tool --version failed" >&2; exit 1; }
    echo "smoke: $tool ok"
  fi
done
echo "smoke test: ok"
