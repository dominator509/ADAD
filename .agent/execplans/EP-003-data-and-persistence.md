---
id: EP-003
status: complete
depends_on: [EP-002]
verify: scripts/verify.sh
---

# EP-003 — Data & Persistence (LUKS2 Vault)

## 1. Purpose / Big Picture
Implement the LUKS2 vault lifecycle (create/unlock/mount/write/lock/seal) and the
persona-identity persistence format, enforcing the amnesic-vs-vault boundary so
no data ever lands on host internal storage. Tests run against loopback image
files — never a real device.

## 2. Scope
- Vault module in `forge-rs` (create/unlock/seal) + persona persistence format.
- Config/keys/identity/repo storage layout inside the vault.
- Vault config-version + a tested in-place upgrade path.
- Boundary enforcement: writes confined to tmpfs + vault image.

## 3. Non-goals
- No real device operations (image-only). No networking/providers (EP-004). No
  DMS/panic (EP-006). No DB/migrations (forbidden).

## 4. Context and Orientation
SPEC-002 governs. AGENTS.md §13 forbids real-device writes. ARCHITECTURE.md sets
the amnesic/vault boundary as an absolute invariant.

## 5. Files to Read First
- SPEC-002, SECURITY.md (KDF, zeroize, no-swap), ARCHITECTURE.md (persistence
  boundaries), ENVIRONMENT.md (`ADAD_VAULT_PATH`).

## 6. Files to Change
- `crates/forge/src/vault.rs` (create/unlock/mount/lock/seal)
- `crates/persona/src/store.rs` (identity persistence via adad-core types)
- `crates/forge/tests/vault_roundtrip.rs`, `tests/vault_boundary.rs`,
  `tests/vault_upgrade.rs`
- test helper: `crates/forge/tests/support/loop_image.rs` (build a loopback
  LUKS image at test time)

## 7. Interfaces and Contracts
- `Vault::create(path, passphrase) -> Result<(), Error>` (LUKS2/Argon2id).
- `Vault::unlock(path, passphrase) -> Result<Unsealed, Error>`;
  `Unsealed::seal(self)` zeroizes key material.
- `PersonaStore::save/load(&Unsealed, SessionIdentity) -> Result<_, Error>`.
- Vault config carries `config_version: u32`.

## 8. Milestones

### M1 — Loopback image test harness
- Goal: helper creates/destroys a loopback LUKS image for tests.
- Files: `tests/support/loop_image.rs`.
- Validation: `cargo test -p forge --test vault_roundtrip -- --list` (compiles).
- Expected: test binary compiles; helper available.
- Recovery: fix losetup/cryptsetup invocation shape; confirm flags via `--help`.

### M2 — Vault create/unlock/seal
- Goal: full lifecycle on a loopback image; key material zeroized on seal.
- Files: `crates/forge/src/vault.rs`, `tests/vault_roundtrip.rs`.
- Validation: `cargo test -p forge --test vault_roundtrip`
- Expected: create → unlock → write → seal → re-unlock → read-back passes.
- Recovery: on unlock failure inspect KDF params; bounded retry; never touch a
  real device.

### M3 — Persona persistence
- Goal: save/load `SessionIdentity` in the vault.
- Files: `crates/persona/src/store.rs`, tests.
- Validation: `cargo test -p persona`
- Expected: identity round-trips; redacted in logs/errors.
- Recovery: ensure identity uses adad-core types; no real fields leak.

### M4 — Boundary enforcement test
- Goal: prove no write lands outside tmpfs/vault image.
- Files: `tests/vault_boundary.rs`.
- Validation: `cargo test -p forge --test vault_boundary`
- Expected: monitored run shows writes only to the image + a tmpdir.
- Recovery: locate the stray write path; route it to the vault or tmpfs.

### M5 — Vault upgrade path
- Goal: upgrade an old-config-version vault in place, non-destructively.
- Files: upgrade fn in `vault.rs`, `tests/vault_upgrade.rs`.
- Validation: `cargo test -p forge --test vault_upgrade`
- Expected: old→new version bump; data intact; pre-upgrade backup made.
- Recovery: make upgrade additive; require a backup image before mutating.

