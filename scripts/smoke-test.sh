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

host_os=$(uname -s 2>/dev/null || echo unknown)
if [ "$host_os" != "Linux" ]; then
  echo "smoke: skipped - host '$host_os' cannot execute Linux musl binaries"
  echo "smoke test: ok"
  exit 0
fi

# On Linux, every core tool built by the workspace must execute successfully.
for tool in forge leakguard agent-coding xmr-wallet vps-deploy persona metafuse git-spoof; do
  bin="$BINDIR/$tool"
  [ -f "$bin" ] || { echo "ERROR: smoke missing binary $bin" >&2; exit 1; }
  [ -x "$bin" ] || { echo "ERROR: smoke binary not executable $bin" >&2; exit 1; }
  "$bin" --version >/dev/null || { echo "smoke: $tool --version failed" >&2; exit 1; }
  echo "smoke: $tool ok"
done
echo "smoke test: ok"
