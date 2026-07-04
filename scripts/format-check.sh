#!/usr/bin/env sh
# Rust formatting check (non-mutating).
set -eu
cd "$(dirname "$0")/.."
if [ ! -f Cargo.toml ]; then
  echo "ERROR: no Cargo.toml yet. EP-001 must create the workspace first." >&2
  exit 1
fi
cargo fmt --all --check
echo "format check: ok"