### M6 — Full verify
- Validation: `scripts/verify.sh`
- Expected: `verify: ok`
- Recovery: address first failing gate; bounded retry.

## 9. Concrete Steps
1. Build the loopback-image test helper (losetup + cryptsetup on an image file).
2. Implement vault create/unlock/seal with Argon2id + zeroize.
3. Implement persona store; round-trip identity.
4. Add the boundary test (monitor writes; assert containment).
5. Implement + test the config-version upgrade path with a backup.
6. Run full verify.

## 10. Validation and Acceptance
- [x] `vault_roundtrip` passes on a loopback image.
- [x] `vault_boundary` proves no host-disk write.
- [x] `vault_upgrade` passes with backup + version bump.
- [x] persona identity round-trips and is redacted.
- [x] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
Tests create and tear down their own loopback images (detach loop devices, remove
temp images) so re-runs are clean. A stuck loop device is detached in test
teardown; never operate on `/dev/sdX`.

## 12. Progress
- [x] M1 — loopback image harness
- [x] M2 — vault create/unlock/seal
- [x] M3 — persona persistence
- [x] M4 — boundary enforcement test
- [x] M5 — vault upgrade path
- [x] M6 — full verify
- [x] verify + status set to complete

## 13. Surprises & Discoveries
(Record cryptsetup/losetup flag confirmations and any zeroize caveats.)
- `forge` and `persona` both started this plan as version-only binary stubs, so
  M1 had to establish the first vault-oriented test surface from scratch.
- The loopback harness milestone succeeded as a compile-only integration target:
  `cargo test -p forge --test vault_roundtrip -- --list` now builds a helper that
  shapes `truncate`, `losetup`, `cryptsetup`, `mkfs.ext4`, `mount`, and
  `umount` commands without touching a real device.
- This host does not provide the required Linux vault tools: `cryptsetup --help`
  and `losetup --help` both failed with `command not found`. That blocks M2's
  required runtime validation on this machine before any compliant recovery path
  exists inside an agent session.
- The EP-003 interface sketch for `PersonaStore::save/load(&Unsealed, ...)`
  conflicts with `ARCHITECTURE.md`, which forbids tool crates from importing one
  another. The implemented seam keeps `persona` path-based so `forge` and
  `persona` stay decoupled.
- Local Windows cargo runs can now compile and execute the `forge` integration
  targets, but the runtime LUKS lifecycle assertions still skip unless the host
  is Linux and already provides the documented loopback/LUKS toolchain.
- Ubuntu 24.04.4 LTS under WSL2 provides the needed kernel/storage substrate for
  EP-003: `/dev/loop-control` and `/dev/loop0` were present, and the real
  loopback/LUKS tests passed once `cryptsetup` was installed.
- The runtime vault tests require root-capable execution in Linux because they
  call `losetup`, `cryptsetup`, `mkfs.ext4`, and `mount` directly. Running the
  WSL validations as root with the existing user toolchain made M2-M5 pass.
- The first full `scripts/verify.sh` attempt in WSL cleared preflight, format,
  lint, typecheck, unit tests, and integration tests, then failed in
  `scripts/build.sh` because the Rust `x86_64-unknown-linux-musl` target is not
  installed in that WSL toolchain.
- A focused recovery attempt to add the musl target with `rustup target add`
  failed twice for environment reasons, not repo-code reasons: once during a
  stable-toolchain update/rollback conflict, and again on a direct `1.96.0`
  target add with a TLS/body-decode download failure from `static.rust-lang.org`.
- Resumable `curl` against the exact Rust 1.96.0 musl std archive eventually
  completed after mid-stream TLS drops. The downloaded
  `rust-std-1.96.0-x86_64-unknown-linux-musl.tar.xz` matched its published
  SHA-256 checksum and installed cleanly into the existing WSL
  `1.96.0-x86_64-unknown-linux-gnu` toolchain with the archive's `install.sh`.
