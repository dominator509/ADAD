# Runbook — Killswitch Fired

- **Applies to:** leakguard, status monitor, network posture
- **Trigger:** Status monitor shows `Alert: KILLSWITCH FIRED - all egress dropped`
- **Risk level:** high
- **Reversible:** yes, only after the safe network posture is restored

## Preconditions
ADAD is booted, the operator can view the status monitor, and no attempt will be
made to bypass Tor/WireGuard or disable the killswitch.

## Procedure
1. Confirm the alert in the status monitor.
   - Expected: killswitch is `down` or unknown, and egress is blocked.
2. Check Tor and WireGuard status in the same monitor.
   - Expected: at least one tunnel or interface state explains the fail-closed
     transition.
3. Restore the underlying tunnel/interface outside the app if this is an
   operator-controlled lab environment.
   - Expected: health checks return to `ready`; fallback egress remains blocked
     until leakguard reports a safe snapshot.

## Verification
Run `scripts/test-e2e.sh` in the repository after any code or image change.
Expected: `e2e tests: ok`.

## Rollback
Do not roll back by weakening firewall rules. Revert the code or image change
that introduced the unsafe posture, then rerun `scripts/verify.sh`.

## Escalation
If the cause is unclear or the alert persists after restoring the tunnel,
STOP and require human review. Never disable the killswitch to continue work.
