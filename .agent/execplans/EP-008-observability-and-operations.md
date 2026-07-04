---
id: EP-008
status: complete
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
- [x] Redaction test passes; logs proven RAM-only.
- [x] Status monitor reflects real (mocked) daemon states incl. Unknown.
- [x] Alerts render for all four conditions with text labels.
- [x] Runbooks authored.
- [x] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
Logging/health are stateless per call; tests deterministic. Re-runs clean.

## 12. Progress
- [x] M1 — redacting logger
- [x] M2 — health checks + status wiring
- [x] M3 — alerts
- [x] M4 — runbooks + full verify
- [x] verify + status set to complete

## 13. Surprises & Discoveries
(Record journald-in-tmpfs specifics and any redaction edge cases.)
- M1: Implemented logging as a pure in-memory sink in `adad-core`, not as host
  journald I/O. This gives every crate the same redaction boundary now while
  preserving the RAM-only invariant; Linux image-level journald/tmpfs wiring
  remains EP-009/EP-010 work.
- M2: EP-005's status monitor was already headless and snapshot-driven. EP-008
  adds the health-check boundary as a pure `DaemonProbe`; failed health queries
  become `Unknown` immediately so the TUI never preserves a stale `ready`.
- M3: The status view did not yet model vault-lock imminence, so
  `StatusSnapshot` gained `vault_lock_minutes_remaining` in addition to the DMS
  countdown. Alert labels are rendered as visible text, not color-only state.
- M4: There was no `docs/` tree yet. EP-008 created `docs/runbooks/` with the
  four required runbooks from the template.
- M4: Full verify first stopped on formatting and then on a test struct mismatch
  after `StatusSnapshot` gained `vault_lock_minutes_remaining`. Both were
  test-shape issues; after `cargo fmt --all` and correcting the `HealthReport`
  initializer, `scripts/verify.sh` completed with `verify: ok`.

## 14. Decision Log
(Record log field set, health-check cadence, alert thresholds.)
- M1: Structured log lines carry `ts`, `level`, `component`, `event`,
  `outcome`, and typed fields. Fields must be marked `Public` or `Sensitive`;
  sensitive values render as `[REDACTED]` at the emit boundary.
- M2: Health polling cadence is modeled as one `check_all` call per status
  refresh. Real daemon-specific probes remain a Linux/on-image integration
  concern, while the status monitor now consumes `HealthReport` directly.
- M3: Alert thresholds are DMS near expiry at `<= 2` hours remaining and vault
  lock imminent at `<= 15` minutes remaining. Killswitch and tunnel alerts map
  directly from `Down` health states.
- M4: Authored runbooks for killswitch, DMS/panic, vault backup/restore, and
  VPS provisioning. Each runbook includes preconditions, procedure,
  verification, rollback, and escalation, with real-device/real-remote STOP
  boundaries preserved.
- M4: Ran `scripts/verify.sh` with host-cache access because `cargo audit`
  needs to lock/update `C:\Users\domin\.cargo\advisory-db` outside the
  workspace sandbox. The final run passed.

## 15. Outcomes & Retrospective
- EP-008 completed the observability/operations layer at the current repo
  stage: shared redacting in-memory structured logs, health-check reports wired
  into the status monitor, alert banners for killswitch/tunnel/DMS/vault lock
  conditions, and required operational runbooks.
- Redaction and RAM-only behavior are proven at the pure in-memory logging
  boundary. The booted image must still prove journald/tmpfs behavior once
  EP-009 creates `build/adad.img`.
- Status monitor health checks are real code paths with mocked probes in tests;
  daemon-specific Linux/Tor/WireGuard/Monero/Git query implementations remain
  on-image/backend work.
- Remaining risks: static-musl execution smoke remains Linux-authoritative on
  this Windows/Git-Bash host; QEMU/on-image leak and RAM-only log absence remain
  pending until EP-009/EP-010; production-readiness has not run yet.
- `scripts/verify.sh` passed on 2026-07-04 with `verify: ok`.
