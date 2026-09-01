#!/usr/bin/env sh
# Full local validation sequence. This is the `verify` command referenced by
# every ExecPlan's front matter. Runs the whole gate in order and stops at the
# first failure.
set -eu
cd "$(dirname "$0")/.."

scripts/preflight.sh
scripts/format-check.sh
scripts/lint.sh
scripts/typecheck.sh
scripts/test-unit.sh
scripts/test-integration.sh
scripts/build.sh
scripts/security-check.sh
scripts/dependency-audit.sh
# The fetcher uses the release tag in repository-relative paths. Keep this
# fail-closed regression in the source gate so a future edit cannot turn an
# operator-provided tag into a path traversal or recursive-delete target.
if ADAD_LLAMA_CPP_RELEASE_TAG='../escape' sh scripts/fetch-llama-cpp-runtime.sh >/dev/null 2>&1; then
  echo "ERROR: llama runtime input validation accepted a path-like release tag." >&2
  exit 1
fi
echo "llama runtime input validation: ok"
# The on-image leak battery uses ping as its controlled clearnet probe. Keep
# the target package list and runtime assertion coupled so a missing binary
# cannot be mistaken for a successful blocked-traffic test.
grep -Fx 'iputils-ping' live-build/config/package-lists/adad-base.list.chroot >/dev/null || {
  echo "ERROR: target image is missing the iputils-ping package required by the leak battery." >&2
  exit 1
}
grep -Fx 'require_cmd ping' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image leak battery does not require ping before using it." >&2
  exit 1
}
echo "image leak-probe dependency check: ok"
# The drop probe must exercise a real interface transition. A missing
# interface or ignored transition is not evidence that the killswitch reacted.
grep -Fx '[ -n "$drop_iface" ] || console_fail "drop-interface"' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image leak battery can pass without a non-loopback interface." >&2
  exit 1
}
grep -Fx 'ip link set dev "$drop_iface" down 2>/dev/null || console_fail "drop-interface-down"' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image leak battery does not fail when interface down fails." >&2
  exit 1
}
echo "image interface-drop dependency check: ok"
# The boot posture must not claim MAC randomization when there is no usable
# interface or when an assignment transition fails. Keep the boot marker and
# leak-battery assertion tied to those fail-closed checks.
grep -Fx 'mac_ifaces=0' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: boot hardening can claim MAC randomization without an interface." >&2
  exit 1
}
grep -Fx '  mac_ifaces=$((mac_ifaces + 1))' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: boot hardening does not count non-loopback interfaces." >&2
  exit 1
}
grep -Fx '  ip link set dev "$iface" down 2>/dev/null || exit 1' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: boot hardening can ignore a failed MAC transition." >&2
  exit 1
}
grep -Fx '  ip link set dev "$iface" address "$mac" 2>/dev/null || exit 1' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: boot hardening can ignore a failed MAC assignment." >&2
  exit 1
}
grep -Fx '  ip link set dev "$iface" up 2>/dev/null || exit 1' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: boot hardening can ignore a failed MAC restore." >&2
  exit 1
}
grep -Fx '[ "$mac_ifaces" -gt 0 ] || exit 1' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: boot hardening lacks a non-loopback-interface fail-closed check." >&2
  exit 1
}
grep -Fx 'mac_iface_count=0' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image MAC smoke can pass without observing an interface." >&2
  exit 1
}
grep -Fx '  mac_iface_count=$((mac_iface_count + 1))' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image MAC smoke does not count non-loopback interfaces." >&2
  exit 1
}
grep -Fx '[ "$mac_iface_count" -gt 0 ] || console_fail "mac-interface"' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image MAC smoke lacks a non-loopback-interface failure." >&2
  exit 1
}
echo "image mac-randomization dependency check: ok"
# The hardening hook uses sysctl for IPv6 policy checks. Keep its target
# package explicit so a missing command cannot masquerade as a passing probe.
grep -Fx 'procps' live-build/config/package-lists/adad-base.list.chroot >/dev/null || {
  echo "ERROR: target image is missing the procps package required by sysctl checks." >&2
  exit 1
}
grep -Fx 'require_cmd sysctl' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image hardening battery does not require sysctl before using it." >&2
  exit 1
}
echo "image sysctl dependency check: ok"
# Do not let a pipeline hide a non-zero application exit: the image smoke
# requires both a successful --help process and an actual usage surface.
grep -Fx '  help_output=$("/usr/local/bin/$tool" --help 2>/dev/null) || console_fail "help-$tool"' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image static-tool smoke can mask a failed --help exit." >&2
  exit 1
}
echo "image help-exit dependency check: ok"
# Runtime downloads are release inputs. Keep both metadata and asset fetches
# on HTTPS, including redirects, so a changed release response cannot add a
# cleartext egress path to the builder.
grep -Fx '  https://*) ;;' scripts/fetch-llama-cpp-runtime.sh >/dev/null || {
  echo "ERROR: llama runtime fetcher does not require HTTPS asset URLs." >&2
  exit 1
}
proto_count=$(grep -F -- "--proto '=https'" scripts/fetch-llama-cpp-runtime.sh | wc -l | tr -d '[:space:]')
redir_count=$(grep -F -- "--proto-redir '=https'" scripts/fetch-llama-cpp-runtime.sh | wc -l | tr -d '[:space:]')
[ "$proto_count" -ge 2 ] && [ "$redir_count" -ge 2 ] || {
  echo "ERROR: llama runtime downloads do not enforce HTTPS redirects." >&2
  exit 1
}
echo "llama runtime HTTPS transport check: ok"
# Image inputs are supplied through ignored build paths. Keep their resolved
# targets inside the checked-out tree and reject symlinks before the builder
# copies bytes into the release image.
grep -Fx 'repo_real=$(readlink -f -- "$repo") || {' scripts/build-image-inside.sh >/dev/null || {
  echo "ERROR: image builder does not resolve the checkout before copying release inputs." >&2
  exit 1
}
grep -Fx 'ensure_repo_path "$llama_runtime"' scripts/build-image-inside.sh >/dev/null || {
  echo "ERROR: image builder does not bind the llama runtime to the checkout." >&2
  exit 1
}
grep -Fx 'ensure_repo_path "$llama_model"' scripts/build-image-inside.sh >/dev/null || {
  echo "ERROR: image builder does not bind the model artifact to the checkout." >&2
  exit 1
}
grep -Fx '[ ! -L "$llama_model" ] || {' scripts/build-image-inside.sh >/dev/null || {
  echo "ERROR: image builder can follow a model symlink into an unreviewed path." >&2
  exit 1
}
grep -Fx 'runtime_symlink=$(find "$llama_runtime" -type l -print -quit)' scripts/build-image-inside.sh >/dev/null || {
  echo "ERROR: image builder does not reject symlinks in the runtime tree." >&2
  exit 1
}
echo "image input provenance check: ok"
# The minimum-system simulator can fetch a reviewed llama runtime from inside
# the repo-owned builder. Keep the builder's explicit network/archive tools
# coupled to that fetch path rather than relying on incidental base-image state.
grep -Fx "      curl \\" live-build/builder/Dockerfile >/dev/null || {
  echo "ERROR: image builder is missing curl for the llama HTTPS fetch path." >&2
  exit 1
}
grep -Fx "      unzip \\" live-build/builder/Dockerfile >/dev/null || {
  echo "ERROR: image builder is missing unzip for ZIP llama runtime archives." >&2
  exit 1
}
echo "image builder llama tools: ok"
# Keep GitHub Actions inputs immutable. The comments retain the human-facing
# release labels while the commit IDs make the workflow source reproducible.
for action_pin in \
  '        uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4' \
  '        uses: dtolnay/rust-toolchain@06e5a564a0556e338780f5aecf2e7dcc9b267f07 # 1.90.0' \
  '        uses: taiki-e/install-action@8e38755317fb11cc24a0cd3b573a64008362d207 # cargo-audit' \
  '        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4'
