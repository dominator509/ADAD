---
id: EP-013
status: complete
depends_on: [EP-012]
verify: scripts/verify.sh
---

# EP-013 — Independent Audit Remediation

## 1. Purpose / Big Picture

Correct the confirmed low-level defects identified by the independent audit and
remove false release confidence where the repository currently cannot prove a
production capability. This plan is deliberately bounded: external provider,
wallet, VPS, WireGuard, FUSE, DMS, panic, hardware, and image-runtime work
remain separate engineering tracks unless their concrete adapters already exist
in the repository.

## 2. Scope

- Use a certificate-validating Rust HTTPS client for inference requests.
- Make fallback egress fail closed by default.
- Make vault configuration string serialization and parsing symmetric.
- Pass vault passphrases to `cryptsetup` through a closed stdin pipe, never a
  plaintext temporary file.
- Bind image and on-image leak evidence to the current source revision and image
  digest.
- Make release-critical CI and local release gates fail when required image
  evidence is unavailable or stale.
- Prevent completed-plan bookkeeping from producing a readiness pass while the
  evidence-first production checklist still has open gates.
- Reconcile public release documentation with the current implementation.

## 3. Non-goals

- No real wallet transfer, VPS provisioning, WireGuard interface activation,
  FUSE mount, DMS destruction, panic/kexec, hardware boot, or production
  deployment.
- No claims that pure models, mocks, or headless TUI tests prove those external
  behaviors.
- No real device, vault, remote host, wallet, or secret is touched.

## 4. Context and Orientation

The audit correctly identifies that the completed historical ExecPlans closed
mock/model seams while leaving production adapters deferred. The current source
also contains independently actionable defects in the provider client, config
parser, vault secret handling, and release evidence chain. The source and tests
are authoritative; historical readiness text is not evidence of current image
or external-service behavior.

## 5. Files to Read First

- `AGENTS.md`
- `COMMANDS.md`
- `scripts/production-readiness-check.sh`
- `scripts/build-image-inside.sh`
- `tests/os/run-qemu-leak-battery-inside.sh`
- `crates/agent-coding/src/client.rs`
- `crates/adad-core/src/config.rs`
- `crates/forge/src/vault.rs`

## 6. Files to Change

- `.agent/execplans/EP-013-independent-audit-remediation.md`
- `COMMANDS.md`
- `Cargo.lock`
- `crates/agent-coding/Cargo.toml`
- `crates/agent-coding/src/client.rs`
- `crates/agent-coding/tests/egress_guard.rs`
- `crates/agent-coding/tests/failure_provider.rs`
- `crates/adad-core/src/config.rs`
- `crates/adad-core/tests/config.rs`
- `crates/forge/src/vault.rs`
- `scripts/build-image-inside.sh`
- `scripts/build.sh`
- `scripts/production-readiness-check.sh`
- `scripts/smoke-test.sh`
- `scripts/test-integration.sh`
- `tests/e2e/run-leak-battery.sh`
- `tests/os/run-qemu-leak-battery-inside.sh`
- `live-build/builder/Dockerfile`
- `.github/workflows/ci.yml`
- `README.md`
- `LICENSE`
- `HOW_TO_USE.md`
- `PROJECT_BRIEF.md`
- `.agent/specs/SPEC-003-api-contracts.md`
- `PRODUCTION_READINESS.md`
- `RELEASE.md`
- `AGENTS.md`
- `CONTRIBUTING.md`
- `ROLLBACK.md`
- `SECURITY.md`
- `ROADMAP.md`

## 7. Interfaces and Contracts

- `OpenAiCompatClient` supports local HTTP only for loopback endpoints and HTTPS
  for fallback endpoints, using normal certificate and hostname validation.
- A newly constructed fallback client denies requests until an authoritative
  egress state is injected.
- Rendered config strings round-trip all supported basic-string escapes.
- `cryptsetup` receives the passphrase on stdin and no passphrase file is
  created.
- `build/adad-image.provenance` records the source SHA, source tree, image
  SHA-256, and source date; readiness verifies all of them against the current
  clean checkout and leak pass marker.
- Missing release image evidence is a failure for the release job, while normal
  source-only development verification remains explicit about its scope.

## 8. Milestones

### M1 — Secure provider transport and egress default

- Goal: HTTPS provider URLs are supported with TLS and fallback clients deny by
  default.
- Files to read: `crates/agent-coding/src/client.rs`, provider tests, Cargo
  manifests.
- Files to change: `crates/agent-coding/Cargo.toml`, `Cargo.lock`,
  `crates/agent-coding/src/client.rs`, `crates/agent-coding/tests/egress_guard.rs`.
- Exact edits expected: replace raw provider transport with a rustls-backed
  client, retain the OpenAI-compatible request shape, and add the default-deny
  regression assertion.
- Validation command: `cargo test -p agent-coding`.
- Expected result: all agent-coding tests pass.
- Recovery: inspect the exact client API/compiler error and make one narrow
  compatibility correction; do not weaken TLS or egress assertions.

### M2 — Config and vault secret correctness

- Goal: config values round-trip exactly and no passphrase temp file is created.
- Files to read: `crates/adad-core/src/config.rs`, `crates/forge/src/vault.rs`,
  related tests.
- Files to change: `crates/adad-core/src/config.rs`,
  `crates/adad-core/tests/config.rs`, `crates/forge/src/vault.rs`.
- Exact edits expected: decode TOML basic-string escapes symmetrically and pipe
  the sensitive bytes to `cryptsetup --key-file -`.
