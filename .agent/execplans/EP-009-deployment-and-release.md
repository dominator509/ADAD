---
id: EP-009
status: not-started
depends_on: [EP-008]
verify: scripts/verify.sh
---

# EP-009 — Deployment & Release (live-build Image)

## 1. Purpose / Big Picture
Assemble the bootable ADAD image from the static binaries using a `live-build`
recipe over a hardened Debian base, boot it in QEMU, and run the leak battery
against it — producing the release artifact and the `build/leak-battery.pass`
marker the readiness gate requires. This is where the OS substrate and the Rust
tools come together.

## 2. Scope
- `live-build/` recipe: Debian base, hardening hooks (tmpfs root, Tor-default,
  WireGuard, IPv6 off, MAC randomization, killswitch armed at boot), package
  lists, and inclusion of the static binaries.
- `tests/os/boot-smoke.sh` (QEMU boot) + `tests/e2e/run-leak-battery.sh`
  (on-image battery).
- Reproducible build producing `build/adad.img`.
- Release checklist + rollback path (docs exist; wire the artifacts).

## 3. Non-goals
- No writing to a real USB/device (human runbook only). No cloud staging. No new
  application features.

## 4. Context and Orientation
SPEC-008 + DEPLOYMENT.md + RELEASE.md + ROLLBACK.md govern. ADR-001 (Debian-Live
base). AGENTS.md §13: device imaging is human-only; automation uses image files
+ QEMU.

## 5. Files to Read First
- SPEC-008, DEPLOYMENT.md, RELEASE.md, ROLLBACK.md, ARCHITECTURE.md (image
  consumes binaries), EP-006 (posture the image must enforce at boot).

## 6. Files to Change
- `live-build/config/*` (recipe, package lists, hooks, includes)
- `live-build/hooks/*.hook.chroot` (harden: tmpfs, Tor, WG, IPv6 off, killswitch)
- `tests/os/boot-smoke.sh`, `tests/e2e/run-leak-battery.sh` + assertions
- a `scripts`-adjacent build entry the recipe invokes (documented in COMMANDS.md
  if a new command is needed — add it there first, with evidence)

## 7. Interfaces and Contracts
- The recipe embeds `target/x86_64-unknown-linux-musl/release/*` into the image.
- Boot arms the killswitch and applies MAC randomization + Tor-default before any
  network is usable.
- `build/adad.img` is the artifact; `build/leak-battery.pass` is written only on
  a passing on-image battery.

## 8. Milestones

### M1 — live-build recipe skeleton
- Goal: a recipe that builds a minimal bootable Debian-Live image.
- Files: `live-build/config/*`.
- Validation: the recipe builds `build/adad.img` (document the exact build
  command in COMMANDS.md first if new).
- Expected: `build/adad.img` produced.
- Recovery / **STOP**: if host tools (`lb`, `mksquashfs`, `qemu`) are missing,
  that is the install STOP — record a blocker, do not `apt install` in-session.

### M2 — Hardening hooks + binary inclusion
- Goal: tmpfs root, Tor-default, WireGuard, IPv6 off, MAC randomization,
  killswitch armed at boot; static binaries embedded.
- Files: `live-build/hooks/*`, includes.
- Validation: `tests/os/boot-smoke.sh` (QEMU boot to TUI; posture armed).
- Expected: boots to the TUI; killswitch armed; no IPv6; Tor bootstrapping.
- Recovery: fix the failing hook; a hook that could allow clearnet is a STOP.

### M3 — On-image leak battery
- Goal: run the leak battery against the booted image; write the pass marker.
- Files: `tests/e2e/run-leak-battery.sh` + assertions.
- Validation: `scripts/test-e2e.sh`
- Expected: `e2e tests: ok`; `build/leak-battery.pass` written; no clearnet/DNS/
  IPv6/discovery; killswitch fires on simulated drop.
- Recovery: trace the leaking class; fix the corresponding hook/rule; re-run.

### M4 — Reproducibility + release wiring + full verify
- Goal: reproducible build; release/rollback artifacts wired.
- Files: recipe pinning; release notes template usage.
- Validation: `scripts/verify.sh`
- Expected: `verify: ok` (e2e now runs since the harness+image exist).
- Recovery: first failing gate; bounded retry.

## 9. Concrete Steps
1. Author the live-build recipe; build a minimal image.
2. Add hardening hooks + embed the static binaries; boot-smoke in QEMU.
3. Author + run the on-image leak battery; write the pass marker.
4. Pin for reproducibility; wire release/rollback; run full verify.

## 10. Validation and Acceptance
- [ ] `build/adad.img` builds and boots in QEMU (boot-smoke passes).
- [ ] Boot posture armed: killswitch, Tor-default, IPv6 off, MAC randomized.
- [ ] On-image leak battery passes; `build/leak-battery.pass` present.
- [ ] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
The recipe rebuilds cleanly (clean build dir). QEMU runs are ephemeral. No real
device is written. A failed image build leaves no partial artifact claimed.

## 12. Progress
- [ ] M1 — live-build recipe skeleton
- [ ] M2 — hardening hooks + binary inclusion
- [ ] M3 — on-image leak battery
- [ ] M4 — reproducibility + release wiring + full verify
- [ ] verify + status set to complete

## 13. Surprises & Discoveries
(Record Debian suite pinned, hook ordering, QEMU NIC/disk monitoring approach.)

## 14. Decision Log
(Record base suite, package list rationale, build command added to COMMANDS.md.)

## 15. Outcomes & Retrospective
(Filled at completion — the image is the deliverable; document build determinism
and residual risks.)
