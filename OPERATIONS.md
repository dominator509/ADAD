# OPERATIONS.md — ADAD Runbook

## Local operations
- Build: `scripts/build.sh`. Verify: `scripts/verify.sh`.
- Run a tool: `target/x86_64-unknown-linux-musl/release/<tool> --help`.
- Loop build: `AGENT_CMD=... scripts/loop.sh`; status: `scripts/loop-status.sh`.

## Staging (QEMU) operations
- Boot the image: `qemu-system-x86_64` with a monitored NIC/disk (see
  `tests/os/boot-smoke.sh`).
- Run leak battery: `scripts/test-e2e.sh`.

## Production operations
- The current repository is not release-cleared for production operation; use
  the workflow only after every gate in `PRODUCTION_READINESS.md` is closed.
- The current source has no production panic-button or `kexec` backend. Do not
  rely on a panic wipe for a live session; the image-only DMS adapter and its
  disposable-file tests are not a substitute.
- The current DMS command accepts caller-supplied authoritative Tor-NTP time
  for a regular image file only. Automatic scheduling and production-device
  destruction remain unimplemented and release-gated.
- Normal shutdown is intended to scrub key material from memory; zero-host-
  write and live shutdown evidence remain release gates.

## Health checks
- Tor bootstrapped (control-port query).
- WireGuard interface state (up only when configured; killswitch armed).
- llama-server reachable on loopback (when local provider active).
- monero-wallet-rpc reachable over Tor (when wallet in use).
- Git daemon / Forgejo hidden service status (when publishing).

## Common failure modes
| Symptom | Likely cause | First action |
|---|---|---|
| All egress dropped unexpectedly | killswitch fired on interface change | check NIC state; this is fail-closed by design |
| Local inference fails | llama-server not running / wrong base URL | check loopback endpoint; confirm GGUF loaded |
| API fallback fails | WireGuard down / key missing | verify tunnel; keys load from vault |
| Wallet ops time out | Tor remote node slow/unreachable | rotate remote node; check Tor bootstrap |
| Vault won't unlock | wrong passphrase / DMS wiped header | if DMS wiped, header is gone by design |

## Troubleshooting
Follow the failure table, then the relevant runbook (use
`.agent/templates/runbook-template.md` to author new ones). Never disable the
killswitch or DMS to "fix" a symptom.

## Vault backup/restore
- Backup (test/staging only): copy the LUKS2 vault image to encrypted external
  media. Restore: place the image at `ADAD_VAULT_PATH` and unlock.
- Production vault backup is a user decision; document but never automate copying
  a real vault off-box without user action.

## Scheduled jobs
- No production Dead Man's Switch scheduler is currently shipped. The
  `leakguard dms evaluate-image` command is an image-only validation adapter;
  automatic Tor-NTP acquisition and destructive scheduling remain
  release-gated. Never treat its output as evidence that a live device is
  protected.

## Incident triage
See `.agent/checklists/incident-response.md`. Detect → triage → mitigate →
communicate → resolve → verify → document → follow up.

## Escalation rules
Single-operator tool: "escalation" = STOP and require human decision for
anything touching real devices, remotes, wallets, or the killswitch/DMS.

## Maintenance windows
N/A (no hosted service). Image updates are re-imaging events chosen by the user.

## Operational safety rules
- Never operate on a real device/remote/wallet from an automated context.
- Never weaken leak-prevention or self-destruct to ease operations.
- Every destructive operation has a runbook with an explicit reversibility note.
