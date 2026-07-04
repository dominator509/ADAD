# DECISIONS.md — Architecture Decision Log

## Decision table

| ADR | Decision | Status | Date | Owner |
|---|---|---|---|---|
| ADR-001 | Build over a hardened Debian-Live base rather than a from-scratch OS. | accepted | 2026-07-03 | architect |
| ADR-002 | Vendor only claw-code's MCP + tool-exec crates at a pinned commit; ADAD owns the control loop. | accepted | 2026-07-03 | architect |
| ADR-003 | Single `OpenAiCompatClient`; local `llama-server` is the default provider; OpenAI-compatible and Venice are fallbacks over WireGuard. | accepted | 2026-07-03 | architect |
| ADR-004 | Venice fallback defaults to *private* models; anonymized models require explicit opt-in with a warning. | accepted | 2026-07-03 | security |
| ADR-005 | Git identity = one stable pseudonym + stripped real metadata. No per-push rotation. | accepted | 2026-07-03 | architect |
| ADR-006 | MAC handling = randomization (locally-administered random address), not believable-OUI impersonation. | accepted | 2026-07-03 | security |
| ADR-007 | All core tools are static musl binaries; no dynamic linking. | accepted | 2026-07-03 | architect |
| ADR-008 | No database/ORM/migrations; persistence is the LUKS2 vault only. | accepted | 2026-07-03 | architect |

## ADR index
- ADR-001 … ADR-008 above. Full entries live inline below as they are expanded;
  new ADRs use `.agent/templates/adr-template.md`.

## Initial ADR entries (from assumptions)

### ADR-002 — Vendor MCP + tool-exec crates, own the loop
**Context:** claw-code is an API-key-first, cloud-first harness whose upstream
"which language is canonical" story is unstable, but its MCP integration and
tool-execution engine solve genuinely hard problems. **Decision:** vendor those
two crates at a pinned commit under `vendor/`, import them only from
`agent-coding`, and build the local-first control loop ourselves. **Alternatives:**
fork claw-code wholesale (inherits cloud-first flow + unstable upstream); build
MCP + tool-exec from scratch (months of work). **Consequences:** ADAD controls
the control flow and provider defaults; vendored crates are frozen and must be
re-pinned deliberately; only `agent-coding` may import them.

### ADR-004 — Venice private-by-default
**Context:** Venice offers "private" models (fully private) and "anonymized"
models (metadata stripped but forwarded to the real upstream provider, which
still sees content). **Decision:** default the Venice fallback to private model
IDs; anonymized models are opt-in and emit a warning. **Consequences:** the
privacy posture is not silently weakened by choosing a fallback; config carries
an explicit flag.

## Rules for adding new decisions
- Any change to an architectural invariant (ARCHITECTURE.md) requires an ADR.
- Copy `.agent/templates/adr-template.md`, assign the next number, fill Context/
  Decision/Alternatives/Consequences, set Status + Date + Owner, and add a row
  to the decision table.
- Superseding an ADR: set the old one's status to `superseded by ADR-NNN` and
  reference it in the new one.
