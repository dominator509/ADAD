---
id: EP-010
status: not-started
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
- [ ] `scripts/verify.sh` → `verify: ok`
- [ ] `scripts/security-check.sh` + on-image leak battery pass.
- [ ] Perf values within documented bands (or risks documented).
- [ ] Backup/restore + upgrade + rollback drill pass.
- [ ] Docs/DECISIONS/ASSUMPTIONS reconciled; CHANGELOG created.
- [ ] `scripts/production-readiness-check.sh` → `production readiness: ok`
- [ ] `scripts/loop.sh` prints `build: complete` (all plans complete).

## 11. Idempotence and Recovery
Reviews and the gate are read-mostly; drills use images/QEMU. Re-runs are clean.
The final on-hardware smoke is a documented human step, never automated.

## 12. Progress
- [ ] M1 — full verification + gap closure
- [ ] M2 — security + privacy review
- [ ] M3 — performance review
- [ ] M4 — backup/restore + upgrade + rollback drill
- [ ] M5 — docs reconciliation + launch gate
- [ ] verify + status set to complete

## 13. Surprises & Discoveries
(Record any late gap, perf finding, or doc drift and how it was closed.)

## 14. Decision Log
(Record perf results, any accepted risks with rationale, launch decision.)

## 15. Outcomes & Retrospective
(Filled at completion — final residual-risk register and the launch statement.)
