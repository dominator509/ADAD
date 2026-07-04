# SPEC-003 — Service Contracts (Agent Harness, Providers, Wallet, VPS)

- **Status:** active
- **Owner:** architect
- **Roadmap phase:** Phase 3
- **Linked ExecPlans:** EP-004

## User-visible goal
The agent-coding harness drives a tool-using loop against a local model by
default and cloud fallbacks on demand; wallet and VPS service methods work
against their (mocked) backends with stable contracts.

## Non-goals
No UI here (EP-005). No real inference weights, real wallet, or real VPS in
tests — mocks only. No inference HTTP calls outside `OpenAiCompatClient`.

## Terms
- **OpenAiCompatClient:** the single client whose `base_url`/`api_key`/`model`
  select provider; used for llama-server, OpenAI-compatible, and Venice.
- **Vendored crates:** claw-code MCP + tool-exec crates under `vendor/`, imported
  only by `agent-coding`.

## Required behavior
### Provider client
- `OpenAiCompatClient` MUST issue OpenAI-compatible `POST /v1/chat/completions`
  requests and parse the standard response/stream shape.
- Provider selection MUST default to `local`
  (`http://127.0.0.1:8080/v1`). `openai` and `venice` are opt-in and MUST be
  routed over WireGuard (the client MUST refuse to send a fallback request if
  the tunnel is not the active egress — coordinated with leakguard in EP-006).
- Venice MUST default to private model IDs; anonymized models MUST require
  `ADAD_VENICE_ALLOW_ANONYMIZED=true` and MUST emit a warning.

### Agent loop
- The ADAD control loop MUST use the vendored tool-exec engine to run tools in
  the workspace and the vendored MCP integration for MCP servers (stdio + HTTP/
  SSE), while owning planning/iteration itself (not claw-code's cloud-first loop).
- The loop MUST enforce a bounded tool-iteration budget and surface tool errors
  as typed results.

### Wallet service (`xmr-wallet-rs`)
- MUST expose methods (balance, address, transfer-prepare) via
  `monero-wallet-rpc` JSON-RPC over Tor. Real transfers are a STOP; tests use a
  mock RPC.

### VPS service (`vps-deploy-rs`)
- MUST expose a provision method that runs a setup script over SSH (over Tor).
  Real provisioning is a STOP; tests use a mock SSH target. Setup MUST complete
  (mock-timed) under 2 minutes.

## Inputs
Config (provider/base URL/model/keys), prompts, tool definitions, MCP server
configs, wallet RPC URL, VPS target + setup script.

## Outputs
Model completions (text/tool-calls); tool results; wallet query results; a
provisioned (mock) host handle.

## Error states
Provider unreachable → `Error::Provider`; tunnel-not-active on fallback →
`Error::EgressBlocked` (request NOT sent); RPC/SSH failure → typed errors;
anonymized-without-optin → `Error::Config`.

## Data rules
API keys loaded from vault into env; never logged. Model IDs validated against
the active provider.

## Security rules
No inference call bypasses `OpenAiCompatClient`; no fallback egress bypasses
WireGuard; keys redacted; Venice private-by-default.

## Accessibility rules
N/A (service layer).

## Performance rules
VPS setup < 2 min (mock-timed). Streaming responses handled incrementally.

## Observability rules
Log provider selected (not the key), token counts, tool-iteration count,
outcomes — all redacted.

## Required tests
- Contract: request shape matches OpenAI-compatible schema (one client, three
  base URLs) against a mock server.
- Venice: private default; anonymized requires opt-in + warning.
- Egress guard: fallback refused when tunnel inactive (mock leakguard state).
- Agent loop: drives a mock model through a tool call and returns the result;
  respects the iteration budget.
- Wallet/VPS: methods succeed against mocks; real ops are gated STOPs.

## Acceptance criteria
- [ ] Contract + Venice + egress-guard tests pass.
- [ ] Agent loop test passes against the mock inference server and mock tools.
- [ ] Wallet + VPS mock integration tests pass; no real backend touched.
- [ ] Only `agent-coding` imports the vendored crates (checked via Cargo.toml).
- [ ] `scripts/verify.sh` exits 0.
