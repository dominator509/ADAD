---
id: EP-001
status: complete
depends_on: [EP-000]
verify: scripts/verify.sh
---

# EP-001 — Foundation

## 1. Purpose / Big Picture
Establish the Rust workspace, static-musl toolchain, formatting/lint/type gates,
test harness, and CI baseline so every later ExecPlan has a verifiable
substrate. After this plan, `scripts/verify.sh` runs end-to-end on a minimal but
real workspace, and all placeholder scripts resolve to real commands.

## 2. Scope
- Create the cargo workspace with `adad-core` and empty binary crates for the
  eight tools.
- Pin the toolchain and the `x86_64-unknown-linux-musl` target.
- Wire fmt/clippy/check/test/build to actually run.
- Add a minimal CI config that runs `scripts/verify.sh`.
- Add `.gitignore`, `rust-toolchain.toml`, workspace `Cargo.toml`.

## 3. Non-goals
- No domain logic (EP-002). No vendoring claw-code (EP-002). No networking, UI,
  vault, wallet, or OS image work.

## 4. Context and Orientation
Greenfield. Constraints: static musl for all core binaries; no dynamic linking;
Rust 2021. The workspace layout is defined in ARCHITECTURE.md → Repository map.

## 5. Files to Read First
- ARCHITECTURE.md (repo map, dependency rules), ENVIRONMENT.md (tools/versions),
  COMMANDS.md (lifecycle scripts), SPEC-001 (what adad-core will hold).

## 6. Files to Change
- `Cargo.toml` (workspace), `rust-toolchain.toml`, `.gitignore` (exists — extend
  if needed), `crates/adad-core/{Cargo.toml,src/lib.rs}`,
  `crates/{forge,leakguard,agent-coding,xmr-wallet,vps-deploy,persona,metafuse,git-spoof}/{Cargo.toml,src/main.rs}`,
  `.github/workflows/ci.yml` (or `.gitlab-ci.yml`), `rustfmt.toml`,
  `clippy.toml` (if needed).

## 7. Interfaces and Contracts
- Each binary crate exposes a `--version` flag (clap or manual) returning exit 0.
- `adad-core` is a library crate with an empty-but-compiling public surface
  (`pub fn version() -> &'static str`), no ADAD-crate deps.

## 8. Milestones

### M1 — Workspace skeleton
- Goal: a compiling cargo workspace with all crates.
- Files to read: ARCHITECTURE.md repo map.
- Files to change: `Cargo.toml`, `rust-toolchain.toml`, all crate `Cargo.toml`
  + minimal `src`.
- Exact edits: workspace `[workspace] members = [...]`; `adad-core` lib with
  `pub fn version()`; each tool `main.rs` printing name + `--version`.
- Validation: `scripts/typecheck.sh`
- Expected: `typecheck: ok`
- Recovery: if a crate fails to resolve, fix its `Cargo.toml` members/paths;
  bounded-retry rule on repeated failure.

### M2 — Format + lint gates
- Goal: fmt and clippy pass cleanly.
- Files to change: `rustfmt.toml`, `clippy.toml` (if needed), source formatting.
- Validation: `scripts/format-check.sh && scripts/lint.sh`
- Expected: `format check: ok` then `lint: ok`
- Recovery: run `cargo fmt --all` to fix formatting; address clippy findings
  narrowly (no broad rewrites).

### M3 — Static musl build
- Goal: all binaries build as static musl.
- Files to read: `scripts/build.sh` (asserts static).
- Files to change: crate configs if a dep breaks static linking (record any).
- Validation: `scripts/build.sh`
- Expected: `build: ok` (and static assertion passes)
- Recovery: if a crate links dynamically, identify the offending dep; if core,
  this is a possible STOP (record blocker); prefer a static-friendly alternative.

### M4 — Test harness + smoke
- Goal: unit + smoke run green on the skeleton.
- Files to change: a trivial unit test in `adad-core`; ensure `--version` works
  for smoke.
- Validation: `scripts/test-unit.sh && scripts/smoke-test.sh`
- Expected: `unit tests: ok` then `smoke test: ok`
- Recovery: fix the failing crate; do not weaken tests.

### M5 — CI baseline + full verify
- Goal: CI runs verify; full verify passes locally.
- Files to change: `.github/workflows/ci.yml` (runs `scripts/verify.sh`).
- Validation: `scripts/verify.sh`
- Expected: `verify: ok`
- Recovery: address the first failing gate; bounded retry.

