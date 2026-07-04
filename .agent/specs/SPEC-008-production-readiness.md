# SPEC-008 — Production Readiness

- **Status:** active
- **Owner:** architect
- **Roadmap phase:** Phases 8–9
- **Linked ExecPlans:** EP-009, EP-010

## User-visible goal
A releasable, bootable ADAD image exists that passes the full verification and
leak battery on real hardware, with all operational docs, rollback, and backups
in place.

## Non-goals
No cloud staging; no fleet telemetry; no autonomous production device writes.

## Terms
- **Readiness gate:** `scripts/production-readiness-check.sh` exiting 0.

## Required behavior
- The `live-build` recipe MUST assemble a bootable `build/adad.img` from the
  static binaries, reproducibly.
- The image MUST boot in QEMU and pass `scripts/test-e2e.sh` (leak battery),
  producing `build/leak-battery.pass`.
- Every ExecPlan MUST be `status: complete`.
- The readiness gate MUST assert: verify passes, all plans complete, image
  exists, leak battery passed, and required operational docs present.
- A rollback drill and a vault backup/restore + upgrade test MUST pass.
- A manual on-hardware boot smoke MUST confirm the leak battery on real hardware
  before launch (human step).

## Inputs
The verified workspace; the built image; QEMU; the readiness scripts.

## Outputs
A release artifact + checksum; a passing readiness gate; a launch decision.

## Error states
Any failing gate blocks launch; a leak regression is a rollback trigger.

## Data rules
Vault backup/restore verified at image level; upgrade path tested.

## Security rules
Full leak battery + security-check clean; no secrets in the artifact; Venice
private-by-default confirmed.

## Accessibility rules
Keyboard-only + high-contrast confirmed on the booted image.

## Performance rules
Inference tok/s in band; killswitch latency in target; VPS provisioning < 2 min
(mock) — all re-confirmed on the built image where feasible.

## Observability rules
Logs RAM-only + redacted confirmed on the booted image.

## Required tests
- Image boot smoke (QEMU).
- Leak battery against the image (writes the pass marker).
- Rollback drill (re-image to previous artifact).
- Vault backup/restore + upgrade.
- Readiness gate script exits 0.

## Acceptance criteria
- [ ] `build/adad.img` builds and boots in QEMU.
- [ ] Leak battery passes against the image (`build/leak-battery.pass` present).
- [ ] Rollback drill and vault backup/restore/upgrade pass.
- [ ] `scripts/production-readiness-check.sh` exits 0.
- [ ] `scripts/loop.sh` prints `build: complete`.
