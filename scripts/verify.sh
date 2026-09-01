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
scripts/smoke-test.sh
# The E2E leak battery is a required repository control. It may explicitly omit
# the expensive image run during source-only verification, but a missing
# harness must never turn the full verifier green.
scripts/test-e2e.sh

echo "verify: ok"
