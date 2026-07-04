---
id: EP-003
status: not-started
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
- [ ] `vault_roundtrip` passes on a loopback image.
- [ ] `vault_boundary` proves no host-disk write.
- [ ] `vault_upgrade` passes with backup + version bump.
- [ ] persona identity round-trips and is redacted.
- [ ] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
Tests create and tear down their own loopback images (detach loop devices, remove
temp images) so re-runs are clean. A stuck loop device is detached in test
teardown; never operate on `/dev/sdX`.

## 12. Progress
- [ ] M1 — loopback image harness
- [ ] M2 — vault create/unlock/seal
- [ ] M3 — persona persistence
- [ ] M4 — boundary enforcement test
- [ ] M5 — vault upgrade path
- [ ] M6 — full verify
- [ ] verify + status set to complete

## 13. Surprises & Discoveries
(Record cryptsetup/losetup flag confirmations and any zeroize caveats.)

## 14. Decision Log
(Record Argon2id params chosen, vault layout, config-version scheme.)

## 15. Outcomes & Retrospective
(Filled at completion.)
