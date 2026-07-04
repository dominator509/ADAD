#!/usr/bin/env sh
# Installs build/host dependencies for ADAD development.
# Rust toolchain + host packages for live-build and QEMU integration testing.
set -eu
cd "$(dirname "$0")/.."

if ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: cargo/rustup not installed." >&2
  echo "Install via: https://rustup.rs  (then re-run scripts/install.sh)" >&2
  echo "ADAD requires the stable toolchain plus musl target for static bins." >&2
  exit 1
fi

# Static-binary target (torsocks-free static Rust bins, per constraints).
rustup target add x86_64-unknown-linux-musl

# Host tooling required by later phases. Fail loud if absent; do NOT auto-apt
# in an agent session without permission (see AGENTS.md production data rules).
missing=""
for t in qemu-system-x86_64 mksquashfs lb cryptsetup losetup mkfs.ext4 mount umount truncate; do
  command -v "$t" >/dev/null 2>&1 || missing="$missing $t"
done
if [ -n "$missing" ]; then
  echo "install: host tools missing:$missing" >&2
  echo "install: on Debian: sudo apt install qemu-system-x86 squashfs-tools live-build cryptsetup util-linux e2fsprogs coreutils" >&2
  echo "install: install them, then re-run. (Agent: this is a STOP if apt is unavailable.)" >&2
  exit 1
fi

# Workspace deps (no-op if Cargo.toml not yet created by EP-001).
if [ -f Cargo.toml ]; then
  cargo fetch --locked
fi

echo "install: ok"
