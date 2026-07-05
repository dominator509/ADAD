---
id: EP-010
status: complete
depends_on: [EP-009]
verify: scripts/production-readiness-check.sh
---

# EP-010 — Production Readiness

## 1. Purpose / Big Picture
Bring ADAD to production readiness: full verification, security/privacy/
performance review, vault backup/restore + upgrade verification, a deployment
dry run, a rollback drill, documentation review, and the launch gate. Completing
this plan makes `scripts/production-readiness-check.sh` exit 0 and the loop print
`build: complete`.

## 2. Scope
- Run the full readiness gate and close every gap it surfaces.
- Perf review (inference tok/s band, killswitch latency, VPS < 2 min mock).
- Privacy review (tmpfs zero-write, metafuse, MAC, redacted logs).
- Backup/restore + upgrade verification.
- Deployment dry run (QEMU) + rollback drill (re-image to prior artifact).
- Docs/DECISIONS/ASSUMPTIONS reconciliation; launch checklist.

## 3. Non-goals
- No new features. No real-hardware device write (that final on-hardware smoke is
  a documented human step, not an automated one).

## 4. Context and Orientation
SPEC-008 + PRODUCTION_READINESS.md govern. This plan's `verify` is the readiness
check itself. Real device/remote/wallet operations remain STOPs.

## 5. Files to Read First
- SPEC-008, PRODUCTION_READINESS.md, RELEASE.md, ROLLBACK.md, all
  `.agent/checklists/*`, EP-006/EP-009 outcomes.

## 6. Files to Change
- `docs/` reviews/notes; `CHANGELOG.md` (create); any small fixes surfaced by
  the gate (narrow, in-scope). Reconcile DECISIONS.md / ASSUMPTIONS.md.

## 7. Interfaces and Contracts
- `scripts/production-readiness-check.sh` exits 0 (its assertions are the
  contract): verify passes, all plans complete, image exists, leak battery
  passed, ops docs present.

## 8. Milestones

### M1 — Full verification + gap closure
- Goal: `scripts/verify.sh` green; close any surfaced gaps narrowly.
- Validation: `scripts/verify.sh`
- Expected: `verify: ok`
- Recovery: fix the first failing gate; bounded retry; do not broaden scope.

### M2 — Security + privacy review
- Goal: confirm no-leak posture, redaction, secrets handling, tmpfs zero-write,
  metafuse, MAC on the built image.
- Validation: `scripts/security-check.sh && scripts/test-e2e.sh`
- Expected: `security check: ok`; leak battery passes on the image.
- Recovery: any leak/redaction finding is fixed before proceeding (may loop back
  to EP-006 via a blocker if structural).

### M3 — Performance review
- Goal: inference tok/s in band; killswitch latency in target; VPS < 2 min mock.
- Validation: a perf smoke (documented command) + the killswitch latency test.
- Expected: measured values within documented bands.
- Recovery: record any out-of-band result as a risk; fix if in-scope, else
  document explicitly.

### M4 — Backup/restore + upgrade + rollback drill
- Goal: vault backup/restore + upgrade verified; re-image rollback drilled.
- Validation: `cargo test -p forge --test vault_upgrade` + a documented rollback
  dry run in QEMU.
- Expected: upgrade + restore pass; rollback boots the prior artifact cleanly.
- Recovery: ensure non-destructive paths; a data-loss risk is a STOP.

### M5 — Docs reconciliation + launch gate
- Goal: reconcile docs/DECISIONS/ASSUMPTIONS with the built system; create
  CHANGELOG; run the readiness gate.
- Validation: `scripts/production-readiness-check.sh`
- Expected: `production readiness: ok`
- Recovery: close the specific assertion that fails; bounded retry.

## 9. Concrete Steps
1. Run full verify; close gaps.
2. Security + privacy review on the image; fix findings.
3. Perf review; record results.
4. Backup/restore + upgrade + rollback drill.
5. Reconcile docs; create CHANGELOG; run the readiness gate.

## 10. Validation and Acceptance
- [x] `scripts/verify.sh` → `verify: ok`
- [x] `scripts/security-check.sh` + on-image leak battery pass.
- [x] Perf values within documented bands (or risks documented).
- [x] Backup/restore + upgrade + rollback drill pass.
- [x] Docs/DECISIONS/ASSUMPTIONS reconciled; CHANGELOG created.
- [x] `scripts/production-readiness-check.sh` → `production readiness: ok`
- [x] `scripts/loop.sh` prints `build: complete` (all plans complete).

## 11. Idempotence and Recovery
Reviews and the gate are read-mostly; drills use images/QEMU. Re-runs are clean.
The final on-hardware smoke is a documented human step, never automated.