## 9. Concrete Steps
1. Write workspace `Cargo.toml` + `rust-toolchain.toml` (stable, musl target).
2. Create `adad-core` lib and eight binary crates with minimal `--version`.
3. Add `rustfmt.toml`; run fmt; resolve clippy.
4. Confirm static build via `scripts/build.sh`.
5. Add a unit test; confirm smoke.
6. Add CI; run full verify.

## 10. Validation and Acceptance
- [x] `scripts/typecheck.sh` → `typecheck: ok`
- [x] `scripts/format-check.sh` + `scripts/lint.sh` green
- [x] `scripts/build.sh` → `build: ok` (static)
- [x] `scripts/test-unit.sh` + `scripts/smoke-test.sh` green
- [x] `scripts/verify.sh` → `verify: ok`
- [x] CI config runs `scripts/verify.sh`

## 11. Idempotence and Recovery
Re-running is safe: crate creation checks for existing files; fmt/lint/build are
idempotent. A half-created crate is fixed by completing its `Cargo.toml`+`src`.

## 12. Progress
- [x] M1 — workspace skeleton
- [x] M2 — format + lint gates
- [x] M3 — static musl build
- [x] M4 — test harness + smoke
- [x] M5 — CI baseline + full verify
- [x] verify + status set to complete

## 13. Surprises & Discoveries
- Windows sandboxed PowerShell failed to launch with `CreateProcessAsUserW
  failed: 5`; `cmd.exe` plus `rtk proxy C:\Progra~1\Git\bin\bash.exe ...`
  successfully runs repository shell scripts.
- First sandboxed `scripts/typecheck.sh` attempt could not let rustup create a
  temp file under `C:\Users\domin\.rustup`; rerunning with approved escalation
  allowed rustup to finish and the typecheck passed.
- The first static-musl build failed because the target attempted to invoke
  missing host linker `cc`; a target-specific `.cargo/config.toml` entry using
  Rust's bundled `rust-lld` fixed the build without adding host packages.
- `scripts/smoke-test.sh` printed `smoke test: ok` on Windows, but did not emit
  per-tool lines; the `-x` check does not treat the Linux musl artifacts as
  executable on this host.
- The original integration script used `cargo test --workspace --test '*'`,
  which cargo rejected with `no test target matches pattern '*'`; switching to
  `cargo test --workspace --tests` made the integration gate work for the
  skeleton workspace.
- `cargo-audit` was installed as `cargo audit`; Git Bash could not find a bare
  `cargo-audit` command, so the dependency audit script now checks the same
  cargo subcommand form it invokes.

## 14. Decision Log
- Pinned `rust-toolchain.toml` to `stable` with the
  `x86_64-unknown-linux-musl` target, matching ENVIRONMENT.md and avoiding a
  narrower point release until the project has a reason to freeze one.
- Implemented `--version` manually in each skeleton binary instead of adding
  `clap`, keeping EP-001 dependency-free and static-musl-friendly.
- Added `.cargo/config.toml` as a standard Rust workspace build config, outside
  the original Files to Change list, to keep `x86_64-unknown-linux-musl` builds
  self-contained on this Windows host by using `rust-lld`.
- Added `.github/workflows/ci.yml` as the CI baseline; it installs stable Rust
  with the musl target, installs `cargo-audit`, and runs `scripts/verify.sh`.
- Updated `scripts/test-integration.sh` and `scripts/dependency-audit.sh`,
  outside the original Files to Change list, because full verify exposed
  command-shape issues in existing lifecycle scripts.
- Installed `cargo-audit v0.22.2` per `scripts/dependency-audit.sh`; the final
  verify run required escalation so cargo-audit could update its advisory DB
  under `C:\Users\domin\.cargo`.

## 15. Outcomes & Retrospective
- Shipped a dependency-free Rust workspace with `adad-core` plus eight skeleton
  binary crates, a stable+musl toolchain pin, rustfmt configuration, CI, and
  lifecycle script fixes needed for `scripts/verify.sh`.
- All EP-001 acceptance gates passed locally; final `scripts/verify.sh` printed
  `verify: ok`.
- Residual risk: Windows smoke validation did not execute per-tool Linux musl
  binaries because the script's executable check skips them on this host; the
  static build gate still produced the musl artifacts successfully.
