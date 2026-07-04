#!/usr/bin/env sh
# Build all Rust core tools as STATIC musl binaries (constraint: no dynamic
# linking, to keep torsocks/LD_PRELOAD out of the picture). The bootable image
# itself is built by scripts in the live-build recipe, invoked by EP-009, not
# here - this target builds the binaries the image consumes.
set -eu
cd "$(dirname "$0")/.."
if [ ! -f Cargo.toml ]; then
  echo "ERROR: no Cargo.toml yet. EP-001 creates the workspace first." >&2
  exit 1
fi
cargo build --workspace --release --target x86_64-unknown-linux-musl

host_os=$(uname -s 2>/dev/null || echo unknown)
if [ "$host_os" != "Linux" ]; then
  echo "build: static verification skipped on host '$host_os'; Linux CI is authoritative"
  echo "build: ok"
  exit 0
fi

# Verify the binaries really are static (no NEEDED entries). Guards the
# constraint automatically.
for bin in target/x86_64-unknown-linux-musl/release/*; do
  [ -f "$bin" ] && [ -x "$bin" ] || continue
  case "$bin" in *.d) continue;; esac
  if command -v ldd >/dev/null 2>&1; then
    if ldd "$bin" 2>/dev/null | grep -q '=>'; then
      echo "ERROR: $bin is dynamically linked - must be static musl." >&2
      exit 1
    fi
  fi
done
echo "build: ok"
