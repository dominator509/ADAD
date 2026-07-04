---
id: EP-004
status: not-started
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
- [ ] contract, venice, egress_guard, agent_loop tests pass.
- [ ] wallet + vps mock tests pass; no real backend touched.
- [ ] Default provider is Local; Venice private-by-default proven.
- [ ] Only `agent-coding` imports vendored crates (Cargo.toml check).
- [ ] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
Mocks start/stop within tests; re-runs are clean. No network egress occurs in
tests (guard + mocks). A real transfer/provision request is a STOP, never
executed.

## 12. Progress
- [ ] M1 — client + contract test
- [ ] M2 — provider selection + Venice private-default
- [ ] M3 — egress guard
- [ ] M4 — control loop over vendored crates
- [ ] M5 — wallet + VPS mocks
- [ ] M6 — full verify
- [ ] verify + status set to complete

## 13. Surprises & Discoveries
(Record vendored-crate API specifics, Venice model-id handling, RPC/SSH shapes.)

## 14. Decision Log
(Record iteration-budget value, streaming approach, mock designs.)

## 15. Outcomes & Retrospective
(Filled at completion.)
