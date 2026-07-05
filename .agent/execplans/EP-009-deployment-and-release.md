---
id: EP-009
status: complete
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
- [x] `build/adad.img` builds and boots in QEMU (boot-smoke passes).
- [x] Boot posture armed: killswitch, Tor-default, IPv6 off, MAC randomized.
- [x] On-image leak battery passes; `build/leak-battery.pass` present.
- [x] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
The recipe rebuilds cleanly (clean build dir). QEMU runs are ephemeral. No real
device is written. A failed image build leaves no partial artifact claimed.

## 12. Progress
- [x] M1 — live-build recipe skeleton
- [x] M2 — hardening hooks + binary inclusion
- [x] M3 — on-image leak battery
- [x] M4 — reproducibility + release wiring + full verify
- [x] verify + status set to complete

## 13. Surprises & Discoveries
(Record Debian suite pinned, hook ordering, QEMU NIC/disk monitoring approach.)
- M1 recovery: the Windows/Git-Bash host is missing the required image-build
  tools (`qemu-system-x86_64`, `mksquashfs`, `lb`, `cryptsetup`, `losetup`,
  `mkfs.ext4`). `scripts/install.sh` reported the exact missing set and, per
  `COMMANDS.md`, agents must not install them in-session.
- M1 recovery: WSL2 is present according to `wsl.exe --list --verbose`, but this
  session could not execute the default Ubuntu distro from the sandboxed shell:
  explicit `-d Ubuntu` returned `WSL_E_DISTRO_NOT_FOUND`, while default
  execution reported no installed distributions.
- M1 recovery: Docker Desktop is available with local `debian:trixie` and
  `ubuntu:24.04` images, but neither image contains `/usr/bin/lb`,
  `/usr/bin/mksquashfs`, or `/usr/bin/qemu-system-x86_64`. Installing those
  packages inside a container is still a package-install path, so this session
  did not proceed without a resolved builder environment.
- M1 recovery: the user explicitly resolved BLK-004 by approving required
  tool/software installation and execution in whichever safe environment is
  necessary. The project now has a first-class containerized EP-009 builder path
  that installs `live-build`, `squashfs-tools`, `qemu-system-x86`, and the other
  `scripts/install.sh` image tools inside `adad-ep009-builder:local`, not on the
  Windows host and not against any physical device.
- M1 recovery: `scripts/build-image-builder.sh` produced
  `adad-ep009-builder:local`, and `scripts/check-image-builder.sh` returned
  `image builder check: ok`. Direct tool probes inside that image returned
  live-build `20250505+deb13u1`, mksquashfs `4.6.1`, and QEMU `10.0.8`.
- M1 build attempt 1: `scripts/build-image.sh` completed the debootstrap
  bootstrap stage, then failed in `lb chroot_devpts install` and
  `lb chroot_proc install` because the non-privileged Docker run could not mount
  `/dev/pts` or `/proc` inside the live-build chroot.
- M1: `scripts/build-image.sh` then completed successfully after the targeted
  container mount-capability fix. It produced `build/adad.img` (333 MiB) with
  SHA-256 `54f617810ad98e3b583f7ad8d801e21355020360be7f417aa2b90e087c7be64b`.
  The live-build output artifact was `adad-amd64.hybrid.iso`, copied to the
  repo's required `build/adad.img` path.
- M2 boot-smoke attempt 1: QEMU timed out without hardening markers because the
  generated BIOS bootloader config used `default vesamenu.c32`, `prompt 0`, and
  `timeout 0`, which leaves headless QEMU waiting at a VGA boot menu. The
  generated live and GRUB entries did include the serial kernel console args.
- M2: adding `live-build/config/includes.binary/isolinux/isolinux.cfg` made the
  ISO default directly to `live-amd64`, preserving the generated `live.cfg`.
  `tests/os/boot-smoke.sh` then returned `boot smoke: ok`.
- M3: `scripts/test-e2e.sh` now runs the existing EP-006 model battery and a
  QEMU boot of the built image with `adad.leak_battery=1`. The on-image service
  reported static tools present, IPv6 disabled, killswitch rules armed, DNS and
  discovery blocks present, Tor-default config/service present, locally
  administered MACs, direct clearnet blocked, and drop simulation still
  fail-closed. The run wrote `build/leak-battery.pass`.
