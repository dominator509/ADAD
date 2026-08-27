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

# Source-only verification may omit the expensive image run, but release CI
# must opt into a hard failure rather than silently treating it as green.
if [ "${ADAD_REQUIRE_IMAGE:-0}" = "1" ]; then
  [ -f build/adad.img ] || {
    echo "ERROR: release integration requires build/adad.img." >&2
    exit 1
  }
  [ -f build/adad-image.provenance ] || {
    echo "ERROR: release integration requires image provenance." >&2
    exit 1
  }
  [ -f tests/os/boot-smoke.sh ] || {
    echo "ERROR: release integration requires the boot-smoke harness." >&2
    exit 1
  }
  sh tests/os/boot-smoke.sh
else
  echo "integration tests: (source-only; ignored image artifacts are not used)"
fi
echo "integration tests: ok"
