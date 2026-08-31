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
scripts/smoke-test.sh
# The E2E leak battery is a required repository control. It may explicitly omit
# the expensive image run during source-only verification, but a missing
# harness must never turn the full verifier green.
scripts/test-e2e.sh

echo "verify: ok"
