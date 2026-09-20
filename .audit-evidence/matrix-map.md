# ADAD Production-Readiness Audit — Matrix Category Map (skeleton)

Status: SKELETON — no verdicts assigned. This file maps the 32 matrix
categories to the checks and evidence files that cover them. Verdict cells are
left BLANK on purpose; nothing here is a PASS/FAIL/NOT_RUNNABLE_ENV/N/A claim.
The final audit (`PRODUCTION_READINESS_AUDIT.md`) fills each verdict only from
executed evidence logs that exist under `.audit-evidence/`.

- Author: scribe (kanban task t_199e67ef), 2026-09-07
- Source spec: `adad-matrix.txt` (operator spec 2026-09-07), read from
  `[HERMES]/kanban/attachments/<task>/adad-matrix.txt`
  (md5 `fdc3a08a9cad97ce11489cd2b3f5cd7a`; byte-identical copies on
  t_92568f59, t_5c390ad3, and `/tmp/adad-matrix.txt`).
- Repo under audit: `/root/ADAD` (dominator509/ADAD, branch `main`,
  commit `03e865b` "Update MCP boundary and release verification").

## 1. Evidence layout (what each file is, who produces it)

| Evidence artifact | Produced by | Meaning |
|---|---|---|
| `.audit-evidence/matrix-map.md` | scribe (this task) | Category → check → evidence-file map; blank verdicts |
| `.audit-evidence/not-runnable-env.md` | scribe (this task) | OS-image, LUKS, VPS, XMR = NOT_RUNNABLE_ENV annotations with exact reasons |
| `.audit-evidence/baseline/<n>.log` | sprint (t_bf7d4ee4) | Per-command baseline evidence: exact command line, working dir, stdout+stderr, exit code |
| `.audit-evidence/baseline/<n>.reason` | sprint (t_bf7d4ee4) | Exact blocker when a command was unsafe/blocked and therefore NOT run |
| `.audit-evidence/final/<n>.log` | forge (t_bd3bc731) | Post-fix rerun of the full executable matrix, same per-command format |
| `.audit-evidence/fix-summary.md` | forge (t_bd3bc731) | Defects found, fix applied, rerun proof |
| `PRODUCTION_READINESS_AUDIT.md` (repo root) | sentinel (t_a9afcc5f) | Final verdict table over all 32 categories |

Convention note: `<n>` in `baseline/<n>.log` / `final/<n>.log` is the per-command
run index recorded by the sprint/forge tasks (each log embeds its exact command
line), NOT the matrix category number. A matrix category may be evidenced by
several command logs, and one command log may evidence several categories. The
final audit must cite the actual log path per executed command and must never
treat a category index as a log index.

