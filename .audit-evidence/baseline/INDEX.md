ADAD BASELINE EVIDENCE INDEX (task t_bf7d4ee4 — sprint baseline capture)
Repo HEAD: 03e865bd634af30b39226578dd6257cb5a47ec59
Captured:  2026-09-07T18:32:28Z .. 18:33:58Z UTC (log 08 diagnostic finished later)
Env used:  RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo (real cargo 1.98.1,
           bypassing the host's rtk `cargo` alias which masks failures)
Invocation: scripts run as `sh scripts/<name>.sh` (repo CI form; files committed
           mode 100644, non-executable).

Matrix scope: EXECUTABLE baseline commands only. OS-image/LUKS(real)/VPS/XMR
categories were NOT run (task records them NOT_RUNNABLE_ENV elsewhere). Vault
LUKS loopback-image tests DID run as part of cargo test --workspace (safe,
image-file based, per TESTING.md).

Per-command verdicts (exact command line + full stdout/stderr + exit code in
each log):

01-cargo-build-locked.log
  cmd: /root/.cargo/bin/cargo build --locked
  exit: 0  PASS  (workspace debug build, deterministic, no source drift)

02-cargo-test-workspace-locked.log
  cmd: /root/.cargo/bin/cargo test --workspace --locked
  exit: 101 FAIL
  failure: crates/forge/tests/vault_roundtrip.rs:178
    vault_roundtrip_runs_when_linux_host_tools_are_available
    "vault image is created: Error { variant: Io, message: I/O error }"
  context: other 4 tests in that binary passed; all other crates passed
    (~60 tests). Diagnostics in 08 show the test PASSES in isolation twice
    (24.75s / 96.36s) with cryptsetup "keyslot operation could fail ... more
    than available memory" warnings -> failure is parallel-suite / resource
    sensitive (LUKS keyslot memory pressure), not a deterministic assertion.

03-cargo-fmt-check.log
  cmd: /root/.cargo/bin/cargo fmt --all -- --check   (matrix "rustfmt --check")
  exit: 0  PASS

04-scripts-build-sh.log
  cmd: /bin/sh scripts/build.sh   (static x86_64-unknown-linux-musl release)
  exit: 101 FAIL
  cause: ring v0.17.14 build script (cc-rs) cannot find C cross-compiler
    "x86_64-linux-musl-gcc": No such file or directory.
    ENVIRONMENT GAP: musl C toolchain not installed on host. Repo pins only
    linker rust-lld (.cargo/config.toml); ring still needs a musl C compiler.
    Not a source defect; no code change made (task forbids source edits).

05-scripts-lint-sh.log
  cmd: /bin/sh scripts/lint.sh   (cargo clippy --workspace --all-targets
       --all-features -- -D warnings)
  exit: 101 FAIL
  cause: clippy::manual-slice-fill denied by -D warnings at
    crates/forge/src/vault.rs:358-360 (manual byte-zeroing loop ->
    `self.0.fill(0)`). REAL code lint defect introduced vs pinned stable
    clippy; forge task should apply the smallest fix.

06-scripts-format-check-sh.log
  cmd: /bin/sh scripts/format-check.sh   (cargo fmt --all --check)
  exit: 0  PASS

07-scripts-dependency-audit-sh.log
  cmd: /bin/sh scripts/dependency-audit.sh   (cargo audit; safe to run:
       cargo-audit 0.22.2 installed, advisory DB fetched, no device/egress
       mutation)
  exit: 0  PASS with 1 allowed warning: lru 0.18.0 unsound
    RUSTSEC-2026-0253 (potential use-after-free LruCache::pop, 2026-05-12)

08-diagnostic-vault-roundtrip-isolated.log  [SUPPLEMENTARY, not matrix]
  cmd: cargo test -p forge --test vault_roundtrip
       vault_roundtrip_runs_when_linux_host_tools_are_available --locked
  exit: 0  (isolated rerun passes -> 02 failure is parallelism/resource
       sensitive, not deterministic)

Source integrity: tracked files 0 modified after all runs (each log header
records git_status: 0). Cargo.lock untouched (--locked everywhere). This
directory (.audit-evidence/) is untracked by design.
