# PROJECT_BRIEF.md — ADAD

## Project name
Amnesic Decentralized AI Development Environment (codename **ADAD**).

## Problem statement
Developers who need genuine privacy have no turnkey environment that combines
(a) a footprint-free amnesic OS, (b) a local-first AI coding harness, and
(c) leak-free networking — without leaving forensic traces on host hardware or
leaking identity across the network, financial, and code-hosting layers. Tails,
Whonix, and Qubes solve the OS substrate but ship no AI coding harness; existing
coding agents are cloud-API-first and leak metadata freely.

## Target users
Privacy-conscious developers, security researchers operating on systems they own
or are authorized to test, journalists and activists in high-surveillance
environments, and anyone who needs an AI coding environment that leaves zero
host trace and no network leaks.

## Primary user outcomes
- Boot ADAD on an x86_64 host from USB, use local (and optionally cloud-fallback)
  AI coding agents, and shut down leaving zero trace on the host's internal
  storage (RAM-only tmpfs root).
- Run local LLM inference by default via `llama-server` (llama.cpp, CPU/GGUF),
  and fall back to OpenAI-compatible or Venice.ai APIs over WireGuard only when
  explicitly chosen.
- Route all general traffic through Tor by default and all API traffic through a
  WireGuard tunnel to an XMR-paid VPS, with a fail-closed killswitch.
- Privately provision VPS infrastructure with Monero from the terminal.
- Protect persistent data in a LUKS2 vault; wipe on panic; auto-wipe via a Dead
  Man's Switch if not accessed within a configured window.
- Push code to privacy-respecting Git remotes with a **stable pseudonymous**
  identity and all real author/email/timezone metadata stripped.

## Business goals
- Infrastructure paid in XMR to keep funding private and decentralized.
- Prefer self-hosted / privacy-centric services over centralized platforms.
- Account for GGUF model licensing; avoid distributing "uncensored" fine-tunes
  in a way that creates takedown exposure.

## Technical goals
- Hardened **Debian-Live** base (built over, not from scratch) providing the
  amnesic tmpfs root, MAC randomization, Tor-by-default, LUKS persistence, and
  killswitch — reusing proven substrate rather than reimplementing it.
- Static musl Rust binaries for all core tools (no dynamic linking; avoids
  `torsocks`/`LD_PRELOAD` proxy issues).
- A local-first agent harness that **vendors two crates from `claw-code` at a
  pinned commit** — the MCP integration and the tool-execution engine — wrapped
  in an ADAD-owned local-first control loop. The harness talks to one
  `OpenAiCompatClient` whose base URL selects local `llama-server`, a generic
  OpenAI-compatible endpoint, or Venice.
- IPv6 disabled at the kernel level; no swap; no clearnet under any condition.

## Out-of-scope items
- Running a full Monero node locally.
- GPU or Apple-unified-memory inference (CPU only).
- A traditional GUI desktop environment.
- Relying on `torsocks` for proxying static binaries.
- Any clearnet traffic, any IPv6, any KYC-required service.
- **MAC address impersonation** to blend into networks the user does not own.
  ADAD does MAC **randomization** (a locally-administered random address, as
  Tails does) for the user's own privacy — not believable-OUI spoofing aimed at
  evading a network operator's controls.
- **Per-push Git identity rotation.** ADAD uses a single stable pseudonym with
  real metadata stripped — not engineered cross-repository unlinkability.

## Success metrics
- Zero host-disk writes across a boot/use/shutdown cycle (verified in QEMU with
  a monitored virtual disk).
- Leak battery passes: no clearnet, no DNS leak, no IPv6, no mDNS/SSDP/NetBIOS
  chatter; killswitch drops all traffic within the target latency on interface
  state change.
- Local inference throughput within the documented tok/s band per model tier.
- VPS provisioning completes automated setup under 2 minutes.
- Git pushes contain no real name, email, or local-timezone timestamp.

## Production readiness definition
See PRODUCTION_READINESS.md. In short: all ExecPlans `complete`,
`scripts/verify.sh` exits 0, `scripts/production-readiness-check.sh` exits 0
(which requires a bootable image and a passing leak battery against it), and all
operational docs/runbooks present.
