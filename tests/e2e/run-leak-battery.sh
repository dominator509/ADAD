#!/usr/bin/env sh
# Leak-test battery entrypoint. Until EP-009 produces a bootable ADAD image, this
# runs the model-level leak assertions and records the on-image battery as pending.
set -eu
cd "$(dirname "$0")/../.."

sh tests/e2e/assert-leakguard-model.sh
sh tests/e2e/assert-agent-egress-guard.sh

# Source-only verification must not accidentally consume an ignored image from
# another checkout or source revision. Release CI sets ADAD_REQUIRE_IMAGE=1.
if [ "${ADAD_REQUIRE_IMAGE:-0}" != "1" ]; then
  echo "leak battery: on-image assertions not requested; source-only model checks passed"
  exit 0
fi

if [ ! -f build/adad.img ]; then
  echo "ERROR: release leak battery requires build/adad.img." >&2
  exit 1
fi

if [ ! -f tests/os/run-qemu-leak-battery.sh ]; then
  echo "ERROR: build/adad.img exists, but tests/os/run-qemu-leak-battery.sh is not available." >&2
  echo "EP-009/EP-010 must provide the booted-image leak runner before image e2e can pass." >&2
  exit 1
fi

sh tests/os/run-qemu-leak-battery.sh build/adad.img
echo "leak battery: ok"