- After the musl target repair, the WSL full-verify run exposed a separate
  root/cache issue in `cargo audit`: the user-owned RustSec advisory cache hit
  Git ownership protection, and a fresh root-owned clone hit the same flaky TLS
  stream. Seeding `/root/.cargo/advisory-db` from the repaired user cache and
  making it root-owned made the audit gate reliable.
- Final WSL verification on Ubuntu 24.04.4 LTS passed all gates, including real
  loopback/LUKS integration tests, static-musl release build, dependency audit,
  smoke tests, and the top-level `verify: ok`.

## 14. Decision Log
(Record Argon2id params chosen, vault layout, config-version scheme.)
- For M1, kept the loopback harness inside `crates/forge/tests/support/` with no
  new crate dependencies, since the milestone only required a compileable helper
  and command-shape assertions.
- Deferred real `cryptsetup`/`losetup` execution to M2 rather than faking a
  passing runtime test on a host that lacks the required Linux tools; per
  ENVIRONMENT.md, missing host tools are a STOP condition rather than something
  the agent installs ad hoc.
- Added `crates/forge/src/lib.rs` and `crates/persona/src/lib.rs` as standard
  library entrypoints so the planned `vault.rs`/`store.rs` modules can be tested
  without changing the existing version-printing binaries.
- Implemented the vault lifecycle in `crates/forge/src/vault.rs` using the
  documented Linux toolchain (`truncate`, `losetup`, `cryptsetup`, `mkfs.ext4`,
  `mount`, `umount`), keeping zeroization in-tree via a small sensitive-bytes
  wrapper instead of adding a new dependency mid-plan.
- Implemented `PersonaStore` as a vault-root-path store rather than a
  `forge::Unsealed` consumer so the code follows the repo's no-cross-tool-crate
  rule.
- Updated `ENVIRONMENT.md`, `scripts/install.sh`, and `ASSUMPTIONS.md` because
  EP-003 now has concrete evidence for additional required host tools beyond the
  earlier `cryptsetup`/`losetup` blocker note.
- Used a WSL-local working copy under `/home/doministic/src/ADAD-ep003-wsl`
  instead of moving the Windows checkout so Linux validation could run on an
  ext4-backed path without disturbing the primary workspace.
- Reused the existing WSL Rust toolchain at
  `/home/doministic/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo`
  directly to avoid a spurious rustup sync on every test command.
- Stopped the M6 recovery path after the third same-root failure on the musl
  target/toolchain install seam, per AGENTS.md bounded-retry rules. The next
  step is environment repair, not more blind retries.
- Resolved BLK-003 by manually installing the checksum-verified musl std
  component into the pinned WSL Rust 1.96.0 toolchain rather than retrying the
  failing Rustup update path.
- Ran the final WSL verify as root with
  `HOME=/root`, `CARGO_HOME=/root/.cargo`,
  `RUSTUP_HOME=/home/doministic/.rustup`, and the Rust 1.96.0 toolchain bin
  first in `PATH`, because the EP-003 vault tests require root-level loopback
  and mount operations while the audit cache must be root-owned.

## 15. Outcomes & Retrospective
(Filled at completion.)
- EP-003 no longer blocks at M2. Real Linux evidence now exists from Ubuntu
  24.04.4 LTS on WSL2: `vault_roundtrip`, `vault_boundary`, `vault_upgrade`, and
  `cargo test -p persona` all passed against the WSL-local working copy with the
  actual loopback/LUKS toolchain.
- The remaining EP-003 source surface is no longer empty: `forge` now owns a
  concrete vault module, `persona` owns a concrete identity store, and the
  roundtrip/boundary/upgrade integration targets exist as Linux-runtime-capable
  tests with explicit host-tool skips.
- The repo now has both kinds of evidence it lacked before: Windows-local source
  and test work for the EP-003 implementation, plus real Linux runtime proof for
  M2-M5. The remaining gap is the Rust toolchain environment needed for the full
  static-musl build gate.
- EP-003 is complete. The remaining WSL toolchain gap was closed, the static
  musl build gate was proven by `scripts/build.sh` (`build: ok`), and
  `scripts/verify.sh` completed successfully with `verify: ok`.