Scope note: ADAD has no database (ADR-008: "No database/ORM/migrations;
persistence is the LUKS2 vault only") and is a local-first, single-tenant
tool with no web front end. Generic matrix bullets that assume a DB, a
multi-tenant SaaS, webhooks, containers, or a browser UI are product-N/A
territory; whether a category earns N/A (product non-goal) or FAIL (missing
evidence for an in-scope claim) is decided by the final audit from evidence,
never pre-assigned here.

## 2. The 32 categories (from adad-matrix.txt) — checks, evidence files, blank verdict

Legend for "Evidence checks that should cover it": repo gate / command names.
`R` = runnable in this environment (baseline+final logs expected);
`NR-OS`, `NR-LUKS`, `NR-VPS`, `NR-XMR` = evidence requires one of the
NOT_RUNNABLE_ENV areas detailed in `not-runnable-env.md`.

Verdict column is intentionally BLANK. Do not fill it from this map.

| # | Category (adad-matrix.txt) | Evidence checks that should cover it | Expected evidence file(s) | Verdict |
|---|---|---|---|---|
| 1 | REPOSITORY INTEGRITY & CLEAN-ROOM REPRODUCIBILITY | `R` `git status --porcelain` hygiene (no committed binaries/secrets/dumps/logs), `Cargo.lock` committed, locked resolve `cargo build --locked`; repo secret/IPv6 guard `scripts/security-check.sh`; prereq docs ENVIRONMENT.md vs host. | baseline/final logs for build+security-check; source review |  |
| 2 | STATIC CORRECTNESS | `R` `cargo check --workspace --all-targets --all-features` (scripts/typecheck.sh), `cargo clippy --workspace --all-targets --all-features -- -D warnings` (scripts/lint.sh), `cargo fmt --all --check` (scripts/format-check.sh). Dead-code/unused-dep/circular-dep via clippy+lockfile review. No DB schema/IaC/Dockerfile in product (ADR-008) — config-file parse validated under category 23. | baseline/final logs for typecheck/lint/format-check |  |
| 3 | UNIT TESTS | `R` `cargo test --workspace --locked`, `scripts/test-unit.sh` (`cargo test --workspace --lib`). Covers every crate: adad-core, agent-coding, forge, git-spoof, leakguard, metafuse, persona, vps-deploy, xmr-wallet. Zero-discovery / unexplained skips fail per matrix meta-rules — checked from logs. | baseline/final logs for cargo test (unit) |  |
| 4 | INTEGRATION TESTS | `R` `scripts/test-integration.sh` (`cargo test --workspace --tests`); crate tests/ dirs (e.g. leakguard egress_guard/failure_provider/failure_dms/vault_roundtrip) against mock servers + loopback LUKS fixtures (ENVIRONMENT.md: "Loopback LUKS images + QEMU + mock servers, all created at test time. No real devices or remotes"). "REAL migrations exercised" — product N/A (no DB). Booted-image integration (`boot-smoke`) requires OS-image. | baseline/final logs; not-runnable-env.md (OS-image rows) |  |
| 5 | CONTRACT TESTS | `R` cargo test contract suites: MCP/tool schema via `rmcp` protocol in agent-coding; OpenAI-compatible provider request/response contract tests; config string round-trip (adad-core config.rs). REST/OpenAPI, GraphQL, gRPC, webhooks: product N/A (no server surface). Backward-compat/version negotiation where implemented. | baseline/final logs for cargo test |  |
| 6 | END-TO-END TESTS (production-like artifact) | `NR-OS` E2E surface is the leak battery against a BOOTED image: `tests/e2e/run-leak-battery.sh`, `scripts/test-e2e.sh`; requires `build/adad.img` + QEMU. Source-level E2E assertion helpers exist (`tests/e2e/assert-leakguard-model.sh`, `assert-agent-egress-guard.sh`) but the production-like artifact gate is image-bound. | not-runnable-env.md (OS-image); baseline `.reason` if test-e2e blocked |  |
| 7 | ACCEPTANCE/REQUIREMENTS TRACEABILITY | `R` repo traceability review: SPEC-000..008, EP-000..013 with exit criteria, `.agent/checklists/*.md`; `scripts/verify.sh` as the aggregate gate. No single command proves traceability — source review + verify logs. | baseline/final logs for verify.sh if run; source review |  |
| 8 | REGRESSION TESTS | `R` regression suites executed inside `cargo test --workspace` (leakguard regression_security_flows.rs, killswitch/DMS clock-freeze/vault-upgrade regression tests per PRODUCTION_READINESS.md). Golden/snapshot checks where present. | baseline/final logs for cargo test |  |
| 9 | DATABASE TESTS | Product N/A (ADR-008, ARCHITECTURE forbids DB; no schema/migrations exist). Evidence = source review confirming absence; no DB command exists to run. | source review (absence scan) |  |
| 10 | SECURITY TESTS | `R` `scripts/security-check.sh` (committed-secret scan, IPv6/clearnet guards, delegates cargo audit), `scripts/dependency-audit.sh` (`cargo audit` — cargo-audit not installed on host at map time; install or `.reason`), clippy `-D warnings` SAST. Container scan / IaC scan: product N/A (no containers/IaC). Web-application attacks (XSS/CSRF/IDOR/SSRF…): product N/A for web bullets; auth-bypass/privesc/injection/fuzz-of-parsers assessed where adapters exist. | baseline/final logs for security-check + dependency-audit |  |
| 11 | PERMISSION/TENANT-ISOLATION | Product N/A for multi-tenant bullets (single-tenant local-first). In-scope: human-confirmation/authz on wallet/VPS/SSH actions (SPEC-005; agent-coding/forge authz tests). Evidence via cargo test logs + SPEC-005 review. | baseline/final logs for cargo test |  |
| 12 | FUZZ & PROPERTY TESTS | `R`-checkable if a harness exists (search for fuzz/property suites across crates). If none exists in source, the map records absence; the final audit decides FAIL (matrix: fuzzing of external parsers) — verdict not assigned here. | baseline/final logs (only if such tests ran); source absence note |  |
| 13 | CONCURRENCY/RACE | `R` concurrency-relevant suites inside `cargo test --workspace` (leakguard killswitch, vault lock/seal, agent session). Race/duplicate/lost-update bullets assessed against what the codebase implements. | baseline/final logs for cargo test |  |
| 14 | FAULT-INJECTION/RESILIENCE | `R` source-level failure tests (agent-coding failure_provider.rs, leakguard failure_dms.rs, killswitch fail-closed tests); `NR-OS` image-level fault injection (interface drop on booted image, disk full, reboot) requires OS-image. Retry-storm/circuit-breaker: assess where implemented. | baseline/final logs; not-runnable-env.md (OS-image rows) |  |
| 15 | PERFORMANCE | `R`-partial: `NR-OS` `scripts/min-system-sim.sh` requires `build/adad.img` (floor/target/comfort timings); pure-logic perf benchmarks inside cargo test if any. Image/runtime latency (killswitch latency, inference tok/s) is OS-image-bound. | not-runnable-env.md (OS-image); baseline `.reason` if min-system-sim blocked |  |
| 16 | LOAD/STRESS/SPIKE/SOAK | `NR-OS` requires a running system (booted image) to load/stress/soak; no source-level gate exists. | not-runnable-env.md (OS-image) |  |
| 17 | RESOURCE EXHAUSTION | `R`-partial source-level checks where implemented (alloc/error paths in unit tests); `NR-OS` image-level exhaustion (full disk, fd/conn exhaustion on live system) requires OS-image. | baseline/final logs; not-runnable-env.md (OS-image) |  |
| 18 | FRONTEND/UI (if applicable) | `R` ratatui TUIs with headless keyboard-drive acceptance tests per ROADMAP Phase 4/SPEC-004 (inside `cargo test --workspace`). Browser bullets (viewport/hydration/console errors…): product N/A (no web front end). | baseline/final logs for cargo test |  |
| 19 | ACCESSIBILITY | `R` keyboard-only reachability + high-contrast theme tests (SPEC-004 / PRODUCTION_READINESS.md) where implemented in cargo test. WCAG/browser a11y: product N/A (no web). | baseline/final logs for cargo test |  |
| 20 | COMPATIBILITY | `R` rust-toolchain.toml pins toolchain; static musl build (build.sh) for Linux targets; supported-OS/runtime bullets assessed vs product surface; no DB/browser/runtime matrix. | baseline/final logs for build |  |
| 21 | PACKAGING/INSTALL | `R` `scripts/install.sh` + `scripts/preflight.sh` for host tooling; `NR-OS` production package = the bootable Debian-Live image (`scripts/build-image*.sh`, `live-build/`) — requires real image target; Docker bullets product N/A. | baseline/final logs for preflight/install; not-runnable-env.md (OS-image) |  |
| 22 | DEPLOYMENT | `R` CI workflow review (`.github/workflows/ci.yml` fresh checkout, artifact promotion, env config, secret injection); `NR-OS` image promotion/boot; `NR-VPS` VPS provisioning/egress deployment (`crates/vps-deploy`, WireGuard split-tunnel) requires a real VPS and egress changes. | not-runnable-env.md (OS-image, VPS); CI workflow source review |  |
| 23 | CONFIGURATION | `R` adad-core config schema parse/validation + env-var fail-fast tests (ENVIRONMENT.md ADAD_* vars; config.rs tests inside `cargo test --workspace`); unknown-value/bad-value/redaction behavior asserted by tests. | baseline/final logs for cargo test |  |
| 24 | OBSERVABILITY | `R` source-level: RAM-only structured logging with redaction (leakguard/adad-core), TUI status monitors, alert rendering — asserted by tests where implemented. Runtime health/readiness/liveness probes are image-bound (OS-image). | baseline/final logs; not-runnable-env.md (OS-image) |  |
| 25 | OPERATIONAL RECOVERY | `R`-partial: vault backup/restore + vault-layout upgrade path at loop-image level (`forge` vault tests); `NR-OS` crash/host/service/reboot recovery, re-image rollback drill (`tests/os/rollback-drill.sh`, QEMU boot smoke) requires OS-image. Queue/DLQ/DR: product N/A (no queues/DB). | baseline/final logs; not-runnable-env.md (OS-image) |  |
| 26 | DATA INTEGRITY | `R` vault write/unlock/re-seal round-trips (forge vault_roundtrip.rs), config serialization round-trip (adad-core), idempotent retry paths in agent-coding; cross-service consistency assessed vs what exists. | baseline/final logs for cargo test |  |
| 27 | PRIVACY/COMPLIANCE | `R`-partial: metafuse metadata scrubbing, MAC randomization, secret zeroization asserted in unit tests; `NR-OS` amnesic zero-host-write / tmpfs RAM-only proof requires booted image; GDPR/CCPA/PII: product N/A (local tool, no PII service; export/delete/retention assessed at vault level). | baseline/final logs; not-runnable-env.md (OS-image) |  |
| 28 | DEPENDENCY/SUPPLY CHAIN | `R` `scripts/dependency-audit.sh` (`cargo audit` — tool install or `.reason`), Cargo.lock committed (reproducible), license compat + yanked-crate checks via audit; SBOM/provenance items where present (image provenance is OS-image-bound). | baseline/final logs for dependency-audit; not-runnable-env.md (OS-image) |  |
| 29 | DOCUMENTATION VERIFICATION | `R` source review: root docs (README, ARCHITECTURE, SECURITY, ENVIRONMENT, COMMANDS, HOW_TO_USE, PRODUCTION_READINESS, RELEASE, ROLLBACK, DECISIONS, ASSUMPTIONS, .agent/*) reconciled against implementation. No command; evidence is the audit's doc-vs-code cross-check. | source review (documented in final audit) |  |
| 30 | PRODUCTION ARTIFACT SMOKE (own hard gate) | `NR-OS` `scripts/production-readiness-check.sh` hard-gates on clean checkout + `build/adad.img` + provenance digest + on-image leak marker — real image target not available. `R`-partial: `scripts/smoke-test.sh` (every static musl binary `--version` runs) is runnable. | not-runnable-env.md (OS-image); baseline/final logs for smoke-test |  |
| 31 | RELEASE ROLLBACK TEST | `NR-OS` re-image rollback drill requires booted image (tests/os/rollback-drill*.sh); `R`-partial vault version-compat window regression is source-testable (vault-upgrade regression inside cargo test). | not-runnable-env.md (OS-image); baseline/final logs for cargo test |  |
| 32 | RELEASE-CANDIDATE CLEAN-ROOM (immediately pre-release) | `NR-OS`/`NR-VPS` aggregate: clean-room fresh checkout → image build → clean infra provision → migrations (N/A, no DB) → full suite → security scans → deploy → E2E → smoke → observability → backup/restore. Runnable pieces: cargo suite + scripts; image/deploy pieces require OS-image/VPS. | not-runnable-env.md (OS-image, VPS); baseline/final logs; fix-summary.md |  |

## 3. Project-area annotations (NOT_RUNNABLE_ENV) — cross-reference

The four ADAD functional areas below are NOT_RUNNABLE_ENV in this environment.
They are not numbered matrix categories themselves; they gate the matrix rows
flagged above with `NR-OS` / `NR-LUKS` / `NR-VPS` / `NR-XMR`. Exact reasons and
environment grounding are in `not-runnable-env.md`.

| Area | Repo surface | Gates matrix rows |
|---|---|---|
| OS-image | `scripts/build-image.sh`, `build-image-inside.sh`, `build-image-builder.sh`, `live-build/` recipe, `build/adad.img`, QEMU boot/leak/rollback harness (`tests/os/*`) | 4, 6, 14, 15, 16, 17, 21, 22, 24, 25, 27, 28, 30, 31, 32 |
| LUKS | LUKS2 vault lifecycle `crates/forge/src/vault.rs`, cryptsetup, DMS/panic wipe (`crates/leakguard/src/dms.rs`) | 10 (part), 25, 26 (production-vault bullets), 27 |
| VPS | `crates/vps-deploy`, WireGuard split-tunnel egress, XMR-paid VPS provisioning (PROJECT_BRIEF) | 22, 32 |
| XMR | `crates/xmr-wallet`, Monero RPC/transfers behind human confirmation | 3/4/5 (wallet contract tests are mock-only per EP-013), 10, 32 |

Note: source-level loopback/mock suites that exercise vault or wallet logic
inside `cargo test` are matrix rows 3/4/8/26 evidence and are logged by the
baseline/final runs; they do not constitute the production capability the
NOT_RUNNABLE_ENV areas stand for (see matrix meta-rules: no mocked success
masquerading as production).

## 4. Verification of this map against the source matrix

- adad-matrix.txt rows 1..32 are each listed exactly once above (titles verbatim).
- No verdict is assigned in this file; every verdict cell above is blank.
- Four non-runnable areas are annotated only in `not-runnable-env.md` as
  NOT_RUNNABLE_ENV with exact reasons; this file cross-references them.