do
  grep -Fx "$action_pin" .github/workflows/ci.yml >/dev/null || {
    echo "ERROR: CI workflow contains an unpinned or unexpected action reference." >&2
    exit 1
  }
done
action_lines=$(grep -E '^[[:space:]]+uses:' .github/workflows/ci.yml || true)
while IFS= read -r action_line; do
  case "$action_line" in
    '        uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4'|'        uses: dtolnay/rust-toolchain@06e5a564a0556e338780f5aecf2e7dcc9b267f07 # 1.90.0'|'        uses: taiki-e/install-action@8e38755317fb11cc24a0cd3b573a64008362d207 # cargo-audit'|'        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4')
      ;;
    *)
      echo "ERROR: CI workflow contains an unexpected action reference: $action_line" >&2
      exit 1
      ;;
  esac
done <<EOF
$action_lines
EOF
echo "workflow action pins: ok"
# Required inference acceptance must enforce the product's documented lower
# bound; optional exploratory runs may still record out-of-band measurements.
grep -Fx '  minimum_tok_s=4.0' scripts/min-system-sim-inside.sh >/dev/null || {
  echo "ERROR: required inference acceptance has no SPEC-000 throughput floor." >&2
  exit 1
}
grep -F 'if [ "$require_inference" = "1" ] && ! awk' scripts/min-system-sim-inside.sh >/dev/null || {
  echo "ERROR: required inference acceptance does not guard its throughput floor." >&2
  exit 1
}
grep -F 'BEGIN { exit !(actual >= minimum) }' scripts/min-system-sim-inside.sh >/dev/null || {
  echo "ERROR: required inference acceptance does not enforce the throughput floor." >&2
  exit 1
}
echo "inference throughput gate: ok"
# Pull requests must exercise the real disposable LUKS runtime. Keep the
# source job's package installation and fail-closed environment coupled so a
# future workflow edit cannot restore a green privileged-test skip.
source_ci_block=$(awk '/^  source-verify:/{in_source=1} /^  release-image:/{in_source=0} in_source {print}' .github/workflows/ci.yml)
printf '%s\n' "$source_ci_block" | grep -Fx '      - name: Install vault integration tools' >/dev/null || {
  echo "ERROR: hosted source verification does not install vault integration tools." >&2
  exit 1
}
printf '%s\n' "$source_ci_block" | grep -Fx '        run: sudo apt-get update && sudo apt-get install -y cryptsetup e2fsprogs util-linux' >/dev/null || {
  echo "ERROR: hosted source verification does not install the complete vault toolchain." >&2
  exit 1
}
printf '%s\n' "$source_ci_block" | grep -Fx '      - name: Verify required vault integration' >/dev/null || {
  echo "ERROR: hosted source verification has no required vault integration step." >&2
  exit 1
}
printf '%s\n' "$source_ci_block" | grep -Fx '          ADAD_REQUIRE_VAULT: "1"' >/dev/null || {
  echo "ERROR: hosted source verification can silently skip vault integration." >&2
  exit 1
}
printf '%s\n' "$source_ci_block" | grep -Fx '            "$cargo_bin" test -p forge --tests' >/dev/null || {
  echo "ERROR: hosted source verification does not execute the Forge integration suite." >&2
  exit 1
}
echo "hosted vault integration gate: ok"
scripts/smoke-test.sh
# The E2E leak battery is a required repository control. It may explicitly omit
# the expensive image run during source-only verification, but a missing
# harness must never turn the full verifier green.
scripts/test-e2e.sh

echo "verify: ok"
