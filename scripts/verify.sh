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
scripts/smoke-test.sh
# E2E leak battery runs only when its harness exists (post EP-005/006).
if [ -f tests/e2e/run-leak-battery.sh ]; then
  scripts/test-e2e.sh
else
  echo "verify: (e2e leak battery skipped - harness not present yet)"
fi

echo "verify: ok"