- Validation command: `cargo test -p adad-core && cargo test -p forge`.
- Expected result: both package test commands pass.
- Recovery: preserve image-file-only vault tests and isolate any platform issue;
  never restore plaintext file handling.

### M3 — Artifact provenance and non-skipping release gates

- Goal: a readiness pass is impossible for an absent, stale, or unbound image.
- Files to read: build, integration, leak, CI, and readiness scripts.
- Files to change: `scripts/build-image-inside.sh`,
  `tests/os/run-qemu-leak-battery-inside.sh`,
  `scripts/production-readiness-check.sh`, `scripts/test-integration.sh`,
  `.github/workflows/ci.yml`.
- Exact edits expected: emit and verify provenance, include hashes in the leak
  marker, make the CI release job build and test the exact image, and reject any
  remaining unchecked production-readiness criterion.
- Validation command: `scripts/verify.sh` and shell lint through
  `scripts/lint.sh`.
- Expected result: source-only verification succeeds; release requirements are
  explicit and fail closed when image infrastructure is absent.
- Recovery: inspect the first failing gate and apply the bounded retry rule.

### M4 — Truthful public documentation

- Goal: public docs disclose the current experimental boundary and no longer
  describe claw-code as the active MCP implementation.
- Files to read: `README.md`, `HOW_TO_USE.md`, `SPEC-003`,
  `PRODUCTION_READINESS.md`.
- Files to change: those files plus `LICENSE`.
- Exact edits expected: add MIT text, document supported/currently deferred
  paths, and reconcile the MCP and release-readiness descriptions.
- Validation command: `scripts/security-check.sh`.
- Expected result: `security check: ok`.
- Recovery: keep docs factual and preserve the no-secret scan.

## 9. Concrete Steps

1. Add the TLS transport and default-deny regression test.
2. Fix parser escaping and vault stdin handling with focused tests.
3. Add source/image provenance and required release CI execution.
4. Rewrite only the stale public claims directly contradicted by current code.
5. Run the full source verification gate and leave external validation clearly
   outstanding.

## 10. Validation and Acceptance

- [x] Agent provider tests pass with HTTPS support and default-deny fallback.
- [x] Config escape round-trip tests pass.
- [x] Vault tests pass without passphrase temp-file code.
- [x] Source/image provenance is emitted and checked by release gates.
- [x] CI has a non-optional image build/boot/leak path.
- [x] Readiness rejects an unchecked production evidence criterion.
- [x] Public README, license, and MCP documentation are truthful.
- [x] `scripts/verify.sh` exits 0.

## 11. Idempotence and Recovery

All changes are source-only until an explicitly invoked image build. Image and
provenance files remain under ignored `build/`. Tests use loopback or mock
services and image files only. If a validation command fails, follow the
repository three-strike rule and preserve the first exact error in this plan.

## 12. Progress

- [x] M1 — secure provider transport and egress default (`cargo test -p agent-coding`: 42 passed)
- [x] M2 — config and vault secret correctness (`cargo test -p adad-core`: 18 passed; `cargo test -p forge`: 13 passed)
- [x] M3 — artifact provenance and non-skipping release gates (`scripts/verify.sh`: 0; release mode requires image/provenance and rejects unchecked readiness criteria)
- [x] M4 — truthful public documentation (`scripts/security-check.sh`: ok)
- [x] verify + status set to complete (`scripts/verify.sh`: 0)

## 13. Surprises & Discoveries

- The repository had no active plan because EP-012 and every predecessor were
  marked complete even though EP-004 and EP-006 explicitly recorded deferred
  production adapters. This plan therefore starts as `in-progress` and does not
  retroactively convert model/mock acceptance into production evidence.

## 14. Decision Log

- 2026-08-27: Created EP-013 as the first post-audit remediation plan because
  `scripts/next-step.sh` returned `DONE` while the audit identified source-level
  release blockers.
- 2026-08-27: Selected a rustls-backed `ureq` transport because the existing
  `std::net::TcpStream` implementation cannot perform HTTPS certificate
  validation and the repository requires static-musl-compatible binaries.
- 2026-08-27: Did not pin `rust-toolchain.toml` in this plan. The audit correctly
  identifies the stable channel as mutable, but this checkout lacks the pinned
  rustfmt/clippy components and cannot safely complete rustup downloads; the
  reproducibility claim is narrowed in documentation until a controlled builder
  can pin and verify all inputs.
- 2026-08-27: Adjusted non-Linux `scripts/build.sh` to perform a native release
  compile and defer only the static-musl check to Linux CI. The prior ordering
  attempted the musl build first and failed on Windows when the TLS dependency
  required a musl C compiler that is intentionally not part of this checkout.
- 2026-08-27: Kept real wallet/VPS/security-backend implementation out of this
  bounded plan; no repository evidence or user-approved external environment
  can safely establish those irreversible or operator-owned behaviors here.
- 2026-08-27: Added an executable check for unchecked entries in
  `PRODUCTION_READINESS.md`; a clean image and completed historical plans are
  insufficient for a readiness pass while functional, security, hardware, or
  external-service evidence remains open.
- 2026-08-27: Reconciled active control-plane, contributor, security, rollback,
  and roadmap guidance with `rmcp`; historical ADR references to the superseded
  claw-code design remain intentionally marked as superseded history.

## 15. Outcomes & Retrospective

This plan is complete for its bounded source-level remediation. It does not
close the audit's unresolved production adapters. The readiness gate now fails
closed until real hardware, image boot, live Tor/WireGuard packet tests,
disposable-image DMS/panic tests, external wallet/VPS integration,
representative model performance, and every other checklist item have current
evidence.
