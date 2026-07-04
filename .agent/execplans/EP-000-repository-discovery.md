---
id: EP-000
status: complete
depends_on: []
verify: scripts/verify.sh
---

# EP-000 — Repository Discovery

## 1. Purpose / Big Picture
Discover repository structure, stack, commands, current state, and risks before
implementation. For a GREENFIELD repository there is nothing to discover: the
tree is the blueprint pack itself. This plan is therefore pre-completed, and its
job is to record that decision and confirm the scaffolding assumptions the rest
of the build depends on.

## 2. Scope
- Confirm the repository is greenfield (no prior source).
- Record the intended stack and command surface (already captured in COMMANDS.md,
  ARCHITECTURE.md, ASSUMPTIONS.md).

## 3. Non-goals
- No code. No scaffolding (that is EP-001). No vendoring (that is EP-002).

## 4. Context and Orientation
This is a greenfield ADAD build. The `.agent/` control plane, scripts, and root
docs already exist (they ARE the pack). There is no legacy code, CI, or
environment to reverse-engineer.

## 5. Files to Read First
- PROJECT_BRIEF.md, ARCHITECTURE.md, ROADMAP.md, ASSUMPTIONS.md, COMMANDS.md.

## 6. Files to Change
- This file only (front matter already `complete`; Decision Log entry below).

## 7. Interfaces and Contracts
None.

## 8. Milestones
- **M1 — Confirm greenfield & record.** (Pre-done.)
  - Validation: `test -z "$(ls crates 2>/dev/null)"` (no source yet).
  - Expected: command succeeds (no `crates/` source present).
  - Recovery: if source unexpectedly exists, set status back to `in-progress`
    and perform a real discovery pass (inventory `Cargo.toml`, tests, CI).

## 9. Concrete Steps
Already performed during blueprint generation. Nothing to execute.

## 10. Validation and Acceptance
- [x] Repository confirmed greenfield.
- [x] Stack + commands recorded in COMMANDS.md / ARCHITECTURE.md / ASSUMPTIONS.md.

## 11. Idempotence and Recovery
Re-running is a no-op. If a real (non-greenfield) tree appears, flip status to
`in-progress` and do a genuine discovery pass updating COMMANDS/ARCHITECTURE/
ASSUMPTIONS.

## 12. Progress
- [x] M1 — greenfield confirmed and recorded
- [x] verify + status set to complete

## 13. Surprises & Discoveries
None. Greenfield as declared in the project inputs.

## 14. Decision Log
- 2026-07-03: Greenfield → EP-000 set `status: complete` at generation time per
  the pack's greenfield rule. The discovery outputs (stack, commands, risks)
  were authored directly into COMMANDS.md, ARCHITECTURE.md, and ASSUMPTIONS.md.

## 15. Outcomes & Retrospective
Discovery is a no-op for greenfield. The loop starts real work at EP-001.
Remaining risk: the claw-code vendoring assumption (A2) is unverified until
EP-002 — deliberately deferred there with a STOP guard.