- M4: the final image build pinned `SOURCE_DATE_EPOCH=946684800`, UTC, and
  stable ISO publisher/preparer/application/volume metadata. The final artifact
  is `build/adad.img` (337 MiB) with SHA-256
  `2f1e3e5c0e3c5facf30eb5c8c296f718b362551583f567b72b4b8d10156c1904`.
- M4: `scripts/verify.sh` completed successfully after the final reproducibility
  pinning. The command reported `verify: ok` and refreshed
  `build/leak-battery.pass` after the QEMU on-image leak battery passed.

## 14. Decision Log
(Record base suite, package list rationale, build command added to COMMANDS.md.)
- M1: Blocked before authoring the live-build recipe because the required
  validation command cannot run on any currently usable builder surface. Per the
  plan's recovery rule, missing `lb`/`mksquashfs`/`qemu` is an install STOP, not
  a reason to add an unvalidated recipe or in-session package installation.
- M1: Resolved the builder-environment decision by adding an explicit
  containerized builder setup command to `COMMANDS.md`. The builder is based on
  the local Rust/Debian image lineage (`rust:1.90`) so it can satisfy both the
  Rust musl target and EP-009 image-build tools without host package mutation.
- M1: Kept M1 unchecked after resolving the builder because the recipe has not
  yet produced `build/adad.img`; the next project step is the live-build recipe
  skeleton, now using the verified containerized builder.
- M1: Updated `scripts/build-image.sh` to grant the builder container
  `SYS_ADMIN` plus unconfined seccomp/AppArmor for live-build chroot mounts.
  The runner still mounts only the repository workspace and binds no host block
  devices, preserving the image-file-only rule.
- M1: Pinned the initial image skeleton to Debian `trixie`/`amd64`,
  `iso-hybrid`, `archive-areas main`, no Debian installer, SHA-256 checksums,
  and a minimal package list (`ca-certificates`, `iproute2`, `live-boot`,
  `systemd-sysv`). M2 owns hardening hooks and binary inclusion.
- M2: embedded the static musl tool binaries from
  `target/x86_64-unknown-linux-musl/release` into `/usr/local/bin` during the
  image build, failing the build if any required tool is missing.
- M2: implemented the first image hardening hook as a systemd one-shot before
  networking. It disables IPv6, installs an nftables default-drop posture,
  randomizes non-loopback MAC addresses with locally administered unicast
  addresses, enables Tor, and writes serial-console markers for QEMU smoke
  evidence.
- M2: allowed DHCP client traffic in the nftables output chain so the live
  system can acquire network configuration for Tor while still dropping generic
  clearnet, direct DNS, mDNS, SSDP, and NetBIOS/SMB discovery classes.
- M3: made the on-image leak battery opt-in via the kernel command-line flag
  `adad.leak_battery=1`. Normal boots do not run the test service; the QEMU
  runner extracts the live kernel/initrd from `build/adad.img`, boots with that
  flag, watches serial markers, and writes `build/leak-battery.pass` only after
  every marker is observed.
- M3: Tor-default is asserted as image-local evidence in this milestone: Tor
  service is enabled, Tor DNS/private IPv6 config is present, and only the
  `debian-tor` user plus WireGuard/DHCP/loopback are allowed by the active
  firewall. A live external Tor circuit bootstrap remains an operator/network
  dependent smoke item for release validation, not a claimed internet check in
  the hermetic QEMU battery.
- M4: pinned reproducibility inputs in the image builder with a deterministic
  default source date, UTC timezone, and stable ISO metadata. The release
  artifact path remains `build/adad.img`; rollback remains the documented
  human-run image replacement path and does not write to a physical device from
  automation.

## 15. Outcomes & Retrospective
- EP-009 produced the bootable release artifact at `build/adad.img` and the
  on-image evidence marker at `build/leak-battery.pass`.
- Final artifact evidence: `build/adad.img` is 337 MiB with SHA-256
  `2f1e3e5c0e3c5facf30eb5c8c296f718b362551583f567b72b4b8d10156c1904`.
- `scripts/verify.sh` passed end-to-end, including boot smoke, security and
  dependency audits, the EP-006 leakguard model battery, and the QEMU on-image
  leak battery.
- Remaining risk: the hermetic QEMU battery proves image-local Tor-default
  configuration, firewall ownership, and direct clearnet blocking; it does not
  prove a live external Tor circuit bootstrap on a real network. That remains a
  release/operator smoke item outside automated physical-device writes.
