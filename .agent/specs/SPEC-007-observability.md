# SPEC-007 — Observability

- **Status:** active
- **Owner:** ops
- **Roadmap phase:** Phase 7
- **Linked ExecPlans:** EP-008

## User-visible goal
The operator can see the real state of every subsystem via TUI status monitors,
with structured RAM-only logs that redact all secrets and leave no host trace.

## Non-goals
No external metrics/telemetry backend; no off-box log shipping; no distributed
tracing.

## Terms
- **Status monitor:** the one-screen TUI dashboard (SPEC-004).

## Required behavior
- Logs MUST be structured, RAM-only (tmpfs journald), and wiped on shutdown/
  panic; never written to host disk or transmitted off-box.
- Every log line MUST carry `ts` (Tor/monotonic-anchored), `level`, `component`,
  `event`, `outcome`.
- Redaction MUST remove passphrases, API keys, WireGuard keys, XMR addresses/
  keys, real identity fields, and full onion addresses.
- The status monitor MUST show real state for Tor bootstrap, WireGuard,
  llama-server, Monero node sync, git daemon, killswitch, and DMS countdown.
- Alerts MUST render (high-contrast banners) for: killswitch fired, tunnel down,
  DMS window nearing expiry, vault lock imminent.

## Inputs
Subsystem status queries; log events from all crates.

## Outputs
Redacted structured logs (RAM); a live status dashboard; alert banners.

## Error states
If a status query fails, the monitor shows an explicit "unknown/unreachable"
state (never a stale "ok").

## Data rules
Logs contain only non-secret typed fields; correlation via an in-memory session
id, never persisted off-box.

## Security rules
Redaction enforced at the logging boundary; logs provably absent from host disk
after shutdown.

## Accessibility rules
High-contrast; alerts carry text labels, not color alone.

## Performance rules
Status polling MUST not block the render loop; logging is non-blocking.

## Observability rules
(Self) A test asserts secrets never appear in emitted logs.

## Required tests
- Redaction: feed secret-bearing events; assert none appear in output.
- RAM-only: assert no log file lands on host disk (monitored write test).
- Status accuracy: mock daemon states; assert the monitor reflects them,
  including "unknown" on query failure.
- Alerts: trigger killswitch/tunnel/DMS conditions; assert banners render.

## Acceptance criteria
- [ ] Redaction test passes.
- [ ] RAM-only test proves no host-disk log.
- [ ] Status-accuracy and alert tests pass.
- [ ] `scripts/verify.sh` exits 0.
