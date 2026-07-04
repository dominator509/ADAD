---
id: EP-002
status: complete
depends_on: [EP-001]
verify: scripts/verify.sh
---

# EP-002 — Core Domain + MCP Foundation

## 1. Purpose / Big Picture
Implement `adad-core` (config schema + validator, error taxonomy, Zero-Clock
epoch, session-identity type, provider enum) as pure tested logic, AND establish
an official MCP Rust foundation inside `agent-coding` while keeping execution
logic ADAD-owned. This gives Phase 3 a trustworthy core and a clean harness seam
for a Claude-Code-like UX without inheriting third-party runtime sprawl.

## 2. Scope
- `adad-core`: `Config`+validator, `Error` enum (SPEC-006), `ZeroClockEpoch`,
  `SessionIdentity`, `Provider` enum, redacting `Debug` for secrets.
- `agent-coding`: add a minimal official MCP Rust foundation plus ADAD-owned
  execution/naming seam that future plans can grow toward Claude-Code-like
  feature parity.

## 3. Non-goals
- No I/O in `adad-core`. No provider client, no agent loop (EP-004). No vault
  (EP-003). Do NOT import third-party cloud-first runtime/CLI crates into
  `agent-coding`.

## 4. Context and Orientation
SPEC-001 and SPEC-006 define the core. ARCHITECTURE.md forbids `adad-core` from
importing other ADAD crates and restricts MCP transport/runtime dependencies to
`agent-coding`. ASSUMPTIONS A2 + A6 are verified here.

## 5. Files to Read First
- SPEC-001, SPEC-006, ARCHITECTURE.md (dependency rules), DECISIONS.md
  (ADR-009), ASSUMPTIONS.md (A2, A6).

## 6. Files to Change
- `crates/adad-core/src/{lib.rs,config.rs,error.rs,epoch.rs,identity.rs,provider.rs}`
- `crates/adad-core/tests/*.rs`
- `crates/agent-coding/Cargo.toml`
- `crates/agent-coding/src/{lib.rs,main.rs,execution.rs,mcp.rs}`
- workspace `Cargo.toml` / `Cargo.lock`

## 7. Interfaces and Contracts
- `adad-core::Config::validate() -> Result<(), Error>` rejecting unknown keys and
  the `venice + anonymized without opt-in` combo.
- `Error` variants exactly per SPEC-006 table; redacted `Display`.
- `ZeroClockEpoch::from_seed(seed) -> Self` deterministic; no host clock.
- `Provider` = `Local | OpenAi | Venice`.
- `agent-coding` preserves Claude-Code-like MCP tool qualification
  (`mcp__server__tool`) while keeping execution/policy logic first-party.

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

### M4 — Official MCP Rust foundation + ADAD-owned execution seam
- Goal: `agent-coding` builds against the official MCP Rust SDK and exposes an
  in-tree execution/naming seam for future Claude-Code-like tool UX.
- Files: `crates/agent-coding/Cargo.toml`, `crates/agent-coding/src/*.rs`,
  workspace `Cargo.lock`.
- Exact edits: add the official MCP Rust SDK dependency; implement a minimal
  MCP-facing module plus ADAD-owned execution registry/naming helpers; keep
  provider/control-loop logic out of scope for later plans.
- Validation: `cargo test -p agent-coding`
- Expected: tests pass; the crate compiles with `rmcp`; MCP tool qualification
  stays first-party and Claude-Code-like.
- Recovery / **STOP**: if the official SDK cannot support a minimal compileable
  foundation without violating static/dependency constraints, record the exact
  mismatch and block the plan. Do not fall back to claw-code runtime imports.

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
4. Add the official MCP Rust SDK to `agent-coding`; implement a minimal
   MCP-facing module and first-party execution/naming seam; validate it.
5. Confirm `adad-core` has no ADAD-crate deps; run full verify.

## 10. Validation and Acceptance
- [x] `cargo test -p adad-core` green (error/config/epoch/identity/provider).
- [x] Redaction property test passes.
- [x] `cargo test -p agent-coding` passes with official MCP Rust wiring.
- [x] `agent-coding` keeps MCP naming/execution logic in-tree.
- [x] `adad-core/Cargo.toml` has no ADAD-crate dependency.
- [x] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
Core modules are pure and re-runnable. `agent-coding` foundation work is local
source plus pinned Rust dependencies, so re-running should only update
`Cargo.lock` when the dependency set changes intentionally.

