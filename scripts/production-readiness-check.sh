#!/usr/bin/env sh
# Production-readiness gate. Beyond `verify`, this asserts the release-shaped
# invariants: a bootable image exists, leak battery passed against it, docs and
# runbooks are present, and no plan is left incomplete. See
# PRODUCTION_READINESS.md for the human-readable checklist this automates.
set -eu
cd "$(dirname "$0")/.."

fail() { echo "production readiness: FAIL - $1" >&2; exit 1; }

# Release evidence is valid only for a clean checkout. Ignored build outputs
# are allowed, but source changes must force a new image and new evidence. The
# loop protocol deliberately updates this tracked bookkeeping file as the
# session's final operation, so it is not source input to the artifact.
source_status=$(git status --porcelain --untracked-files=all -- . \
  ':(exclude).agent/state/last-result.env')
[ -z "$source_status" ] || fail "source checkout is dirty"

# 1. Everything verify covers must pass in release-shaped mode.  Without these
# variables, verify intentionally performs source-only integration and E2E
# checks, which is insufficient for this gate even when an image is present.
ADAD_REQUIRE_IMAGE=1 ADAD_REQUIRE_VAULT=1 scripts/verify.sh >/dev/null

# 2. Every ExecPlan must be complete.
for f in .agent/execplans/EP-*.md; do
  st=$(sed -n '/^---$/,/^---$/p' "$f" | grep '^status:' | head -n1 | cut -d: -f2 | tr -d ' ')
  [ "$st" = "complete" ] || fail "$f is not complete (status=$st)"
done

# Plan bookkeeping is not evidence of product readiness. Every checkbox in
# the evidence-first readiness document must be explicitly closed before this
# executable gate can report success.
if grep -q -- '^- \[ \]' PRODUCTION_READINESS.md; then
  fail "PRODUCTION_READINESS.md still has unchecked evidence gates"
fi

# 3. A bootable image artifact must exist and the OS leak battery must have run
#    against it (EP-010 dry run produces build/adad.img + the pass marker).
[ -f build/adad.img ] || fail "no bootable image at build/adad.img"
[ -f build/leak-battery.pass ] || fail "leak battery has not passed against the image"
[ -f build/adad-image.provenance ] || fail "image provenance is missing"

source_sha=$(git rev-parse HEAD) || fail "cannot resolve source SHA"
source_tree=$(git rev-parse 'HEAD^{tree}') || fail "cannot resolve source tree"
provenance_source_sha=$(sed -n 's/^source_sha=//p' build/adad-image.provenance | head -n 1)
provenance_source_tree=$(sed -n 's/^source_tree=//p' build/adad-image.provenance | head -n 1)
provenance_image_sha=$(sed -n 's/^image_sha256=//p' build/adad-image.provenance | head -n 1)
provenance_debian_snapshot=$(sed -n 's/^debian_snapshot=//p' build/adad-image.provenance | head -n 1)
provenance_llama_server_sha=$(sed -n 's/^llama_server_sha256=//p' build/adad-image.provenance | head -n 1)
provenance_llama_model_sha=$(sed -n 's/^llama_model_sha256=//p' build/adad-image.provenance | head -n 1)
actual_image_sha=$(sha256sum build/adad.img | awk '{print $1}')
pass_battery=$(sed -n 's/^battery=//p' build/leak-battery.pass | head -n 1)
pass_source_sha=$(sed -n 's/^source_sha=//p' build/leak-battery.pass | head -n 1)
pass_image_sha=$(sed -n 's/^image_sha256=//p' build/leak-battery.pass | head -n 1)

[ "$provenance_source_sha" = "$source_sha" ] || fail "image was not built from current HEAD"
[ "$provenance_source_tree" = "$source_tree" ] || fail "image source tree does not match current HEAD"
[ "$provenance_image_sha" = "$actual_image_sha" ] || fail "image digest does not match provenance"
[ "$provenance_debian_snapshot" = "20260801T000000Z" ] || fail "image provenance does not identify the pinned Debian snapshot"
printf '%s\n' "$provenance_llama_server_sha" | grep -Eq '^[[:xdigit:]]{64}$' || fail "image provenance has no valid llama-server hash"
printf '%s\n' "$provenance_llama_model_sha" | grep -Eq '^[[:xdigit:]]{64}$' || fail "image provenance has no valid llama model hash"
[ "$pass_battery" = "on-image" ] || fail "leak marker is not an on-image result"
[ "$pass_source_sha" = "$source_sha" ] || fail "leak marker source SHA does not match HEAD"
[ "$pass_image_sha" = "$actual_image_sha" ] || fail "leak marker image digest does not match image"

# 4. Required operational docs present.
for d in DEPLOYMENT.md ROLLBACK.md OPERATIONS.md OBSERVABILITY.md SECURITY.md; do
  [ -f "$d" ] || fail "missing operational doc: $d"
done

echo "production readiness: ok"
