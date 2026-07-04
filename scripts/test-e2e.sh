#!/usr/bin/env sh
# End-to-end / acceptance tests. For ADAD the E2E surface is the leak-test
# battery run against a booted image in QEMU: no clearnet, no DNS leak, no IPv6,
# killswitch fires on interface drop. These are the acceptance gates for the
# networking phases.
set -eu
cd "$(dirname "$0")/.."
if [ -f tests/e2e/run-leak-battery.sh ]; then
  sh tests/e2e/run-leak-battery.sh
  echo "e2e tests: ok"
else
  echo "ERROR: tests/e2e/run-leak-battery.sh not found." >&2
  echo "EP-005/EP-006 must provide the leak-test battery before E2E can pass." >&2
  exit 1
fi
