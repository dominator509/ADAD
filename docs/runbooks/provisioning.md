# Runbook — VPS Provisioning

- **Applies to:** vps-deploy mock/real provisioning boundary
- **Trigger:** preparing Forgejo hidden-service provisioning or debugging a VPS
  provision failure
- **Risk level:** high for real targets
- **Reversible:** no for real infrastructure changes without provider access

## Preconditions
Automated sessions may run mock provisioning tests only. Real VPS provisioning,
SSH keys, provider accounts, and XMR spend are STOP conditions unless a human
explicitly takes over that operation.

## Procedure
1. Validate the mock provisioning path.
   - Expected: `cargo test -p vps-deploy --test vps_mock` passes.
2. Confirm failures remain typed and non-partial.
   - Expected: `cargo test -p vps-deploy --test failure_ssh` passes.
3. For a human-run real provisioning event, confirm Tor, WireGuard, and wallet
   readiness in the status monitor before any SSH or payment action.
   - Expected: health states are `ready`; no killswitch or tunnel alert is
     present.

## Verification
Run `scripts/verify.sh` after changes. Expected: `verify: ok`.

## Rollback
Mock provisioning has no external state. Real infrastructure rollback is
provider-specific and must be performed manually by the operator.

## Escalation
If a real secret, VPS provider account, SSH key, wallet seed, or XMR spend is
required, STOP and require human control.
