# Blockers

## BLK-001 (EP-002, M4) — RESOLVED
Blocker: ADR-002 assumes a standalone claw-code MCP crate plus a standalone tool-execution crate can be vendored in isolation, but upstream `ultraworkers/claw-code` at commit `4ea31c1bc91c4e9bcbd67d51c550c01e127e6d0d` no longer has that shape.
Evidence: `build/vendor-src/claw-code/rust/crates/runtime/src/lib.rs:3-4` says `runtime` owns session persistence, MCP plumbing, tool-facing file operations, and the core conversation loop; `build/vendor-src/claw-code/rust/crates/tools/Cargo.toml:9-13` depends on `api`, `commands`, `plugins`, and `runtime`; `build/vendor-src/claw-code/rust/crates/api/Cargo.toml:10,13` and `build/vendor-src/claw-code/rust/crates/commands/Cargo.toml:12-13` pull `runtime`/`telemetry`; crate inventory under `build/vendor-src/claw-code/rust/crates/*/Cargo.toml` shows no package named `mcp`.
Smallest decision needed: Decide whether ADAD may vendor a larger frozen claw-code subset and accept that runtime surface, or whether ADR-002/EP-002 should switch to a different MCP/tool-execution source.
Recommended default: Revise ADR-002/EP-002 before further implementation; do not vendor the current claw-code runtime stack under the existing two-crate isolation requirement.
Resolution: User chose the architecture pivot on 2026-07-03: replace the failed claw-code vendoring seam with the official MCP Rust SDK plus ADAD-owned execution logic, while targeting Claude-Code-like features and feel through first-party `agent-coding` work.

## BLK-002 (EP-003, M2) — RESOLVED
Blocker: EP-003's vault lifecycle tests require Linux loopback/LUKS host tools, but this host does not provide `cryptsetup` or `losetup`, so M2's required runtime validation cannot run here.
Evidence: `cryptsetup --help` -> `/usr/bin/bash: line 1: cryptsetup: command not found`; `losetup --help` -> `/usr/bin/bash: line 1: losetup: command not found`; `ENVIRONMENT.md:14-18` says the build host is Debian/Ubuntu x86_64 and missing host tools are a STOP rather than an in-session install.
Smallest decision needed: Provide a Linux/Debian build host with `cryptsetup` and `losetup` available, or otherwise supply an approved environment where the loopback LUKS tests can run.
Recommended default: Resume EP-003 on a Debian/Ubuntu x86_64 builder with the required host tools preinstalled, keeping the new M1 harness as the starting point for M2.
Resolution: User approved the host-admin exception on 2026-07-03. Ubuntu 24.04.4 LTS on WSL2 was used as the Linux builder, `cryptsetup` was installed there, `/dev/loop-control` and `/dev/loop0` were present, and the real M2-M5 validations passed from a WSL-local ADAD working copy.

## BLK-003 (EP-003, M6) — RESOLVED
Blocker: `scripts/verify.sh` now reaches `scripts/build.sh` on the WSL Linux builder, but the Rust `x86_64-unknown-linux-musl` target is missing and repeated rustup recovery attempts failed for environment reasons.
Evidence: WSL `scripts/verify.sh` failed in `scripts/build.sh` with `error[E0463]: can't find crate for core` and `can't find crate for std` plus `the x86_64-unknown-linux-musl target may not be installed`; `rustup target add x86_64-unknown-linux-musl` first failed during a stable-toolchain update/rollback conflict (`Directory not empty`) and then `rustup target add --toolchain 1.96.0-x86_64-unknown-linux-gnu x86_64-unknown-linux-musl` failed with `error decoding response body: cannot decrypt peer's message` while downloading `rust-std-1.96.0-x86_64-unknown-linux-musl.tar.xz` from `static.rust-lang.org`.
Smallest decision needed: Repair the WSL Rust toolchain/network path enough to install the musl target, or provide another Linux builder where the musl target is already available.
Recommended default: Keep the WSL runtime environment for EP-003 tests, but fix or replace the Rust toolchain environment before retrying M6.
Resolution: Resolved in-session on 2026-07-03. The Rust 1.96.0 musl std archive was downloaded with resumable curl after Rustup/TLS failures, verified against the published SHA-256 checksum, and installed into `/home/doministic/.rustup/toolchains/1.96.0-x86_64-unknown-linux-gnu`. A root-owned RustSec advisory cache under `/root/.cargo/advisory-db` was seeded to avoid Git safe-directory and flaky full-clone failures. Final WSL `scripts/build.sh` returned `build: ok`, and final WSL `scripts/verify.sh` returned `verify: ok`.

