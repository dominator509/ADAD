# DEPLOYMENT.md — ADAD

## Deployment environments
- **Test:** QEMU VM, mocked providers, loopback vault image.
- **Staging:** QEMU VM booting the real release image, monitored NIC/disk, still
  no real remotes/wallet.
- **Production:** physical USB/NVMe-in-USB drive booted on x86_64 hardware.

## Deployment architecture
The release artifact is a `live-build`-produced hybrid ISO/IMG containing a
SquashFS root, the static core binaries, hardened networking config
(Tor-by-default, WireGuard split-tunnel, IPv6 off, killswitch), and the amnesic
tmpfs overlay. Persistent state is a separate LUKS2 vault the user creates on
first run; it is never baked into the image.

## Build artifact
- `build/adad.img` — the bootable image (gitignored; produced by EP-009's
  `live-build` recipe).
- `target/x86_64-unknown-linux-musl/release/*` — the static tools embedded into
  the image.

## Release flow
1. `scripts/verify.sh` passes on the workspace.
2. EP-009 recipe builds `build/adad.img` reproducibly.
3. Boot the image in QEMU; run `scripts/test-e2e.sh` (leak battery) against it;
   it writes `build/leak-battery.pass`.
4. `scripts/production-readiness-check.sh` passes.
5. Tag a release (RELEASE.md).

## Deployment steps (production imaging — human runbook)
> This writes to a real device and is therefore a **human-only** operation. No
> agent session performs it (AGENTS.md §13).
1. Verify the artifact checksum against the release notes.
2. Identify the target device carefully (`lsblk`); double-check it is the USB,
   not an internal disk.
3. Write the image (`dd`/`cp` to the device) — human-confirmed target.
4. Boot on the target host; create the LUKS2 vault on first run.
5. Configure providers/keys inside the running system (stored in the vault).

## Migration steps
No DB migrations. Vault-layout upgrades: boot the new image, run the
`forge-rs`/`persona-rs` in-place upgrade (bumps config version), verified by the
vault-upgrade test. Keep a vault image backup first.

## Rollback steps
See ROLLBACK.md. In short: re-image the drive with the previous release
artifact; the vault is forward/backward-compatible within a documented version
window.

## Post-deploy smoke tests
- Boot completes to the TUI.
- Status monitors show Tor bootstrapped, WireGuard down-by-default until
  configured, killswitch armed.
- Leak battery passes against the running system.

## Required approvals
- Writing to a physical device: explicit human confirmation of the target.
- Any real VPS provisioning or XMR spend: explicit human action, never
  automated.

## Deployment STOP conditions
- Any automated request to write a real device, provision a real VPS, or spend
  real XMR. Record a blocker; require human sign-off.

## Production verification
`scripts/production-readiness-check.sh` exits 0 AND a manual boot smoke on real
hardware confirms the leak battery passes on-device.
