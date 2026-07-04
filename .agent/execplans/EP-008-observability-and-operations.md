---
id: EP-008
status: not-started
depends_on: [EP-007]
verify: scripts/verify.sh
---

# EP-008 — Observability & Operations

## 1. Purpose / Big Picture
Add RAM-only structured logging with verified redaction, the TUI status monitors
reflecting real daemon state, health checks, alerts, and operational runbooks —
so the operator can see and safely run the system, with nothing leaking off-box.

## 2. Scope
- Structured RAM-only logging with redaction (SPEC-007).
- Health checks for Tor/WireGuard/llama-server/Monero/Git daemons.
- Status monitor wired to real health checks (display was built in EP-005).
- Alerts for killswitch/tunnel/DMS/vault events.
- Runbooks authored from the template.

## 3. Non-goals
- No external metrics/telemetry backend. No off-box log shipping. No tracing.

## 4. Context and Orientation
SPEC-007 + OBSERVABILITY.md govern. Logs are amnesic (tmpfs journald), wiped on
shutdown; redaction enforced at the logging boundary.

## 5. Files to Read First
- SPEC-007, OBSERVABILITY.md, OPERATIONS.md, the EP-005 status view,
  `.agent/templates/runbook-template.md`.

## 6. Files to Change
- a logging module (redacting) shared across crates
- health-check functions per daemon; wire the status monitor to them
- alert rendering hooks in the status view
- `docs/runbooks/*.md` authored from the template
- `crates/*/tests/{redaction,status,alerts}_*.rs`

## 7. Interfaces and Contracts
- `log_event(component, event, outcome, fields)` redacts secrets before emit.
- `health::check(daemon) -> Status{ Ok | Unknown | Down }`.
- Alerts render high-contrast banners with text labels (no color-only).

## 8. Milestones

### M1 — Redacting logger
- Goal: structured RAM-only logging that redacts secrets/identity/onion.
- Files: logging module, `redaction_*` test.
- Validation: `cargo test --workspace --test 'redaction_*'` (or per-crate)
- Expected: secret-bearing events emit nothing secret; RAM-only (no host file).
- Recovery: extend the redaction set; default-redact unknown-sensitive.

### M2 — Health checks + status wiring
- Goal: real health checks; status monitor reflects them, including Unknown.
- Files: health module, status view wiring, `status_*` test.
- Validation: `cargo test -p agent-coding --test status_accuracy`
- Expected: mocked daemon states reflected; query failure → Unknown (not stale).
- Recovery: fix the poll; never show a stale Ok.

### M3 — Alerts
- Goal: banners for killswitch fired / tunnel down / DMS near-expiry / vault
  lock imminent.
- Files: alert hooks, `alerts_*` test.
- Validation: `cargo test -p agent-coding --test alerts`
- Expected: each condition renders its banner with a text label.
- Recovery: pair color with label; ensure all four conditions covered.

### M4 — Runbooks + full verify
- Goal: author operational runbooks; run verify.
- Files: `docs/runbooks/{killswitch,dms-panic,vault-backup,provisioning}.md`.
- Validation: `scripts/verify.sh`
- Expected: `verify: ok`; runbooks present.
- Recovery: first failing gate; bounded retry.

## 9. Concrete Steps
1. Build the redacting logger; test redaction + RAM-only.
2. Implement health checks; wire the status monitor; test accuracy incl. Unknown.
3. Add alert banners; test all four conditions.
4. Author runbooks from the template; run verify.

## 10. Validation and Acceptance
- [ ] Redaction test passes; logs proven RAM-only.
- [ ] Status monitor reflects real (mocked) daemon states incl. Unknown.
- [ ] Alerts render for all four conditions with text labels.
- [ ] Runbooks authored.
- [ ] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
Logging/health are stateless per call; tests deterministic. Re-runs clean.

## 12. Progress
- [ ] M1 — redacting logger
- [ ] M2 — health checks + status wiring
- [ ] M3 — alerts
- [ ] M4 — runbooks + full verify
- [ ] verify + status set to complete

## 13. Surprises & Discoveries
(Record journald-in-tmpfs specifics and any redaction edge cases.)

## 14. Decision Log
(Record log field set, health-check cadence, alert thresholds.)

## 15. Outcomes & Retrospective
(Filled at completion.)
