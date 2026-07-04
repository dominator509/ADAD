# Runbook — Vault Backup And Restore

- **Applies to:** forge vault image, persona/config persistence
- **Trigger:** planned staging backup, restore drill, or vault upgrade rehearsal
- **Risk level:** medium
- **Reversible:** yes in test/staging; production backup is user-controlled

## Preconditions
Use an image file or test/staging vault only. A production vault backup is a
human decision and must not be automated by an agent session.

## Procedure
1. Run the upgrade and boundary tests before a vault-layout change.
   - Expected: `cargo test -p forge --test vault_upgrade` and
     `cargo test -p forge --test vault_boundary` pass.
2. Create a backup of the test/staging vault image before mutation.
   - Expected: a `.bak` image exists next to the source image.
3. Unlock the restored image through the normal vault workflow.
   - Expected: config and payload data are readable; secrets remain redacted in
     logs and errors.

## Verification
Run `scripts/verify.sh`. Expected: `verify: ok`.

## Rollback
For test/staging images, replace the mutated image with the `.bak` file and
rerun the vault tests. Production rollback requires explicit human sign-off.

## Escalation
If a restore cannot prove data integrity, STOP. Do not delete the backup image
or retry destructive operations blindly.
