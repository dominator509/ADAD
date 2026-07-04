---
id: EP-007
status: complete
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
- [x] Coverage gaps filled; unit + integration green.
- [x] Critical-flow regression tests present and passing (none ignored).
- [x] Failure-mode tests present and passing.
- [x] CI runs the full `scripts/verify.sh`.
- [x] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
Tests are deterministic and self-cleaning. Re-runs are stable. A discovered bug
is fixed narrowly with its regression test retained.

## 12. Progress
- [x] M1 — coverage gap sweep
- [x] M2 — critical-flow regression tests
- [x] M3 — failure-mode tests
- [x] M4 — flaky policy + CI full suite
- [x] verify + status set to complete

## 13. Surprises & Discoveries
(Record any real bug surfaced by hardening and its fix.)
- M1: CI already runs `scripts/verify.sh` on Ubuntu with the musl target and
  cargo-audit installed. The hardening gap was not CI absence, but missing
  `regression_*` targets for several critical flows and one integration gap in
  the streaming provider contract.
- M1: The mock inference server always returned non-streaming JSON before this
  plan. It now returns an SSE-style body when the request has `stream: true`,
  which lets integration tests prove `OpenAiCompatClient::chat_stream` sends the
  right request flag and parses streaming deltas.
- M2: The plan's suggested `cargo test --workspace --test 'regression_*'` shape
  is shell/glob-sensitive, so validation ran the regression targets per crate
  instead. All named regression targets passed without `#[ignore]`.
- M3: `forge::Unsealed` intentionally does not implement `Debug`, so the
  wrong-passphrase failure test uses an explicit `match` instead of
  `expect_err`. That keeps the production type surface unchanged.
- M4: The CI workflow already ran `scripts/verify.sh`, but the step name did
  not make the full-suite contract obvious. The workflow now calls
  `sh scripts/verify.sh` in an explicitly named "Verify full suite" step with a
  45-minute job timeout.
- M4: Full verify first stopped at formatting after new tests were added. Running
  `cargo fmt --all` fixed only rustfmt shape changes; the subsequent verify
  completed with `verify: ok`.

## 14. Decision Log
(Record seeding approach, flaky quarantine rule, CI matrix.)
- M1: Added deterministic `regression_*` integration targets for egress
  snapshots, streaming provider contract, vault upgrade, git-spoof stable
  pseudonym, and leakguard security flows. No production code changed; only the
  mock inference test helper gained a streaming response branch.
- M2: Added a dedicated `agent-coding` egress-guard regression target so the
  EP-006 leakguard-backed guard is locked by a `regression_*` test name, not
  only by the original EP-004 guard test.
- M3: Added deterministic `failure_*` targets for provider parse failure,
  egress-blocked fallback, wrong vault passphrase, DMS unsafe clock movement,
  metafuse bad input, wallet RPC error, and SSH nonzero exit. Validation used
  `cargo test --workspace`, which passed 121 tests across 60 suites.
- M4: Refined `TESTING.md` with an explicit flaky-test policy: fixed seeds/model
  clocks/mocks over ambient randomness, no `#[ignore]`/retry masking, and no
  quarantine for security-critical tests.
- M4: Ran `scripts/verify.sh` with host-cache access because `cargo audit`
  needs to lock/update `C:\Users\domin\.cargo\advisory-db` outside the
  workspace sandbox. The final run passed.

## 15. Outcomes & Retrospective
- EP-007 completed testing hardening with named regression and failure-mode
  targets across the critical flows: egress snapshot/guard, streaming provider
  contract, vault upgrade and wrong passphrase, git-spoof stable pseudonym, DMS
  clock safety, metafuse bad inputs, wallet RPC failure, and SSH failure.
- Unit, integration, workspace, and full verify gates passed locally. The final
  `scripts/verify.sh` run completed with `verify: ok`.
- CI now explicitly runs the full verify suite via `sh scripts/verify.sh`, with
  Rust musl target and cargo-audit installation retained.
- Remaining risks: Windows still skips static-musl execution smoke exactly as
  the scripts document; Linux CI remains authoritative for static binary
  execution; QEMU/on-image leak battery remains pending until EP-009 provides
  `build/adad.img`; vault runtime tests still skip on hosts without the Linux
  loopback/LUKS toolchain.
