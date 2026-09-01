#!/usr/bin/env sh
# Internal helper for scripts/build-image.sh. Run inside adad-ep009-builder.
set -eu

repo=/workspace
work=/tmp/adad-live-build-work
# Keep the target image's Debian inputs aligned with the immutable builder
# snapshot in live-build/builder/Dockerfile. Update both only after review.
debian_snapshot=20260801T000000Z
debian_mirror="https://snapshot.debian.org/archive/debian/${debian_snapshot}/"
debian_security_mirror="https://snapshot.debian.org/archive/debian-security/${debian_snapshot}/"

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
source_status=$(git status --porcelain --untracked-files=all -- . \
  ':(exclude).agent/state/last-result.env')
if [ -n "$source_status" ]; then
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
  --apt-options "-o Acquire::Check-Valid-Until=false --yes" \
  --mirror-bootstrap "$debian_mirror" \
  --mirror-chroot "$debian_mirror" \
  --mirror-chroot-security "$debian_security_mirror" \
  --mirror-binary "$debian_mirror" \
  --mirror-binary-security "$debian_security_mirror" \
  --security true \
  --updates false \
  --backports false \
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

# The local-first provider is a release requirement, not an optional image
# decoration. Runtime and model inputs are deliberately repo-relative so the
# container cannot silently copy an untracked host path into the image.
llama_tag="${ADAD_LLAMA_CPP_RELEASE_TAG:-b9892}"
llama_runtime_rel="${ADAD_LLAMA_RUNTIME_DIR:-build/tools/llama.cpp/$llama_tag}"
llama_model_rel="${ADAD_LLAMA_MODEL_SOURCE:-}"
case "$llama_runtime_rel" in
  /*|*..*)
    echo "ERROR: ADAD_LLAMA_RUNTIME_DIR must be a repo-relative path without '..'." >&2
    exit 1
    ;;
esac
case "$llama_model_rel" in
  "")
    echo "ERROR: ADAD_LLAMA_MODEL_SOURCE is required to build a release image." >&2
    echo "Provide a repo-relative GGUF model path; no model is fabricated by the builder." >&2
    exit 1
    ;;
  /*|*..*)
    echo "ERROR: ADAD_LLAMA_MODEL_SOURCE must be a repo-relative path without '..'." >&2
    exit 1
    ;;
esac
llama_runtime="$repo/$llama_runtime_rel"
llama_model="$repo/$llama_model_rel"
[ -x "$llama_runtime/llama-server" ] || {
  echo "ERROR: llama-server runtime is missing at $llama_runtime/llama-server." >&2
  echo "Fetch or supply the reviewed runtime before building the release image." >&2
  exit 1
}
[ -f "$llama_model" ] || {
  echo "ERROR: local model artifact is missing at $llama_model." >&2
  exit 1
}
llama_server_sha256=$(sha256sum "$llama_runtime/llama-server" | awk '{print $1}')
llama_model_sha256=$(sha256sum "$llama_model" | awk '{print $1}')
llama_image_dir="$work/config/includes.chroot/usr/local/lib/adad/llama/$llama_tag"
mkdir -p "$llama_image_dir"
cp -R "$llama_runtime/." "$llama_image_dir/"
ln -s "../lib/adad/llama/$llama_tag/llama-server" "$bin_dir/llama-server"
model_image_dir="$work/config/includes.chroot/var/lib/adad/models"
mkdir -p "$model_image_dir"
cp "$llama_model" "$model_image_dir/default.gguf"
chown nobody:nogroup "$model_image_dir/default.gguf"
chmod 0640 "$model_image_dir/default.gguf"

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
  printf 'debian_snapshot=%s\n' "$debian_snapshot"
  printf 'debian_mirror=%s\n' "$debian_mirror"
  printf 'debian_security_mirror=%s\n' "$debian_security_mirror"
  printf 'llama_tag=%s\n' "$llama_tag"
  printf 'llama_runtime_path=%s\n' "$llama_runtime_rel"
  printf 'llama_server_sha256=%s\n' "$llama_server_sha256"
  printf 'llama_model_path=%s\n' "$llama_model_rel"
  printf 'llama_model_sha256=%s\n' "$llama_model_sha256"
} > "$repo/build/adad-image.provenance.tmp"
mv "$repo/build/adad-image.provenance.tmp" "$repo/build/adad-image.provenance"
echo "image build: ok"
