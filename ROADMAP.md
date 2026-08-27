# ROADMAP.md — ADAD (Strategic Only)

> **Do not implement directly from this file. Implementation must happen through
> an ExecPlan.** This roadmap sequences phases and points at specs and
> ExecPlans. It sets direction; the ExecPlans are the executable contracts.

Phases map to the project's own milestone plan (forge → networking → agent →
wallet/VPS → persona/metafuse/DMS → git-spoof/storage) and to the standard
discovery→production arc.

## Phase 0 — Repository discovery and foundation
- **Purpose:** Establish the workspace, toolchain, scripts, and CI baseline so
  every later phase has a stable, verifiable substrate.
- **Dependencies:** none.
- **Exit criteria:** `scripts/verify.sh` runs end-to-end on an empty-but-scaffolded
  workspace; all placeholder scripts replaced with real commands.
- **Specs:** SPEC-000. **ExecPlans:** EP-000, EP-001.

## Phase 1 — Core domain
- **Purpose:** Implement `adad-core` (types, config schema, error taxonomy) and
  the sterile-creation logic of `forge-rs` (Zero-Clock epoch) as pure, tested
  logic before any I/O-heavy tool leans on it.
- **Dependencies:** Phase 0.
- **Exit criteria:** core types + forge logic unit-tested; no infrastructure
  leakage into `adad-core`.
- **Specs:** SPEC-001. **ExecPlans:** EP-002.

## Phase 2 — Data and persistence
- **Purpose:** LUKS2 vault lifecycle (create/unlock/lock/seal) and the
  persona-identity persistence format; amnesic-vs-vault boundary enforced.
- **Dependencies:** Phase 1.
- **Exit criteria:** vault image can be created, unlocked, written, re-sealed in
  tests; no writes escape tmpfs/vault.
- **Specs:** SPEC-002. **ExecPlans:** EP-003.

## Phase 3 — API / service layer
- **Purpose:** The agent-coding harness: official `rmcp` MCP protocol support
  and ADAD-owned tool execution wrapped in the local-first control loop;
  `OpenAiCompatClient`
  (llama-server default; OpenAI/Venice fallback); `xmr-wallet-rs` and
  `vps-deploy-rs` service surfaces.
- **Dependencies:** Phase 2.
- **Exit criteria:** harness drives a tool loop against a mock inference server;
  provider switching works; wallet/VPS service methods contract-tested with
  mocks.
- **Specs:** SPEC-003. **ExecPlans:** EP-004.

## Phase 4 — UI / client layer
- **Purpose:** `ratatui` TUIs and CLIs for agent, wallet, VPS, and status
  monitors; keyboard-only workflows; high-contrast themes.
- **Dependencies:** Phase 3.
- **Exit criteria:** each TUI renders loading/empty/error states; acceptance
  tests drive keyboard flows headlessly.
- **Specs:** SPEC-004. **ExecPlans:** EP-005.

## Phase 5 — Auth, permissions, and security
- **Purpose:** LUKS2 (Argon2id) unlock; secret-in-memory handling; the leak-free
  networking posture (`leakguard-rs` killswitch, Tor-by-default, WireGuard
  split-tunnel, IPv6 off, no DNS/discovery leaks); MAC randomization; Dead Man's
  Switch; panic wipe; metafuse; git-spoof stable-pseudonym enforcement.
- **Dependencies:** Phase 4.
- **Exit criteria:** leak battery passes; killswitch fail-closed on interface
  drop; git pushes carry no real identity.
- **Specs:** SPEC-005, SPEC-006. **ExecPlans:** EP-006.

## Phase 6 — Testing hardening
- **Purpose:** Raise coverage; add regression and failure-mode tests
  (tunnel-drop, clock-freeze DMS bypass attempt, vault-version upgrade); flaky
  policy; CI gating.
- **Dependencies:** Phase 5.
- **Exit criteria:** critical flows regression-covered; CI runs the full suite.
- **Specs:** SPEC-006. **ExecPlans:** EP-007.

## Phase 7 — Observability and operations
- **Purpose:** RAM-only structured logging with redaction; TUI status monitors
  for Tor/WireGuard/LLM/Monero/Git daemons; health checks; runbooks.
- **Dependencies:** Phase 6.
- **Exit criteria:** logs redacted and RAM-only; status monitors reflect real
  daemon state; runbooks present.
- **Specs:** SPEC-007. **ExecPlans:** EP-008.

## Phase 8 — Deployment and release
- **Purpose:** The `live-build` recipe assembling the bootable image from the
  static binaries; reproducible build; USB imaging runbook; release checklist;
  rollback path.
- **Dependencies:** Phase 7.
- **Exit criteria:** image boots in QEMU; smoke + leak battery pass against it.
- **Specs:** SPEC-008. **ExecPlans:** EP-009.

## Phase 9 — Production readiness
- **Purpose:** Full verification, security/privacy/performance review, backup/
  restore (vault) verification, deployment dry run, rollback drill, docs review,
  launch gate.
- **Dependencies:** Phase 8.
- **Exit criteria:** `scripts/production-readiness-check.sh` exits 0; every
  ExecPlan `complete`; `scripts/loop.sh` prints `build: complete`.
- **Specs:** SPEC-008. **ExecPlans:** EP-010.

## Production readiness milestone
Reached only when Phase 9 exit criteria hold. See PRODUCTION_READINESS.md.
