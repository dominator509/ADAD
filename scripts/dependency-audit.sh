#!/usr/bin/env sh
# Audits Rust dependencies for known vulnerabilities and yanked crates.
set -eu
cd "$(dirname "$0")/.."
if [ ! -f Cargo.lock ]; then
  echo "ERROR: no Cargo.lock yet. EP-001 must create + commit the lockfile." >&2
  exit 1
fi
if ! cargo audit --version >/dev/null 2>&1; then
  echo "ERROR: cargo-audit not installed. Run: cargo install cargo-audit" >&2
  echo "(Agent: adding this tool is allowed; record it in the Decision Log.)" >&2
  exit 1
fi
cargo audit
echo "dependency audit: ok"
