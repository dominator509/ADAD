---
id: EP-004
status: complete
depends_on: [EP-003]
verify: scripts/verify.sh
---

# EP-004 — Service Layer (Agent Harness, Providers, Wallet, VPS)

## 1. Purpose / Big Picture
Build the agent-coding harness (ADAD-owned local-first control loop wrapping the
vendored MCP + tool-exec crates), the single `OpenAiCompatClient` with
llama-server default and OpenAI/Venice fallback, and the wallet + VPS service
surfaces — all tested against mocks. No real inference weights, wallet, or VPS.

## 2. Scope
- `OpenAiCompatClient` (one client; base_url/api_key/model select provider).
- Provider selection defaulting to local; Venice private-by-default.
- ADAD control loop using vendored tool-exec + MCP.
- `xmr-wallet-rs` (JSON-RPC over Tor, mocked) + `vps-deploy-rs` (SSH over Tor,
  mocked).

## 3. Non-goals
- No TUI (EP-005). No real egress (mocks). No killswitch enforcement yet (EP-006)
  — but the client MUST expose the egress-guard hook so EP-006 can enforce it.

## 4. Context and Orientation
SPEC-003 governs. ARCHITECTURE.md: only `agent-coding` imports vendored crates;
inference only via `OpenAiCompatClient`; wallet/VPS only via their crates. Venice
facts + private/anonymized distinction per DECISIONS ADR-004.

## 5. Files to Read First
- SPEC-003, SPEC-006, ARCHITECTURE.md (integration boundaries), ENVIRONMENT.md
  (provider vars), DECISIONS.md (ADR-003, ADR-004).

## 6. Files to Change
- `crates/agent-coding/src/{client.rs,loop.rs,provider_select.rs}`
- `crates/agent-coding/tests/{contract.rs,venice.rs,egress_guard.rs,agent_loop.rs}`
- `crates/xmr-wallet/src/rpc.rs` + `tests/wallet_mock.rs`
- `crates/vps-deploy/src/provision.rs` + `tests/vps_mock.rs`
- test mocks: `crates/agent-coding/tests/support/mock_inference.rs`,
  `crates/xmr-wallet/tests/support/mock_rpc.rs`,
  `crates/vps-deploy/tests/support/mock_ssh.rs`

## 7. Interfaces and Contracts
- `OpenAiCompatClient::new(base_url, api_key, model)`;
  `chat(messages) -> Result<Completion, Error>` (OpenAI-compatible shape).
- `provider_select(config) -> (base_url, api_key, model)`; default `Local`.
- Egress guard: `client` consults an `EgressState` trait; a fallback request when
  the tunnel is not active returns `Error::EgressBlocked` (request NOT sent).
- Venice: anonymized model requires `ADAD_VENICE_ALLOW_ANONYMIZED=true`, else
  `Error::Config`; a warning is logged when enabled.
- Wallet: `balance()/address()/prepare_transfer()` via JSON-RPC (mocked).
- VPS: `provision(target, setup_script) -> Result<Handle, Error>` over SSH
  (mocked); mock-timed under 2 min.

## 8. Milestones

### M1 — OpenAiCompatClient + contract test
- Goal: one client issues OpenAI-compatible requests; parses responses/stream.
- Files: `client.rs`, `tests/contract.rs`, `support/mock_inference.rs`.
- Validation: `cargo test -p agent-coding --test contract`
- Expected: request shape matches schema against the mock for all three base URLs.
- Recovery: confirm the exact JSON shape from mock; do not guess field names.

### M2 — Provider selection + Venice private-default
- Goal: default local; Venice private-by-default; anonymized opt-in + warning.
- Files: `provider_select.rs`, `tests/venice.rs`.
- Validation: `cargo test -p agent-coding --test venice`
- Expected: default provider Local; anonymized-without-optin → `Error::Config`;
  opt-in logs a warning.
- Recovery: fix selection logic; keep private-by-default.

### M3 — Egress guard
- Goal: fallback refused when tunnel inactive.
- Files: `client.rs` (EgressState hook), `tests/egress_guard.rs`.
- Validation: `cargo test -p agent-coding --test egress_guard`
- Expected: fallback with inactive tunnel → `Error::EgressBlocked`, no request
  sent; local provider unaffected.
- Recovery: ensure the guard is checked BEFORE any socket write.

### M4 — Control loop over vendored tool-exec + MCP
- Goal: ADAD loop drives a mock model through a tool call; bounded iterations.
- Files: `loop.rs`, `tests/agent_loop.rs`.
- Validation: `cargo test -p agent-coding --test agent_loop`
- Expected: loop runs a tool via vendored engine, returns the result, respects
  the iteration budget; only agent-coding imports the vendored crates.
- Recovery: confirm vendored API by reading the crate source; no invented calls.

### M5 — Wallet + VPS mock services
- Goal: wallet + VPS methods work against mocks; real ops are STOPs.
- Files: `xmr-wallet/src/rpc.rs`, `vps-deploy/src/provision.rs`, mock tests.
- Validation: `cargo test -p xmr-wallet && cargo test -p vps-deploy`
- Expected: mock RPC/SSH pass; provision mock-timed < 2 min; no real backend.
- Recovery: confirm monero-wallet-rpc JSON-RPC + SSH shapes; mock only.

### M6 — Full verify
- Validation: `scripts/verify.sh`
- Expected: `verify: ok`
- Recovery: first failing gate; bounded retry.

