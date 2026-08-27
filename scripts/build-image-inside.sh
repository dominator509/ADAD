#!/usr/bin/env sh
# Internal helper for scripts/build-image.sh. Run inside adad-ep009-builder.
set -eu

repo=/workspace
work=/tmp/adad-live-build-work

cd "$repo"
# The builder commonly runs as root against a checkout owned by the CI runner.
# Allow Git's read-only provenance queries for that mounted checkout.
git config --global --add safe.directory "$repo" >/dev/null 2>&1 || true
[ -d live-build/config ] || {
  echo "ERROR: live-build/config is missing." >&2
  exit 1
}

source_sha=$(git rev-parse HEAD 2>/dev/null) || {
  echo "ERROR: image build requires a Git checkout." >&2
  exit 1
}
source_tree=$(git rev-parse 'HEAD^{tree}' 2>/dev/null) || {
  echo "ERROR: image build could not resolve the source tree." >&2
  exit 1
}
if [ -n "$(git status --porcelain --untracked-files=all)" ]; then
  echo "ERROR: image build requires a clean source checkout; provenance cannot bind dirty files." >&2
  exit 1
fi

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
image_sha256=$(sha256sum "$repo/build/adad.img" | awk '{print $1}')
{
  printf 'source_sha=%s\n' "$source_sha"
  printf 'source_tree=%s\n' "$source_tree"
  printf 'image_sha256=%s\n' "$image_sha256"
  printf 'source_date_epoch=%s\n' "$SOURCE_DATE_EPOCH"
} > "$repo/build/adad-image.provenance.tmp"
mv "$repo/build/adad-image.provenance.tmp" "$repo/build/adad-image.provenance"
echo "image build: ok"
