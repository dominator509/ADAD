#!/usr/bin/env sh
# Internal helper for scripts/build-image.sh. Run inside adad-ep009-builder.
set -eu

repo=/workspace
work=/tmp/adad-live-build-work

cd "$repo"
[ -d live-build/config ] || {
  echo "ERROR: live-build/config is missing." >&2
  exit 1
}

rm -rf "$work"
mkdir -p "$work" "$repo/build"

cd "$work"
export SOURCE_DATE_EPOCH="${ADAD_SOURCE_DATE_EPOCH:-946684800}"
export TZ=UTC

lb config \
  --distribution trixie \
  --architecture amd64 \
  --binary-image iso-hybrid \
  --image-name adad \
  --iso-volume ADAD \
  --iso-application ADAD \
  --iso-preparer ADAD \
  --iso-publisher ADAD \
  --debian-installer none \
  --archive-areas main \
  --apt-recommends false \
  --checksums sha256 \
  --source false \
  --utc-time true \
  --bootappend-live "boot=live components quiet toram console=ttyS0,115200n8"

cp -R "$repo/live-build/config/." "$work/config/"

if [ -d "$repo/live-build/hooks" ]; then
  mkdir -p "$work/config/hooks/normal"
  for hook in "$repo"/live-build/hooks/*.hook.chroot; do
    [ -f "$hook" ] || continue
    cp "$hook" "$work/config/hooks/normal/$(basename "$hook")"
    chmod 0755 "$work/config/hooks/normal/$(basename "$hook")"
  done
fi

bin_dir="$work/config/includes.chroot/usr/local/bin"
mkdir -p "$bin_dir"
missing=""
for tool in agent-coding forge git-spoof leakguard metafuse persona vps-deploy xmr-wallet; do
  src="$repo/target/x86_64-unknown-linux-musl/release/$tool"
  if [ ! -f "$src" ]; then
    missing="$missing $tool"
    continue
  fi
  cp "$src" "$bin_dir/$tool"
  chmod 0755 "$bin_dir/$tool"
done

if [ -n "$missing" ]; then
  echo "ERROR: missing static release tools:$missing" >&2
  echo "Run scripts/build.sh before scripts/build-image.sh." >&2
  exit 1
fi

lb build

artifact=$(
  find "$work" -maxdepth 1 -type f \( -name '*.iso' -o -name '*.img' \) \
    | sort \
    | head -n 1
)

if [ -z "$artifact" ]; then
  echo "ERROR: live-build completed without an image artifact." >&2
  exit 1
fi

cp "$artifact" "$repo/build/adad.img.tmp"
mv "$repo/build/adad.img.tmp" "$repo/build/adad.img"
echo "image build: ok"
