#!/usr/bin/env sh
# Unit tests: pure-logic tests inside each crate (cargo's default test target).
set -eu
cd "$(dirname "$0")/.."
if [ ! -f Cargo.toml ]; then
  echo "ERROR: no Cargo.toml yet. EP-001 creates the workspace before tests run." >&2
  exit 1
fi
# --lib restricts to unit tests living beside the code; integration tests run
# separately in test-integration.sh.
cargo test --workspace --lib
echo "unit tests: ok"
