# ADAD Production-Readiness Audit — NOT_RUNNABLE_ENV annotations

Status: ANNOTATION ONLY. The four functional areas below are recorded as
NOT_RUNNABLE_ENV for this audit run, with the exact reasons from the original
task (operator spec 2026-09-07, kanban t_3345b9d4). NONE of the commands that
would evidence these areas were executed; no evidence log exists for them and
none is claimed. This follows the matrix meta-rules: no mocked success
masquerading as production, no claim of capability without an executable test
against the real target.

- Author: scribe (kanban task t_199e67ef), 2026-09-07
- Cross-reference: category map in `matrix-map.md` (rows flagged `NR-OS`,
  `NR-LUKS`, `NR-VPS`, `NR-XMR`).
- Producer convention: sprint (t_bf7d4ee4) and forge (t_bd3bc731) do NOT run
  these areas; any command they record as blocked carries a `.reason` file.
  The final audit (`PRODUCTION_READINESS_AUDIT.md`) marks these four areas
  NOT_RUNNABLE_ENV and nothing else.

Environment facts verified on this host at annotation time (2026-09-07):

- No physical block device besides the system disk; `/dev/mapper` contains only
  the device-mapper `control` node (no mapped volumes). The only other block
  nodes are read-only snap loop mounts and ephemeral loop fixtures created by
  in-repo tests at test time (ENVIRONMENT.md: "Loopback LUKS images + QEMU +
  mock servers, all created at test time. No real devices or remotes.").
- No production image artifact exists: `/root/ADAD/build/adad.img` and image
  provenance are absent (repo `.gitignore` excludes `/build` and `*.img`).
- Host tools required by the image track are not installed: `lb`
  (live-build), `debootstrap`, `qemu-system-x86_64` all absent
  (ENVIRONMENT.md host-tool list: cryptsetup, util-linux, e2fsprogs,
  coreutils, live-build, squashfs-tools, qemu-system-x86).
- Network egress changes are not permitted on this host (hardened host;
  project-boundary rule: no VPS/tunnel/egress changes without permission), and
  no real VPS, WireGuard peer, Monero daemon, wallet RPC endpoint, or Monero
  network access is available.
- Repo's own release posture agrees: EP-013 non-goals — "No real wallet
  transfer, VPS provisioning, WireGuard interface activation, FUSE mount, DMS
  destruction, panic/kexec, hardware boot, or production deployment."

---

## 1. OS-image — NOT_RUNNABLE_ENV

Status: NOT_RUNNABLE_ENV

Exact reason: requires a real Debian-Live/block-device image target not
available.

What this area covers (would-be checks, NOT run):

- `scripts/build-image.sh` / `build-image-inside.sh` / `build-image-builder.sh`
  / `check-image-builder.sh` producing a bootable `build/adad.img` from the
  `live-build/` recipe (immutably pinned inputs, clean isolated build,
  digest provenance).
- Booted-image verification: QEMU boot smoke (`tests/os/boot-smoke.sh`), leak
  battery against the booted image (`tests/e2e/run-leak-battery.sh`), rollback
  drill (`tests/os/rollback-drill*.sh`), and the release gates that consume
  them (`scripts/production-readiness-check.sh`, `scripts/test-e2e.sh`,
  `scripts/min-system-sim.sh`, `scripts/test-integration.sh` with
  `ADAD_REQUIRE_IMAGE=1`).

Environment grounding: no image artifact exists (`build/adad.img` absent) and
the image toolchain is absent (`lb`, `debootstrap`, `qemu-system-x86_64` not
installed). Producing and booting the real Debian-Live image target is not
possible in this environment. Nothing in this area was executed and no
evidence log exists for it.

## 2. LUKS — NOT_RUNNABLE_ENV

Status: NOT_RUNNABLE_ENV

Exact reason: requires a real block device to encrypt/decrypt (none
available).

What this area covers (would-be checks, NOT run):

- LUKS2 vault lifecycle on a real block-device target — create/unlock/lock/seal
  (`crates/forge/src/vault.rs`, vault passphrase to `cryptsetup` over a closed
  stdin pipe, Argon2id unlock), Dead Man's Switch and panic wipe
  (`crates/leakguard/src/dms.rs`), and vault backup/restore/upgrade at the
  device level (PRODUCTION_READINESS.md LUKS2 vault readiness bullets).

Environment grounding: no real (physical) block device is available to
encrypt/decrypt; `/dev/mapper` exposes only the `control` node. In-repo tests
that touch LUKS-vault logic use ephemeral loopback images created at test time
per ENVIRONMENT.md and are matrix rows 3/4/8/26 evidence logged by the
baseline/final cargo-test runs — they are not the real block-device target
this area requires, and per the matrix meta-rules they do not stand in for it.
Nothing in this area was executed against a real block device and no evidence
log exists for it.

## 3. VPS — NOT_RUNNABLE_ENV

Status: NOT_RUNNABLE_ENV

Exact reason: requires a real VPS/network egress changes not permitted.

What this area covers (would-be checks, NOT run):

- Real VPS provisioning via `crates/vps-deploy` (XMR-paid VPS provisioning,
  automated setup, `< 2 min` provisioning target), WireGuard split-tunnel
  egress activation, and any deployment that changes host network egress.

Environment grounding: no real VPS is available and network egress changes are
not permitted on this host (hardened host, project-boundary rule). The crate's
service surface is contract-tested with mocks inside the cargo suite (matrix
rows 3/4/5 evidence); that is not real provisioning or egress activity.
Nothing in this area was executed and no evidence log exists for it.

## 4. XMR — NOT_RUNNABLE_ENV

Status: NOT_RUNNABLE_ENV

Exact reason: requires live Monero network/wallet/egress activity not
permitted.

What this area covers (would-be checks, NOT run):

- Live Monero wallet RPC and transfer activity via `crates/xmr-wallet`
  (production wallet RPC and SSH transports behind human confirmation,
  PROJECT_BRIEF XMR-paid VPS funding flow).

Environment grounding: no Monero daemon, wallet RPC endpoint, or Monero
network access exists, and live wallet/egress activity is not permitted in
this environment (EP-013: "No real wallet transfer ... No real device, vault,
remote host, wallet, or secret is touched"). In-repo wallet tests are
mock/contract tests inside the cargo suite (matrix rows 3/4/5 evidence); they
are not live Monero activity. Nothing in this area was executed and no
evidence log exists for it.

---

## Compliance statement

- These four areas are annotated NOT_RUNNABLE_ENV only; no PASS/FAIL is
  assigned to them and no runnable category verdict is assigned anywhere in
  this skeleton.
- No command in these areas was executed, and no unrun command is claimed as
  run. Any `.reason` files the sprint/forge tasks record for blocked commands
  are the authoritative per-command blockers.
- Reasons above match the audit environment (verified device/tool/egress
  facts) and the original task wording.
