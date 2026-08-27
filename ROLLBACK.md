# ROLLBACK.md — ADAD

## Rollback triggers
- Leak battery regression discovered post-release (any leak class).
- Killswitch or DMS misbehavior.
- Vault corruption or failed upgrade path.
- Static-build/boot failure on target hardware.

## Rollback decision owner
The operator (single-user tool). For the blueprint loop, a rollback need is a
STOP condition requiring human action — never automated device writes.

## Rollback types
- **Image rollback:** re-image the drive with the previous release artifact.
- **Config rollback:** revert provider/networking config inside the vault.
- **Dependency rollback:** restore the prior workspace manifests and
  `Cargo.lock` through the active ExecPlan; the superseded claw-code vendoring
  design is not a rollback target.

## Application rollback
Re-image with the previous `adad.img`. Because the root is amnesic, there is no
in-place app state to unwind — the new boot uses the old image cleanly.

## Database rollback
Not applicable (no database).

## Vault (data) rollback
- Vault is separate from the image and versioned by `adad-core` config-version.
- Within the documented compatibility window, an older image reads a newer vault
  read-only or after a documented downgrade step.
- Always restore from a vault image backup rather than mutating in place. If no
  compatible path exists, that is a STOP — do not destroy the vault.

## Config rollback
Revert the vault config to the previous known-good values; re-run config
validation.

## Feature flag rollback
The main runtime flags are `ADAD_PROVIDER` and `ADAD_VENICE_ALLOW_ANONYMIZED`.
Roll back by resetting to defaults (`local`, `false`).

## Verification after rollback
- Boot smoke passes.
- Leak battery passes on the rolled-back image.
- Vault unlocks and reads correctly.

## Communication
Single-user: record the rollback in the local session notes and the next
release's CHANGELOG Security section if leak-related.

## Postmortem
Author a short postmortem (context, trigger, fix, prevention) and, if a boundary
or invariant was involved, an ADR. Add a regression test that would have caught
the issue.
