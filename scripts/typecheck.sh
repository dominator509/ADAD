#!/usr/bin/env sh
# Static validation for Rust = type/borrow check without producing binaries.
set -eu
cd "$(dirname "$0")/.."
if [ ! -f Cargo.toml ]; then
  echo "ERROR: no Cargo.toml yet. EP-001 must create the workspace first." >&2
  exit 1
fi
cargo check --workspace --all-targets --all-features
echo "typecheck: ok"
