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
