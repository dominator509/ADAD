#!/usr/bin/env sh
# Integration tests: crate-level tests in each crate's tests/ dir, plus the
# QEMU/KVM OS-boot smoke harness once EP-009 provides an image.
set -eu
cd "$(dirname "$0")/.."
if [ ! -f Cargo.toml ]; then
  echo "ERROR: no Cargo.toml yet. EP-001 creates the workspace first." >&2
  exit 1
fi
cargo test --workspace --tests

# OS integration test is opt-in: only run if an image + harness exist. Building
# an image every verify cycle is too slow, so it is gated on a built artifact.
if [ -f build/adad.img ] && [ -x tests/os/boot-smoke.sh ]; then
  tests/os/boot-smoke.sh
else
  echo "integration tests: (OS boot harness skipped - no build/adad.img yet)"
fi
echo "integration tests: ok"