## 9. Concrete Steps
1. Implement the mock inference server; build `OpenAiCompatClient`; contract test.
2. Implement provider selection + Venice rules; test.
3. Add the egress-guard hook + test.
4. Wrap vendored tool-exec/MCP in the ADAD loop; test against the mock model.
5. Implement wallet RPC + VPS provision against mocks; test.
6. Run full verify.

## 10. Validation and Acceptance
- [x] contract, venice, egress_guard, agent_loop tests pass.
- [x] wallet + vps mock tests pass; no real backend touched.
- [x] Default provider is Local; Venice private-by-default proven.
- [x] Only `agent-coding` imports vendored crates (Cargo.toml check).
- [x] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
Mocks start/stop within tests; re-runs are clean. No network egress occurs in
tests (guard + mocks). A real transfer/provision request is a STOP, never
executed.

## 12. Progress
- [x] M1 — client + contract test
- [x] M2 — provider selection + Venice private-default
- [x] M3 — egress guard
- [x] M4 — control loop over vendored crates
- [x] M5 — wallet + VPS mocks
- [x] M6 — full verify
- [x] verify + status set to complete

## 13. Surprises & Discoveries
- M1 mock request parsing had to skip the HTTP request line before reading
  `Content-Length`; the first contract test run compiled but returned
  `Error::Provider` until the mock parser was narrowed.
- M2 found no pinned Venice private model ID in `ENVIRONMENT.md` or
  `DECISIONS.md`. Provider selection therefore requires an explicit Venice model
  and enforces private-by-default by rejecting model IDs containing
  `anonymized` unless `venice_allow_anonymized` is true.
- M3 loopback mocks need to simulate fallback routing, so `OpenAiCompatClient`
  carries an explicit `EgressMode`. Production defaults infer non-loopback URLs
  as fallback, while tests can force fallback mode against a loopback mock and
  prove no request was sent.
- M4 followed ADR-009 / EP-002 rather than the stale EP-004 wording about
  vendored claw-code crates. The control loop runs over the official-MCP
  `agent-coding` boundary and ADAD-owned `ExecutionRegistry`/tool executor
  traits; the test asserts only `agent-coding` imports `rmcp`.
- M5 found no pinned in-repo Monero wallet RPC reference beyond SPEC-003, so the
  JSON-RPC envelope and `get_balance`/`get_address`/non-relayed `transfer`
  fields were checked against the Monero wallet RPC docs before implementing
  mock-only tests.
- M6 full verify passed on this Windows/Git-Bash host. As documented by the
  scripts, static-musl verification and smoke execution were skipped here
  (`MSYS_NT-10.0-19045`); Linux remains authoritative for those checks.

## 14. Decision Log
- M1: Added `serde_json = "1"` as a direct `agent-coding` dependency because
  `OpenAiCompatClient` owns the OpenAI-compatible JSON request/response
  contract. The crate was already present transitively through `rmcp`, so this
  records ownership without expanding the resolved dependency graph.
- M1: Implemented the first client over `std::net::TcpStream` and loopback test
  mocks instead of adding an HTTP/TLS stack. EP-004 tests are mock-only, and this
  keeps static-musl risk low while the egress guard and provider policy mature.
- M2: `provider_select` returns a `ProviderSelection` struct rather than a bare
  tuple so warnings can travel with the selected base URL, API key, and model.
  Venice anonymized opt-in returns a `VeniceAnonymizedModelEnabled` warning
  marker for the caller to log/render without adding a logging dependency here.
- M3: Added the `EgressState` hook as a trait plus a static implementation for
  tests. The guard runs after URL parsing but before `TcpStream::connect`, so an
  inactive fallback tunnel returns `Error::EgressBlocked` without a socket write.
- M4: Set the first ADAD control-loop budget behavior to return
  `AgentLoopStatus::IterationBudgetExhausted` as a typed outcome instead of
  treating a bounded stop as a provider failure. Unknown tool names still fail
  closed with `Error::Provider`.
- M5: Added direct `serde`/`serde_json` dependencies to `xmr-wallet` because the
  crate now owns its wallet JSON-RPC envelope and response parsing. No new
  packages were pulled into `Cargo.lock`; `scripts/dependency-audit.sh` remained
  green after the direct dependency ownership change.
- M5: Wallet and VPS real I/O remain abstract traits only
  (`WalletRpcTransport`, `SshSession`). EP-004 introduces no concrete HTTP or
  SSH backend, keeping real transfer/provision operations as future STOP-gated
  work.
- M5: Files outside the initial EP-004 list were required for standard Rust
  crate wiring: `Cargo.lock`, `crates/xmr-wallet/Cargo.toml`,
  `crates/xmr-wallet/src/lib.rs`, and `crates/vps-deploy/src/lib.rs`.
- M6: Ran `scripts/verify.sh` with host-cache access because cargo-audit needs
  the user cargo advisory database outside the workspace sandbox.

## 15. Outcomes & Retrospective
- EP-004 completed the mock-only service layer: OpenAI-compatible client
  contract, provider selection, Venice privacy gate, egress guard hook, bounded
  ADAD-owned agent loop, wallet JSON-RPC mock, and VPS SSH mock all pass.
- No real inference provider, wallet RPC backend, SSH target, transfer, or VPS
  provision was contacted. Real backends remain STOP-gated future work.
- Remaining risks: the current `OpenAiCompatClient` is dependency-light and
  mock-oriented (`std::net` HTTP), so production-grade HTTPS/TLS fallback
  transport is still future work; EP-006 must wire the real leakguard egress
  state; EP-009/Linux CI remains authoritative for static-musl and smoke
  execution; the e2e leak battery is still skipped until its harness exists.
- `scripts/verify.sh` passed on 2026-07-03 with `verify: ok`.
