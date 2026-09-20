#!/usr/bin/env sh
# Validates the workspace is ready for an agent session.
set -eu
cd "$(dirname "$0")/.."

fail() { echo "preflight: FAIL - $1" >&2; exit 1; }

# 1. Running from repo root (must see the control-plane files).
[ -f AGENTS.md ] || fail "AGENTS.md not found - not at repo root"
[ -f COMMANDS.md ] || fail "COMMANDS.md not found"
[ -d .agent/state ] || fail ".agent/state/ directory missing"

# 2. Required state files present.
for f in .agent/state/build-state.env .agent/state/last-result.env \
         .agent/state/blockers.md; do
  [ -f "$f" ] || fail "missing state file: $f"
done

# 3. Package managers / toolchains available (warn-only until EP-001 pins them).
command -v cargo >/dev/null 2>&1 || echo "preflight: WARN - cargo not found (required by EP-001+)" >&2

# 4. Active ExecPlan front matter valid, if one is named or discoverable.
plan="${1:-}"
[ -n "$plan" ] || plan=$(scripts/next-step.sh 2>/dev/null || true)
case "$plan" in
  ""|DONE|BLOCKED:*) : ;;   # nothing to validate
  *)
    [ -f "$plan" ] || fail "named ExecPlan does not exist: $plan"
    fm=$(sed -n '/^---$/,/^---$/p' "$plan")
    echo "$fm" | grep -q '^id:' || fail "ExecPlan $plan missing id in front matter"
    echo "$fm" | grep -q '^status:' || fail "ExecPlan $plan missing status"
    echo "$fm" | grep -q '^depends_on:' || fail "ExecPlan $plan missing depends_on"
    echo "$fm" | grep -q '^verify:' || fail "ExecPlan $plan missing verify"
    ;;
esac

echo "preflight: ok"
