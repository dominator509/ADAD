---
id: EP-007
status: not-started
depends_on: [EP-006]
verify: scripts/verify.sh
---

# EP-007 — Testing Hardening

## 1. Purpose / Big Picture
Raise coverage and reliability: fill unit/integration gaps, add regression tests
for the security-critical flows (killswitch, DMS clock-freeze, vault upgrade,
egress guard), add failure-mode tests, define a flaky-test policy, and ensure CI
runs the full suite. After this plan the suite reliably catches regressions in
the invariants that matter most.

## 2. Scope
- Regression tests for every previously-fixed bug and every critical flow.
- Failure-mode tests: tunnel drop mid-request, provider timeout, vault
  wrong-passphrase, DMS expiry, RPC/SSH failure.
- Flaky-test policy + deterministic seeding.
- CI runs `scripts/verify.sh` including integration.

## 3. Non-goals
- No new features. No new subsystems. No perf tuning (EP-010 reviews perf).

## 4. Context and Orientation
TESTING.md defines the pyramid, mocking, and the flaky policy. The leak battery
belongs to EP-006/EP-009; here we harden unit/integration/regression coverage.

## 5. Files to Read First
- TESTING.md, SPEC-006, the existing `crates/*/tests`, EP-002..006 acceptance
  criteria.

## 6. Files to Change
- `crates/*/tests/regression_*.rs`, `crates/*/tests/failure_*.rs`
- CI config (ensure integration runs)
- a `TESTING` note updating the flaky policy if refined

## 7. Interfaces and Contracts
- Regression tests named `regression_<area>`; failure-mode tests named
  `failure_<area>`. Deterministic (seeded); no network egress.

## 8. Milestones

### M1 — Coverage gap sweep
- Goal: identify and fill missing unit/integration coverage per crate.
- Files: new tests across crates.
- Validation: `scripts/test-unit.sh && scripts/test-integration.sh`
- Expected: both green with added tests.
- Recovery: add the smallest test that covers the gap; do not refactor code.

### M2 — Critical-flow regression tests
- Goal: lock in killswitch fail-closed, DMS clock-freeze resistance, vault
  upgrade, egress guard, git-spoof stability.
- Files: `regression_*` tests.
- Validation: `cargo test --workspace --test 'regression_*'` (or per-crate)
- Expected: all pass; none `#[ignore]`-d.
- Recovery: if a regression test reveals a real bug, that is a finding — fix the
  code narrowly, keep the test.

### M3 — Failure-mode tests
- Goal: graceful, fail-closed behavior under drop/timeout/wrong-passphrase/RPC
  failure.
- Files: `failure_*` tests.
- Validation: `cargo test --workspace`
- Expected: each failure maps to the right typed error; no partial-success leak.
- Recovery: ensure fail-closed semantics (SPEC-006).

### M4 — Flaky policy + CI full suite
- Goal: deterministic seeding; CI runs the full verify.
- Files: CI config; seeding helpers.
- Validation: `scripts/verify.sh`
- Expected: `verify: ok`; CI green.
- Recovery: seed any nondeterministic test; quarantine policy per TESTING.md.

## 9. Concrete Steps
1. Sweep coverage per crate; add unit/integration tests.
2. Add critical-flow regression tests.
3. Add failure-mode tests.
4. Ensure deterministic seeding; wire CI to the full suite; run verify.

## 10. Validation and Acceptance
- [ ] Coverage gaps filled; unit + integration green.
- [ ] Critical-flow regression tests present and passing (none ignored).
- [ ] Failure-mode tests present and passing.
- [ ] CI runs the full `scripts/verify.sh`.
- [ ] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
Tests are deterministic and self-cleaning. Re-runs are stable. A discovered bug
is fixed narrowly with its regression test retained.

## 12. Progress
- [ ] M1 — coverage gap sweep
- [ ] M2 — critical-flow regression tests
- [ ] M3 — failure-mode tests
- [ ] M4 — flaky policy + CI full suite
- [ ] verify + status set to complete

## 13. Surprises & Discoveries
(Record any real bug surfaced by hardening and its fix.)

## 14. Decision Log
(Record seeding approach, flaky quarantine rule, CI matrix.)

## 15. Outcomes & Retrospective
(Filled at completion — coverage posture and residual gaps.)
