#!/usr/bin/env sh
# Production-readiness gate. Beyond `verify`, this asserts the release-shaped
# invariants: a bootable image exists, leak battery passed against it, docs and
# runbooks are present, and no plan is left incomplete. See
# PRODUCTION_READINESS.md for the human-readable checklist this automates.
set -eu
cd "$(dirname "$0")/.."

fail() { echo "production readiness: FAIL - $1" >&2; exit 1; }

# 1. Everything verify covers must pass.
scripts/verify.sh >/dev/null

# 2. Every ExecPlan must be complete.
for f in .agent/execplans/EP-*.md; do
  st=$(sed -n '/^---$/,/^---$/p' "$f" | grep '^status:' | head -n1 | cut -d: -f2 | tr -d ' ')
  [ "$st" = "complete" ] || fail "$f is not complete (status=$st)"
done

# 3. A bootable image artifact must exist and the OS leak battery must have run
#    against it (EP-010 dry run produces build/adad.img + the pass marker).
[ -f build/adad.img ] || fail "no bootable image at build/adad.img"
[ -f build/leak-battery.pass ] || fail "leak battery has not passed against the image"

# 4. Required operational docs present.
for d in DEPLOYMENT.md ROLLBACK.md OPERATIONS.md OBSERVABILITY.md SECURITY.md; do
  [ -f "$d" ] || fail "missing operational doc: $d"
done

echo "production readiness: ok"
