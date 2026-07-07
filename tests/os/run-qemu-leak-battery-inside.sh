#!/usr/bin/env sh
# Internal helper for tests/os/run-qemu-leak-battery.sh. Run inside
# adad-ep009-builder.
set -eu
cd /workspace

image="${1:-build/adad.img}"
[ -f "$image" ] || {
  echo "ERROR: $image not found. Run scripts/build-image.sh first." >&2
  exit 1
}

tmp="${TMPDIR:-/tmp}/adad-leak-battery.$$"
log="${TMPDIR:-/tmp}/adad-leak-battery.log"
rm -rf "$tmp"
mkdir -p "$tmp"
rm -f "$log" build/leak-battery.pass
trap 'rm -rf "$tmp"' EXIT

xorriso -osirrox on -indev "$image" \
  -extract /live/vmlinuz "$tmp/vmlinuz" >/tmp/adad-leak-xorriso-kernel.log 2>&1
xorriso -osirrox on -indev "$image" \
  -extract /live/initrd.img "$tmp/initrd.img" >/tmp/adad-leak-xorriso-initrd.log 2>&1

set +e
timeout "${ADAD_LEAK_BATTERY_TIMEOUT:-240}" qemu-system-x86_64 \
  -m "${ADAD_QEMU_MEM:-1536}" \
  -smp "${ADAD_QEMU_SMP:-2}" \
  -machine accel=tcg \
  -display none \
  -serial stdio \
  -no-reboot \
  -nic user,model=virtio-net-pci \
  -cdrom "$image" \
  -kernel "$tmp/vmlinuz" \
  -initrd "$tmp/initrd.img" \
  -append "boot=live components quiet toram console=ttyS0,115200n8 adad.leak_battery=1" \
  >"$log" 2>&1
status=$?
set -e

for marker in \
  'adad-leak-battery: static-tools: pass' \
  'adad-leak-battery: ipv6: pass' \
  'adad-leak-battery: killswitch: pass' \
  'adad-leak-battery: dns-discovery: pass' \
  'adad-leak-battery: tor-default: pass' \
  'adad-leak-battery: mac: pass' \
  'adad-leak-battery: clearnet: pass' \
  'adad-leak-battery: drop: pass' \
  'adad-leak-battery: all: pass'
do
  if ! grep -q "$marker" "$log"; then
    echo "ERROR: QEMU leak battery did not report marker: $marker" >&2
    echo "qemu exit: $status" >&2
    tail -n 120 "$log" >&2
    exit 1
  fi
done

printf 'leak battery pass\n' > build/leak-battery.pass.tmp
mv build/leak-battery.pass.tmp build/leak-battery.pass
echo "qemu leak battery: ok"