## 12. Progress
- [x] M1 — error taxonomy
- [x] M2 — config schema + validator
- [x] M3 — epoch + identity + provider
- [x] M4 — official MCP Rust foundation + ADAD-owned execution seam
- [x] M5 — core architecture guard + full verify
- [x] verify + status set to complete

## 13. Surprises & Discoveries
(Record the MCP substrate chosen, any parity-preserving naming rules, and any
failed SDK or dependency hypotheses if M4 struggles.)
- The first `adad-core` implementation used the `toml` crate for config parsing,
  but milestone validation failed because crates.io was unreachable in this
  environment. Replacing it with a small built-in line parser kept `adad-core`
  offline-friendly and dependency-light while preserving unknown-key rejection
  and the required validator rules.
- Cloned `ultraworkers/claw-code` at `4ea31c1bc91c4e9bcbd67d51c550c01e127e6d0d`
  and confirmed the repo is still MIT-licensed, but the current upstream crate
  layout no longer matches the original A2 vendoring seam.
- The smallest dependency-set diagnostic showed `tools` depends directly on
  `api`, `commands`, `plugins`, and `runtime`, while `runtime` itself owns MCP
  plumbing plus the core conversation loop. That overturned the original A2 and
  led to ADR-009: official MCP Rust SDK plus ADAD-owned execution logic.
- The first RMCP integration used `rmcp 0.16.0` because that matched the
  readily available README examples, but `cargo audit` flagged
  `RUSTSEC-2026-0189` against that line. Upgrading to `rmcp 2.1.0` resolved the
  advisory and required a small API refresh (`ContentBlock`, `ServerInfo::new`,
  explicit `tool_handler(router = self.tool_router)`).

## 14. Decision Log
(Record: SDK version chosen; any crate feature flags; parity-preserving naming
rules; ADR-009 realization.)
- Mapped `adad-core::Error` variants to stable exit codes 10-21 in taxonomy
  order, since SPEC-006 requires stable codes but does not prescribe numeric
  values.
- Implemented config parsing without third-party dependencies after the initial
  network-bound `toml` approach failed; this keeps `adad-core` pure, small, and
  runnable in offline validation environments.
- Recorded upstream `ultraworkers/claw-code` commit
  `4ea31c1bc91c4e9bcbd67d51c550c01e127e6d0d` as the tested failed A2 candidate
  and confirmed its license remains MIT.
- Superseded ADR-002 with ADR-009 after the user chose the official MCP Rust SDK
  plus ADAD-owned execution logic rather than a broader claw-code import.
- Chose `rmcp 2.1.0` with `server` and `transport-io` features for the EP-002
  foundation after `cargo audit` rejected `rmcp 0.16.0`; kept MCP naming and
  execution registration in ADAD-owned code under `agent-coding`.
- Added `cargo fmt --all` to `COMMANDS.md` because `scripts/format-check.sh`
  already proves that exact repair shape and the session needed a repo-approved
  formatting command to satisfy `verify`.
- Derived `Default` for `adad-core::Provider` to satisfy the strict clippy gate
  exercised by `scripts/verify.sh`.

## 15. Outcomes & Retrospective
(Filled at completion — especially whether the new A2 held and how much of the
Claude-Code-like harness seam was established.)
- The original claw-code vendoring assumption failed at commit
  `4ea31c1bc91c4e9bcbd67d51c550c01e127e6d0d`, and the plan pivoted to ADR-009:
  official MCP Rust SDK plus ADAD-owned execution logic.
- This plan now treats Claude-Code-like feel and feature breadth as a product
  target implemented incrementally in `agent-coding`, not as a third-party
  runtime import.
- EP-002 now leaves behind a tested MCP foundation rather than full feature
  parity: `agent-coding` compiles against the official SDK, preserves
  Claude-Code-like `mcp__server__tool` naming, and owns its execution registry
  in-tree.
- `scripts/verify.sh` passed on 2026-07-03. Static-musl binary verification and
  Linux execution smoke were skipped on this Windows host exactly as the scripts
  document; Linux CI remains authoritative for those checks.
