# OBSERVABILITY.md — ADAD

## Logging strategy
- RAM-only (journald in tmpfs). Wiped on shutdown/panic. Never written to host
  disk, never transmitted off-box.
- Structured (key=value / JSON), one event per line, with a level and a
  component field.

## Structured log fields
`ts` (monotonic/Tor-anchored, not host wall clock), `level`, `component`
(`leakguard`/`agent`/`wallet`/`vps`/`persona`/`metafuse`/`gitspoof`/`forge`),
`event`, `outcome`, and event-specific typed fields.

## Redaction rules
Always redact: passphrases, API keys, WireGuard keys, XMR addresses/keys, real
identity fields, full onion addresses (log a truncated hash if correlation is
needed). Redaction is applied at the logging boundary; a value that cannot be
proven non-secret is redacted by default.

## Metrics
Local, in-memory operational signals surfaced in the TUI (no external metrics
backend): Tor bootstrap %, WireGuard state, inference tok/s (current session),
Monero node sync state, git daemon status, killswitch state, DMS time-remaining.

## Traces
Not applicable (no distributed system). Per-session correlation is a single
in-memory session id, never persisted off-box.

## Health checks
As in OPERATIONS.md: Tor, WireGuard, llama-server, wallet RPC, git daemon. Each
exposes a local status query the TUI polls.

## Uptime checks
N/A (not a service). The "up" concept is per-boot session state shown in the TUI.

## Dashboards
The `ratatui` status monitor is the dashboard: one screen showing all daemon
states, killswitch, DMS countdown, and current provider/model.

## Alerts
Surfaced in the TUI as high-contrast banners: killswitch fired, tunnel down,
DMS window nearing expiry, vault lock imminent. No external alerting (would leak
off-box).

## Service-level indicators
- Killswitch reaction time on interface drop.
- Local inference throughput (tok/s) vs. tier band.
- VPS provisioning time (< 2 min target).

## Service-level objectives
Informal, single-user: killswitch DROP-ALL within target latency; no leak class
ever observed in the battery; DMS never misses its window.

## Debugging production issues
On-box only, in RAM, redacted logs. Because logs are amnesic, capture the TUI
state and redacted log excerpt during the session; nothing persists after
shutdown by design.

## Observability acceptance criteria
- [ ] Logs are RAM-only and confirmed absent from host disk after shutdown.
- [ ] Redaction verified: a test asserts secrets never appear in log output.
- [ ] TUI status monitor reflects real daemon state (integration-tested).
- [ ] Killswitch/DMS/tunnel alerts render correctly.
