# SPEC-002 — Data Model (Vault & Persistence)

- **Status:** active
- **Owner:** architect
- **Roadmap phase:** Phase 2
- **Linked ExecPlans:** EP-003

## User-visible goal
Persistent data survives across boots inside an encrypted vault, while amnesic
data never touches host storage; the boundary is enforced and testable.

## Non-goals
No relational DB, ORM, or schema migrations. No persistence outside the vault.

## Terms
- **Vault:** LUKS2 (Argon2id) container; image file in tests, partition on
  device in production.
- **Sealed/unsealed:** whether the vault key material is present in RAM.

## Required behavior
- The system MUST create a vault (loopback image in tests) with LUKS2/Argon2id,
  unlock it with a passphrase, mount it, write config/keys/identity/repos, then
  lock and seal it (scrubbing key material from RAM).
- Vault contents MUST include: `config` (with config-version), provider keys,
  the persona `SessionIdentity`, and repository storage.
- No data MUST be written outside tmpfs or the vault image during any operation.
- A vault-layout upgrade MUST be an explicit, tested in-place path that bumps
  config-version and never destroys data without a backup.

## Inputs
Passphrase; config/identity/keys to persist; an existing or new vault image.

## Outputs
An encrypted vault image with the expected structure; a scrubbed RAM state on
lock.

## Error states
Wrong passphrase → `Error::VaultUnlock`; corrupt/incompatible version →
`Error::VaultVersion`; write outside boundary → test failure (must never occur).

## Data rules
- Config-version gates compatibility; upgrades are forward-only within a window.
- Backups are image-level copies; restore places the image at `ADAD_VAULT_PATH`.

## Security rules
Argon2id KDF; key material zeroized on lock/shutdown/panic; no swap; header never
committed or copied off-box by automation.

## Accessibility rules
Passphrase entry is keyboard-only with no echo.

## Performance rules
Unlock/seal within a couple seconds on target hardware (image tests may be
faster).

## Observability rules
Log vault lifecycle events (create/unlock/lock) with NO passphrase or key
material; redacted.

## Required tests
- Integration: create → unlock → write → lock → re-unlock → read-back on a
  loopback image; asserts round-trip integrity.
- Boundary: assert no path outside tmpfs/vault is written (monitor writes).
- Upgrade: create an old-config-version vault, run the upgrade, assert data
  intact and version bumped; a pre-upgrade backup exists.
- Negative: wrong passphrase fails cleanly; no partial mount left behind.

## Acceptance criteria
- [ ] Vault round-trip integration test passes on a loopback image.
- [ ] Boundary test proves no host-disk write.
- [ ] Upgrade test passes with backup + version bump.
- [ ] `scripts/verify.sh` exits 0.
