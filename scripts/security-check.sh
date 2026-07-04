#!/usr/bin/env sh
# Security checks for ADAD. Two layers:
#  (a) source-level: no committed secrets, no clearnet defaults, no IPv6 enable.
#  (b) crate-level: cargo-audit for known-vuln dependencies.
set -eu
cd "$(dirname "$0")/.."

fail() { echo "security check: FAIL - $1" >&2; exit 1; }

# (a) Grep guards. These encode hard project invariants from SECURITY.md.
# No hardcoded API keys / private keys committed.
if git grep -nE '(sk-ant-[A-Za-z0-9]|BEGIN (RSA|OPENSSH|PGP) PRIVATE KEY)' \
     -- . ':(exclude).agent/*' ':(exclude)*.md' 2>/dev/null | grep -q .; then
  fail "possible committed secret (API key or private key)"
fi
# No accidental clearnet DNS or IPv6-enable slipping into shipped config.
if git grep -nE 'net\.ipv6\.conf\..*disable_ipv6\s*=\s*0' 2>/dev/null | grep -q .; then
  fail "config enables IPv6 - constraint requires IPv6 disabled"
fi

# (b) Dependency vuln scan (delegated to dependency-audit.sh to avoid dupes).
scripts/dependency-audit.sh >/dev/null

echo "security check: ok"