## BLK-004 (EP-009, M1) — RESOLVED
Blocker: EP-009 M1 requires a Debian/Ubuntu image-build environment with `lb`, `mksquashfs`, and `qemu-system-x86_64`, but no currently usable host surface provides those tools.
Evidence: `scripts/install.sh` on the Windows/Git-Bash host returned `install: host tools missing: qemu-system-x86_64 mksquashfs lb cryptsetup losetup mkfs.ext4`; `wsl.exe --list --verbose` showed an Ubuntu WSL2 distro, but `wsl.exe -d Ubuntu --exec bash -lc "cd /mnt/c/dev/ADAD && scripts/install.sh"` returned `WSL_E_DISTRO_NOT_FOUND` and default `wsl.exe --exec ...` reported no installed distributions; `docker run --rm debian:trixie /bin/ls /usr/bin/lb /usr/bin/mksquashfs /usr/bin/qemu-system-x86_64` and the same check against `ubuntu:24.04` reported all three paths missing.
Smallest decision needed: Provide or approve a usable Linux builder that already has `live-build`, `squashfs-tools`, and `qemu-system-x86` installed, or explicitly revise COMMANDS.md to allow a containerized EP-009 builder setup path.
Recommended default: Repair/use the existing Ubuntu WSL2 builder or provide a prebuilt Docker builder image with the required packages installed, then rerun EP-009 from the repository root; keep physical device imaging and production release actions human-only.
Resolution: User approved installing the required tools/software and executing in whatever environments are necessary to resolve the EP-009 builder risk. The repo now has an explicit containerized builder setup path: `live-build/builder/Dockerfile`, `scripts/build-image-builder.sh`, and `scripts/check-image-builder.sh`. This keeps package installation inside the Docker builder image and preserves the human-only boundary for physical device imaging and production release actions.

## BLK-005 (EP-013, M43) — OPEN
Blocker: The locally verified remediation cannot obtain fresh GitHub Actions evidence because publishing it would move a real remote branch. The exact attempted push was `git push origin 52cf5e475e454b6a7b9e9f9f6753068decdc4786:refs/heads/codex/ci-green-20260831`; the guarded runner rejected the remote write because explicit authorization for that exact commit and destination was not present.
Evidence: PR #1 currently points at remote commit `9e85a78b36783ee67a942a6f78506352c7203fab`. Local `main` is at `52cf5e475e454b6a7b9e9f9f6753068decdc4786` and the working tree contains the subsequently verified M44-M53 remediation changes. Local `scripts/verify.sh` exits 0; no hosted run exists for the newer local commits.
Smallest decision needed: Explicitly authorize publication of the current verified remediation to `dominator509/ADAD` branch `codex/ci-green-20260831`, including committing the current worktree if required, or perform that publication outside this session.
Recommended default: Have the repository owner perform or explicitly authorize the current branch update, then inspect the resulting CI run before changing any release claim.
Resolution: (empty — human fills this in)

## BLK-006 (EP-013, M59) — OPEN
Blocker: The requested GitHub repository security-setting enablement,
read-back, and publication cannot be completed because the authenticated GitHub
CLI credential is invalid and the available connector is read-only for these
administration endpoints.
Evidence: `rtk gh auth status` reports `The token in default is invalid` for
the active `dominator509` account. The GitHub connector confirms public
`dominator509/ADAD` with default branch `main`, but returns 404 for
`.github/dependabot.yml`, `.github/workflows/dependency-review.yml`, and
`.github/workflows/codeql.yml` on remote `main`; the connector rejects the
Dependabot-alert and CodeQL-default-setup endpoints as unavailable. Local
`scripts/verify.sh` passes, but that does not publish files or enable remote
settings.
Smallest decision needed: Re-authenticate `gh` with `repo` and `workflow`
scopes, then explicitly authorize publication of the verified local changes
and remote security-setting read-back.
Recommended default: Re-authenticate first, publish through a reviewable
branch/PR, enable only settings whose read-back reports the intended state, and
run a one-line PR diff to prove dependency-review triggering.
Resolution: (empty — human fills this in)
