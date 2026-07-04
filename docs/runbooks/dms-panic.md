# Runbook — DMS And Panic Wipe

- **Applies to:** leakguard DMS, panic wipe, vault safety
- **Trigger:** DMS near-expiry alert, panic action, or suspected unattended
  seizure risk
- **Risk level:** high
- **Reversible:** no for an executed header wipe

## Preconditions
The operator understands that an expired DMS wipes the LUKS header by design.
Automated sessions must use image targets only and must never trigger real
device wipes.

## Procedure
1. Confirm the status monitor alert.
   - Expected: `Alert: DMS NEAR EXPIRY - access vault or prepare wipe`.
2. If continued operation is intended, access the vault through the normal ADAD
   workflow before the DMS window expires.
   - Expected: DMS countdown refreshes from Tor-anchored time.
3. If panic wipe is required, invoke the panic action from the live system.
   - Expected: RAM is wiped and the session terminates immediately.

## Verification
For code changes, run `cargo test -p leakguard --test dms` and
`scripts/verify.sh`. Expected: DMS expiry, clock-freeze resistance, and panic
path tests pass.

## Rollback
An executed DMS header wipe is not reversible. Restore only from a deliberate
user-owned vault backup. Automated sessions must not restore or copy a real
production vault.

## Escalation
If DMS behavior appears weakened, STOP. Record a blocker and do not continue
with operations until the invariant is restored.
