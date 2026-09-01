#!/usr/bin/env sh
# Fast executable smoke test: every shipped binary must expose a command
# surface, and safe local-only commands must reach library-backed behavior.
set -eu
cd "$(dirname "$0")/.."
host_os=$(uname -s 2>/dev/null || echo unknown)
if [ "$host_os" != "Linux" ]; then
  echo "smoke: skipped - host '$host_os' cannot execute Linux musl binaries"
  echo "smoke test: ok"
  exit 0
fi

BINDIR=target/x86_64-unknown-linux-musl/release
if [ ! -d "$BINDIR" ]; then
  echo "ERROR: no release binaries. Run scripts/build.sh first (EP-001+)." >&2
  exit 1
fi
BINDIR=$(cd "$BINDIR" && pwd)

# On Linux, every core tool built by the workspace must execute successfully.
for tool in forge leakguard agent-coding xmr-wallet vps-deploy persona metafuse git-spoof; do
  bin="$BINDIR/$tool"
  [ -f "$bin" ] || { echo "ERROR: smoke missing binary $bin" >&2; exit 1; }
  [ -x "$bin" ] || { echo "ERROR: smoke binary not executable $bin" >&2; exit 1; }
  help_output=$("$bin" --help) || { echo "smoke: $tool --help failed" >&2; exit 1; }
  case "$help_output" in
    *"Usage:"*) ;;
    *) echo "smoke: $tool did not expose a usage surface" >&2; exit 1 ;;
  esac
  echo "smoke: $tool ok"
done

"$BINDIR/leakguard" status >/dev/null || {
  echo "smoke: leakguard status failed" >&2
  exit 1
}

"$BINDIR/leakguard" wireguard status >/dev/null || {
  echo "smoke: leakguard wireguard status failed" >&2
  exit 1
}

"$BINDIR/metafuse" scrub README.md >/dev/null || {
  echo "smoke: metafuse scrub failed" >&2
  exit 1
}

ADAD_PSEUDONYM=smoke-user \
ADAD_GIT_AUTHOR_NAME=Smoke\ User \
ADAD_GIT_AUTHOR_EMAIL=smoke@example.invalid \
  "$BINDIR/git-spoof" rewrite-metadata smoke >/dev/null || {
  echo "smoke: git-spoof rewrite-metadata failed" >&2
  exit 1
}

git_smoke_dir=$(mktemp -d)
trap 'rm -rf "$git_smoke_dir"' EXIT HUP INT TERM
git -C "$git_smoke_dir" init -q
git -C "$git_smoke_dir" config commit.gpgsign false
printf '%s\n' 'ADAD smoke' > "$git_smoke_dir/tracked.txt"
git -C "$git_smoke_dir" add -- tracked.txt
(
  cd "$git_smoke_dir"
  ADAD_PSEUDONYM=smoke-user \
  ADAD_GIT_AUTHOR_NAME='Smoke User' \
  ADAD_GIT_AUTHOR_EMAIL=smoke@example.invalid \
    "$BINDIR/git-spoof" commit 'smoke commit' >/dev/null
)
git -C "$git_smoke_dir" show -s --format='%an%n%ae%n%aI%n%cn%n%ce%n%cI' |
  grep -Fx 'Smoke User' >/dev/null || { echo "smoke: git-spoof commit identity failed" >&2; exit 1; }
git -C "$git_smoke_dir" show -s --format='%an%n%ae%n%aI%n%cn%n%ce%n%cI' |
  grep -Fx 'smoke@example.invalid' >/dev/null || { echo "smoke: git-spoof commit email failed" >&2; exit 1; }
git -C "$git_smoke_dir" show -s --format='%aI%n%cI' |
  grep -Fx '2000-01-01T00:00:00Z' >/dev/null || { echo "smoke: git-spoof commit timestamp failed" >&2; exit 1; }

dms_smoke_image="$git_smoke_dir/dms.img"
printf 'LUKS\272\276\000\002' > "$dms_smoke_image"
truncate -s 256 "$dms_smoke_image"
dms_output=$("$BINDIR/leakguard" dms evaluate-image "$dms_smoke_image" 128 1000 1061 60)
printf '%s\n' "$dms_output" | grep -Fx 'dms=Expired header_wiped=true image_only=true' >/dev/null || {
  echo "smoke: leakguard dms image wipe failed" >&2
  exit 1
}

echo "smoke test: ok"
