#!/usr/bin/env sh
# Internal helper for tests/os/boot-smoke.sh. Run inside adad-ep009-builder.
set -eu
cd /workspace

image="${ADAD_BOOT_IMAGE:-build/adad.img}"

[ -f "$image" ] || {
  echo "ERROR: $image not found. Run scripts/build-image.sh first." >&2
  exit 1
}

log="${TMPDIR:-/tmp}/adad-boot-smoke.log"
rm -f "$log"

set +e
timeout "${ADAD_BOOT_TIMEOUT:-180}" qemu-system-x86_64 \
  -m "${ADAD_QEMU_MEM:-1536}" \
  -smp "${ADAD_QEMU_SMP:-2}" \
  -machine accel=tcg \
  -display none \
  -serial stdio \
  -no-reboot \
  -nic user,model=virtio-net-pci \
  -cdrom "$image" \
  -boot d >"$log" 2>&1
status=$?
set -e

if grep -q 'adad-killswitch: armed' "$log" \
  && grep -q 'adad-ipv6: disabled' "$log" \
  && grep -q 'adad-mac: randomized' "$log"; then
  echo "boot smoke: ok"
  exit 0
fi

echo "ERROR: QEMU boot did not report the ADAD hardening posture." >&2
echo "qemu exit: $status" >&2
tail -n 80 "$log" >&2
exit 1
