#!/usr/bin/env sh
# Prints a one-screen build status summary.
set -eu
cd "$(dirname "$0")/.."
echo "== ExecPlan status =="
for f in .agent/execplans/EP-*.md; do
  st=$(sed -n '/^---$/,/^---$/p' "$f" | grep '^status:' | head -n1 | cut -d: -f2 | tr -d ' ')
  printf '%-55s %s\n' "$f" "$st"
done
echo "== Last session =="
cat .agent/state/last-result.env
echo "== Loop state =="
cat .agent/state/build-state.env
echo "loop status: ok"
