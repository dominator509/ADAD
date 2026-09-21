#!/usr/bin/env sh
# Audits Rust dependencies for known vulnerabilities and yanked crates.
set -eu
cd "$(dirname "$0")/.."
if [ ! -f Cargo.lock ]; then
  echo "ERROR: no Cargo.lock yet. EP-001 must create + commit the lockfile." >&2
  exit 1
fi
audit_version=$(cargo audit --version 2>/dev/null || true)
if ! printf '%s\n' "$audit_version" | grep -Eq '(^|[[:space:]])0\.22\.2([[:space:]]|$)'; then
  echo "ERROR: cargo-audit 0.22.2 is required. Install: cargo install cargo-audit --version 0.22.2 --locked" >&2
  echo "(Agent: adding this tool is allowed; record it in the Decision Log.)" >&2
  exit 1
fi
cargo audit --deny warnings
echo "dependency audit: ok"
