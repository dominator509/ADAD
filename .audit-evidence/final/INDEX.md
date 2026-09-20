ADAD FINAL EVIDENCE INDEX (task t_bd3bc731 — fix + fresh rerun)

Pre-fix repo HEAD: 03e865bd634af30b39226578dd6257cb5a47ec59
Changes applied in this task: crates/forge/src/vault.rs (2 hunks) only.
Captured: 2026-09-07T18:47:55Z .. 18:57:30Z UTC.
Env used: RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo (real cargo 1.98.1,
          /root/.cargo/bin/cargo, bypassing the host rtk `cargo` alias).
Invocation: scripts run as `sh scripts/<name>.sh` (repo CI form).

Matrix scope: EXECUTABLE commands only. Vault LUKS loopback-image tests DID run
as part of cargo test --workspace (safe, image-file based, per TESTING.md).
Full workspace test suite executed — a superset of the baseline run, which
fail-fast aborted at the first failing binary and left 7 binaries unrun.

Per-command verdicts (exact command line + full stdout/stderr + exit code in
each log):

01-cargo-build-locked.log
  cmd: /root/.cargo/bin/cargo build --locked
  exit: 0  PASS

02-cargo-test-workspace-locked.log
  cmd: /root/.cargo/bin/cargo test --workspace --locked
  exit: 0  PASS  (wall 168s; vault_roundtrip 5/5, all binaries green)

03-cargo-fmt-check.log
  cmd: /root/.cargo/bin/cargo fmt --all -- --check
  exit: 0  PASS

04-scripts-build-sh.log
  cmd: /bin/sh scripts/build.sh
  exit: 101  ENV GAP (unchanged): ring 0.17.14 cc-rs cannot find
    x86_64-linux-musl-gcc. musl C cross-compiler absent on host. Recorded as
    blocker, NOT a source defect; no code/Cargo.lock change made.

05-scripts-lint-sh.log
  cmd: /bin/sh scripts/lint.sh
  exit: 0  PASS  (manual-slice-fill defect fixed)

06-scripts-format-check-sh.log
  cmd: /bin/sh scripts/format-check.sh
  exit: 0  PASS

07-scripts-dependency-audit-sh.log
  cmd: /bin/sh scripts/dependency-audit.sh
  exit: 0  PASS with 1 allowed warning: lru 0.18.0 unsound RUSTSEC-2026-0253
    (potential use-after-free LruCache::pop, 2026-05-12) — unchanged, allowed.

Defects fixed (detail in ../fix-summary.md):
  1. clippy::manual-slice-fill at crates/forge/src/vault.rs:358-360
     -> SensitiveBytes::zeroize uses self.0.fill(0).
  2. dm-crypt mapper-name race: mapper_name_for() keyed only on PID, so two
     concurrent Vault::create in one process collided on adad-vault-<pid> and
     one cryptsetup open failed (I/O error). Fixed by appending unique_suffix()
     (nanos + atomic counter).

Source integrity: only tracked change is crates/forge/src/vault.rs. Cargo.lock
untouched. No test, lint config, allow/skip/mock attribute, or build profile
changed. .audit-evidence/ is untracked by design.
