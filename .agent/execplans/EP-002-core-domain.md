---
id: EP-002
status: not-started
depends_on: [EP-001]
verify: scripts/verify.sh
---

# EP-002 — Core Domain + Vendored Harness Crates

## 1. Purpose / Big Picture
Implement `adad-core` (config schema + validator, error taxonomy, Zero-Clock
epoch, session-identity type, provider enum) as pure tested logic, AND vendor the
two claw-code crates (MCP integration + tool-execution) at a pinned commit under
`vendor/`, proving they build in isolation. This gives Phase 3 a trustworthy
core and the harness building blocks without inheriting claw-code's cloud-first
loop.

## 2. Scope
- `adad-core`: `Config`+validator, `Error` enum (SPEC-006), `ZeroClockEpoch`,
  `SessionIdentity`, `Provider` enum, redacting `Debug` for secrets.
- Vendor claw-code MCP crate + tool-exec crate at a recorded commit; build each
  in isolation; wire them into the workspace (imported later only by
  agent-coding).

## 3. Non-goals
- No I/O in `adad-core`. No provider client, no agent loop (EP-004). No vault
  (EP-003). Do NOT vendor claw-code's cloud-first runtime/CLI crates.

## 4. Context and Orientation
SPEC-001 and SPEC-006 define the core. ARCHITECTURE.md forbids `adad-core` from
importing other ADAD crates and restricts vendored-crate imports to
`agent-coding`. ASSUMPTIONS A2 + A6 are verified here.

## 5. Files to Read First
- SPEC-001, SPEC-006, ARCHITECTURE.md (dependency rules), DECISIONS.md
  (ADR-002), ASSUMPTIONS.md (A2, A6).

## 6. Files to Change
- `crates/adad-core/src/{lib.rs,config.rs,error.rs,epoch.rs,identity.rs,provider.rs}`
- `crates/adad-core/tests/*.rs`
- `vendor/claw-mcp/` and `vendor/claw-tools/` (vendored sources + a `PINNED.md`
  recording the upstream repo, commit hash, and license)
- workspace `Cargo.toml` (add vendored crates as path members)

## 7. Interfaces and Contracts
- `adad-core::Config::validate() -> Result<(), Error>` rejecting unknown keys and
  the `venice + anonymized without opt-in` combo.
- `Error` variants exactly per SPEC-006 table; redacted `Display`.
- `ZeroClockEpoch::from_seed(seed) -> Self` deterministic; no host clock.
- `Provider` = `Local | OpenAi | Venice`.
- Vendored crates expose their upstream public API unchanged (frozen).

## 8. Milestones

### M1 — Error taxonomy
- Goal: `Error` enum per SPEC-006 with redacted Display and exit-code mapping
  helper.
- Files: `crates/adad-core/src/error.rs`, `tests/error.rs`.
- Validation: `cargo test -p adad-core --test error`
- Expected: tests pass (each variant → message + code; redaction holds).
- Recovery: fix the variant/mapping; redaction property test must stay green.

### M2 — Config schema + validator
- Goal: `Config` parse + `validate()` with unknown-key and invalid-combo
  rejection; config-version field.
- Files: `src/config.rs`, `tests/config.rs`.
- Validation: `cargo test -p adad-core --test config`
- Expected: positive/unknown-key/invalid-combo cases pass.
- Recovery: narrow the validator; do not loosen rejection rules.

### M3 — Zero-Clock epoch + identity + provider
- Goal: deterministic `ZeroClockEpoch::from_seed`; `SessionIdentity` (redacted
  Debug); `Provider` enum.
- Files: `src/epoch.rs`, `src/identity.rs`, `src/provider.rs`, tests.
- Validation: `cargo test -p adad-core`
- Expected: determinism + no-host-clock + redaction tests pass.
- Recovery: seed the RNG; ensure no `SystemTime::now()` in epoch.

### M4 — Vendor claw-code crates (pinned) + isolation build
- Goal: vendored MCP + tool-exec crates build standalone at a pinned commit.
- Files: `vendor/claw-mcp/`, `vendor/claw-tools/`, `vendor/*/PINNED.md`,
  workspace `Cargo.toml`.
- Exact edits: copy the two upstream crates at a chosen commit; record repo +
  commit + license in `PINNED.md`; add as path members.
- Validation: `cargo build -p claw-mcp && cargo build -p claw-tools`
- Expected: both build in isolation (no cloud-first runtime pulled in).
- Recovery / **STOP**: if either crate cannot build without dragging in the
  cloud-first runtime/CLI, do NOT vendor the whole harness. Apply bounded retry
  (try the minimal dependency set). If still impossible, write a blocker per
  .agent/LOOP.md and set `status: blocked` — this is the A2 STOP guard.

### M5 — Core architecture guard + full verify
- Goal: prove `adad-core` imports no ADAD crate; only workspace wiring changed.
- Files: none (assertion) — confirm via `crates/adad-core/Cargo.toml`.
- Validation: `scripts/verify.sh`
- Expected: `verify: ok`
- Recovery: remove any accidental ADAD-crate dep from `adad-core`.

## 9. Concrete Steps
1. Implement `error.rs` with redacted Display + code map; test.
2. Implement `config.rs` validator; test all cases.
3. Implement epoch/identity/provider; test determinism + redaction.
4. Clone claw-code at a chosen commit; extract the MCP + tools crates into
   `vendor/`; write `PINNED.md`; add to workspace; build each in isolation.
5. Confirm `adad-core` has no ADAD-crate deps; run full verify.

## 10. Validation and Acceptance
- [ ] `cargo test -p adad-core` green (error/config/epoch/identity/provider).
- [ ] Redaction property test passes.
- [ ] `cargo build -p claw-mcp` and `cargo build -p claw-tools` succeed.
- [ ] `vendor/*/PINNED.md` records repo + commit + license.
- [ ] `adad-core/Cargo.toml` has no ADAD-crate dependency.
- [ ] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
Core modules are pure and re-runnable. Vendoring checks for an existing
`vendor/claw-*` before re-copying. A half-vendored crate is completed, not
duplicated.

## 12. Progress
- [ ] M1 — error taxonomy
- [ ] M2 — config schema + validator
- [ ] M3 — epoch + identity + provider
- [ ] M4 — vendor claw crates (pinned) + isolation build (STOP guard)
- [ ] M5 — core architecture guard + full verify
- [ ] verify + status set to complete

## 13. Surprises & Discoveries
(Record the claw-code commit chosen and any trimming needed to isolate the two
crates. Record failed isolation hypotheses if M4 struggles.)

## 14. Decision Log
(Record: pinned commit hash; license confirmation; any crate feature disabled to
achieve static/isolated build; ADR-002 realized.)

## 15. Outcomes & Retrospective
(Filled at completion — especially whether A2 held or triggered the STOP.)
