# DECISIONS.md — Architecture Decision Log

## Decision table

| ADR | Decision | Status | Date | Owner |
|---|---|---|---|---|
| ADR-001 | Build over a hardened Debian-Live base rather than a from-scratch OS. | accepted | 2026-07-03 | architect |
| ADR-002 | Vendor only claw-code's MCP + tool-exec crates at a pinned commit; ADAD owns the control loop. | superseded by ADR-009 | 2026-07-03 | architect |
| ADR-003 | Single `OpenAiCompatClient`; local `llama-server` is the default provider; OpenAI-compatible and Venice are fallbacks over WireGuard. | accepted | 2026-07-03 | architect |
| ADR-004 | Venice fallback defaults to *private* models; anonymized models require explicit opt-in with a warning. | accepted | 2026-07-03 | security |
| ADR-005 | Git identity = one stable pseudonym + stripped real metadata. No per-push rotation. | accepted | 2026-07-03 | architect |
| ADR-006 | MAC handling = randomization (locally-administered random address), not believable-OUI impersonation. | accepted | 2026-07-03 | security |
| ADR-007 | All core tools are static musl binaries; no dynamic linking. | accepted | 2026-07-03 | architect |
| ADR-008 | No database/ORM/migrations; persistence is the LUKS2 vault only. | accepted | 2026-07-03 | architect |
| ADR-010 | Use a repo-owned Docker image builder for live-build/QEMU validation when host tools are unavailable. | accepted | 2026-07-05 | architect |

## ADR index
- ADR-001 … ADR-010 above. Full entries live inline below as they are expanded;
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
re-pinned deliberately; only `agent-coding` may import them. **Status note:**
superseded by ADR-009 after EP-002 showed the current claw-code upstream no
longer provides the clean isolated seam this ADR assumed.

### ADR-009 — Official MCP Rust SDK plus ADAD-owned execution engine
**Context:** the current claw-code upstream couples MCP handling to a broader
runtime/control-loop stack, which breaks ADAD's requirement for a clean,
local-first, Rust-owned harness. The product goal remains Claude-Code-like
features, feel, and functionality, but not its implementation tangle.
**Decision:** use the official Model Context Protocol Rust SDK (`rmcp`) as the
protocol layer inside `agent-coding`, and build ADAD's tool execution,
qualification, workspace policy, and control loop in-tree. Preserve
Claude-Code-like MCP ergonomics where they improve UX, especially qualified tool
naming and a broad tool surface, but keep execution, policy, and provider flow
under ADAD ownership. **Alternatives:** vendor a larger claw-code runtime slice
(rejected: drags in cloud-first coupling); build raw MCP transport from scratch
(rejected: unnecessary protocol work). **Consequences:** ADAD gets a clean,
official MCP substrate with a smaller trust surface; feature parity becomes an
explicit product goal implemented incrementally in `agent-coding` rather than
inherited wholesale from third-party runtime code.

### ADR-004 — Venice private-by-default
**Context:** Venice offers "private" models (fully private) and "anonymized"
models (metadata stripped but forwarded to the real upstream provider, which
still sees content). **Decision:** default the Venice fallback to private model
IDs; anonymized models are opt-in and emit a warning. **Consequences:** the
privacy posture is not silently weakened by choosing a fallback; config carries
an explicit flag.

### ADR-010 — Containerized image builder
**Context:** EP-009 requires `live-build`, `squashfs-tools`, QEMU, and related
Linux image-build tools, but the Windows/Git-Bash host does not provide them
natively and automated sessions must not mutate host packages or write physical
devices. **Decision:** keep the image-build toolchain in the repo-owned
`adad-ep009-builder:local` Docker image and run live-build/QEMU validations
there. The container receives the mount capability live-build needs for chroot
`/proc` and `/dev/pts`, but no host block devices are bound. **Alternatives:**
install host packages in-session (rejected by AGENTS/COMMANDS); require a
preconfigured external Linux builder for every EP-009/EP-010 run (rejected as
slower and less reproducible). **Consequences:** image validation is repeatable
from this host without host package mutation, while physical imaging remains a
human-only release step.

## Rules for adding new decisions
- Any change to an architectural invariant (ARCHITECTURE.md) requires an ADR.
- Copy `.agent/templates/adr-template.md`, assign the next number, fill Context/
  Decision/Alternatives/Consequences, set Status + Date + Owner, and add a row
  to the decision table.
- Superseding an ADR: set the old one's status to `superseded by ADR-NNN` and
  reference it in the new one.
