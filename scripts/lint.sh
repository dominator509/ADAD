#!/usr/bin/env sh
# Rust lint via clippy, denying warnings.
set -eu
cd "$(dirname "$0")/.."
if [ ! -f Cargo.toml ]; then
  echo "ERROR: no Cargo.toml yet. EP-001 must create the workspace before lint runs." >&2
  exit 1
fi
cargo clippy --workspace --all-targets --all-features -- -D warnings
echo "lint: ok"
