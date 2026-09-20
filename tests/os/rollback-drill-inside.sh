#!/usr/bin/env sh
# Internal helper for tests/os/rollback-drill.sh. Run inside adad-ep009-builder.
set -eu
cd /workspace

source_image="${ADAD_ROLLBACK_SOURCE_IMAGE:-build/adad.img}"
[ -f "$source_image" ] || {
  echo "ERROR: $source_image not found. Run scripts/build-image.sh first." >&2
  exit 1
}

workdir="$(mktemp -d)"
rollback_image="$workdir/adad-rollback.img"
cp "$source_image" "$rollback_image"

ADAD_BOOT_IMAGE="$rollback_image" sh tests/os/boot-smoke-inside.sh >/tmp/adad-rollback-boot.log
grep -q 'boot smoke: ok' /tmp/adad-rollback-boot.log

echo "rollback drill: ok"