## 12. Progress
- [x] M1 — full verification + gap closure
- [x] M2 — security + privacy review
- [x] M3 — performance review
- [x] M4 — backup/restore + upgrade + rollback drill
- [x] M5 — docs reconciliation + launch gate
- [x] verify + status set to complete

## 13. Surprises & Discoveries
(Record any late gap, perf finding, or doc drift and how it was closed.)
- M1: `scripts/verify.sh` passed on the Windows/Git-Bash host using the
  containerized EP-009 QEMU/image path. The build step still reports that
  static musl execution is skipped on `MSYS_NT-10.0-19045`; that is an existing
  documented host boundary, with Linux CI/builder surfaces authoritative for
  static execution.
- M2: `scripts/security-check.sh` returned `security check: ok`, and
  `scripts/test-e2e.sh` returned `e2e tests: ok`. The QEMU leak battery passed
  against the built image and refreshed the image-local pass marker.
- M3: there is no existing allowed inference tok/s benchmark command, pinned
  GGUF fixture, or documented throughput band beyond the checklist requirement.
  Killswitch latency and mock VPS timing are covered by targeted tests; local
  inference throughput is recorded as a residual risk in
  `docs/EP-010-performance-review.md`.
- M4: the repo had boot-smoke support but no rollback-drill command. Added a
  non-destructive QEMU drill that copies `build/adad.img` inside the EP-009
  builder container as a temporary rollback artifact and boots that copy. It
  does not touch physical devices or persistent vaults.
- M5: `scripts/production-readiness-check.sh` asserts every ExecPlan front
  matter is `complete`, including EP-010. EP-010 therefore has to be marked
  complete before the final gate can evaluate the complete-plan invariant. If
  the gate fails, this session will revert or block on the specific failing
  readiness assertion.
- M5: `scripts/production-readiness-check.sh` returned
  `production readiness: ok`, and `scripts/loop.sh` returned `build: complete`
  after `next-step.sh` reached DONE.

## 14. Decision Log
(Record perf results, any accepted risks with rationale, launch decision.)
- M1: advanced EP-010 to `in-progress` after the repo next-step selector picked
  this plan and `scripts/preflight.sh .agent/execplans/EP-010-production-readiness.md`
  returned `preflight: ok`.
- M1: no code gap was surfaced by full verification; `scripts/verify.sh`
  returned `verify: ok`, including boot smoke, security/dependency audits, and
  the QEMU on-image leak battery.
- M2: accepted the existing automated privacy/security evidence for this
  milestone: RustSec audit, leakguard routing/DNS/IPv6/discovery tests, MAC
  randomization tests, DMS image-header wipe tests, agent egress guard tests,
  redaction tests from full verify, and the QEMU on-image leak battery.
- M3: used the existing targeted performance evidence instead of inventing a
  benchmark command: `cargo test -p leakguard --test netlink_drop` passed the
  250 ms DROP-ALL latency assertions, and
  `cargo test -p vps-deploy --test vps_mock` passed the mock provisioning
  under-120-second assertion.
- M4: added `tests/os/rollback-drill.sh` to `COMMANDS.md` before running it.
  `cargo test -p forge --test vault_upgrade` passed, proving backup creation
  and payload preservation during vault upgrade, and
  `tests/os/rollback-drill.sh` returned `rollback drill: ok`.
- M5: reconciled release/readiness docs by creating `CHANGELOG.md`, recording
  the EP-010 performance review and rollback drill notes, adding ADR-010 for
  the containerized image builder, and updating assumptions A1/A5/A6/A11 to
  match the verified build, perf, static-build, and no-real-device boundaries.
- M5: final automated launch gate passed. The loop proof used
  `AGENT_CMD=echo` via Git `env.exe`; because `next-step.sh` returned DONE, the
  dummy agent command was not invoked.

## 15. Outcomes & Retrospective
- Automated production readiness passed. `scripts/production-readiness-check.sh`
  returned `production readiness: ok`, and `scripts/loop.sh` printed
  `build: complete`.
- Final artifact: `build/adad.img`, SHA-256
  `2f1e3e5c0e3c5facf30eb5c8c296f718b362551583f567b72b4b8d10156c1904`.
- Verified evidence includes full `scripts/verify.sh`, security/dependency
  audit, QEMU boot smoke, QEMU on-image leak battery, vault upgrade backup and
  preservation, rollback drill, killswitch latency tests, and mock VPS timing.
- Residual risk: local inference tok/s is not measured because the repo has no
  allowed benchmark command, pinned GGUF fixture, or documented throughput band.
  This is documented in `docs/EP-010-performance-review.md`.
- Residual risk: final real-hardware boot smoke and any real device imaging,
  real VPS provisioning, real wallet movement, or live external Tor/network
  operations remain human-gated outside the automated loop.
