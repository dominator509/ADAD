# Runbook — DMS And Panic Wipe (Image-Only Boundary)

- **Applies to:** leakguard's image-only DMS adapter and vault safety
- **Trigger:** disposable-image DMS validation or a future live DMS integration
  risk
- **Risk level:** high
- **Reversible:** no for an executed header wipe

## Preconditions
The current source has no production DMS scheduler, live panic button, or
`kexec` backend. `leakguard dms evaluate-image` accepts only a regular,
disposable LUKS2 image and caller-supplied authoritative Tor-NTP values.
Automated sessions must use image targets only and must never trigger real
device wipes.

## Procedure
1. Do not treat the status monitor's `DMS: unknown` value as an active timer;
   the current status probe has no live DMS source.
2. For disposable-image validation, run the documented
   `leakguard dms evaluate-image` command with a regular LUKS2 image and
   authoritative Tor-NTP timestamps.
   - Expected: an unexpired image reports `dms=Armed`; an expired image reports
     `dms=Expired header_wiped=true image_only=true`.
3. If a live panic wipe or production-device DMS is required, STOP and require
   a separately reviewed production backend. Do not substitute the image
   command or claim that RAM/device destruction occurred.

## Verification
For code changes, run `cargo test -p leakguard --test dms` and
`scripts/verify.sh`. These tests prove the model and disposable-image adapter;
they do not prove a live scheduler, production-device destruction, or panic
`kexec` behavior.

## Rollback
An executed DMS header wipe is not reversible. Restore only from a deliberate
user-owned vault backup. Automated sessions must not restore or copy a real
production vault.

## Escalation
If DMS behavior appears weakened, STOP. Record a blocker and do not continue
with operations until the invariant is restored.
