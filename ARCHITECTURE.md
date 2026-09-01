# ARCHITECTURE.md — ADAD

## Purpose
Define the concrete boundaries, dependency rules, and invariants that every
ExecPlan must respect. Rules here are enforceable, not aspirational. When a
rule says "must not," a test or a review step checks it.

## System overview
ADAD is two things shipped together:

1. **A hardened Debian-Live OS image** (the amnesic substrate) — tmpfs RAM-only
   root, MAC randomization, Tor-by-default networking, WireGuard split-tunnel
   for APIs, a fail-closed killswitch, LUKS2 persistent vault, panic wipe, and a
   Dead Man's Switch. Built with `live-build` over a Debian base; where the base
   already provides a capability (amnesia, MAC randomization, LUKS persistence),
   ADAD configures and hardens it rather than reimplementing it.

2. **A set of static-musl Rust tools** that run inside that OS:
   - `forge-rs` — sterile image/vault creation with a Zero-Clock randomized
     epoch (no host timestamp leakage).
   - `leakguard-rs` — stateful killswitch; a Linux `ip monitor link` adapter;
     fail-closed nftables firewall reaction; image-only LUKS2 DMS adapter.
   - `agent-coding-rs` — local-first AI coding harness (uses the official MCP
     Rust SDK; owns tool execution, the control loop, and the provider client).
   - `xmr-wallet-rs` — Monero wallet ops via `monero-wallet-rpc` over Tor.
   - `vps-deploy-rs` — SSH provisioner over Tor; deploys Forgejo hidden service.
   - `persona-rs` — session identity (the pseudonym used by git-spoof + metafuse).
   - `metafuse-rs` — Linux read-only FUSE metadata view over the user's own vault
     files: randomized timestamps, fake UIDs/GIDs, hidden extended attributes,
     and rejection of symlinks and special files. The existing pure policy is
     also available for non-Linux checks; embedded file payloads are not rewritten.
   - `git-spoof-rs` — git wrapper enforcing the stable pseudonymous identity and
     stripping real author/email/timezone metadata before any local commit.

## Repository map (intended)
```
/
  Cargo.toml                 # workspace root (EP-001)
  crates/
    forge/                   # forge-rs
    leakguard/               # leakguard-rs
    agent-coding/            # agent-coding-rs (control loop + provider client)
    xmr-wallet/              # xmr-wallet-rs
    vps-deploy/              # vps-deploy-rs
    persona/                 # persona-rs
    metafuse/                # metafuse-rs
    git-spoof/               # git-spoof-rs
    adad-core/               # shared types, config, error taxonomy (no I/O)
  live-build/                # Debian-Live recipe, hooks, package lists (EP-009)
  tests/
    os/                      # QEMU boot smoke harness
    e2e/                     # leak battery (no-clearnet/DNS/IPv6/killswitch)
  scripts/                   # lifecycle + loop scripts (present now)
  build/                     # artifacts: adad.img, *.pass markers (gitignored)
  .agent/                    # control plane (present now)
```

The release image package list explicitly carries the runtime tools used by
the vault, Git, Tor, WireGuard, filesystem, and leakguard paths. The on-image
leak battery verifies their presence before emitting any pass marker; the
builder image's toolchain packages are not substitutes for target-image
contents. Builder and target-image Debian resolution use the pinned
`20260801T000000Z` snapshot, and the image provenance records that input.

## Layer responsibilities
- **`adad-core`** — pure types, config schema, error taxonomy. NO network, NO
  filesystem, NO process spawning. Everything else may depend on it.
- **Tool crates** (`forge`, `leakguard`, ...) — own their domain and its I/O.
  Each is a binary crate plus a library module for its testable logic.
- **`agent-coding` MCP layer** — the only crate allowed to depend on official
  MCP protocol/runtime crates. Tool execution, naming, and control flow stay
  ADAD-owned in-tree. Configured MCP servers may use direct stdio or the
  official rmcp streamable-HTTP transport; remote HTTPS MCP requires the
  authoritative fallback egress state, while plain HTTP is restricted to
  loopback fixtures.
- **`live-build/`** — OS assembly. Consumes the release binaries; contains no
  Rust logic.

## Dependency rules (concrete)
- Any crate may import `adad-core`. `adad-core` must import NO other ADAD crate.
- Only `agent-coding` may depend on MCP protocol/runtime crates such as `rmcp`.
  No other crate may carry MCP transport/runtime dependencies.
- Tool crates must NOT import each other directly. Cross-tool interaction goes
  through `adad-core` types or through process/IPC boundaries defined in a spec.
  (Example: `git-spoof` reads the current identity from `persona` via a
  `adad-core` config type written to the vault, not by importing `persona`.)
- No crate may add a dependency that requires dynamic linking of the final
  binary. Static musl must keep working (`scripts/build.sh` asserts this).

## Import rules
- No `use` of a symbol you have not confirmed exists in the target crate.
- No wildcard re-exports across tool crates.
- External protocol dependencies must be pinned in `Cargo.toml`/`Cargo.lock`.
  ADAD-owned execution logic lives in-tree and is reviewed like first-party
  code.

## Runtime flow (agent coding, the core loop)
1. `agent-coding-rs` loads config from the vault (provider, base URL, model).
2. It selects a provider via one `OpenAiCompatClient`:
   - **local (default):** `http://127.0.0.1:8080/v1` (`llama-server`).
   - **openai-compatible fallback:** a configured base URL, over WireGuard.
   - **venice fallback:** `https://api.venice.ai/api/v1`, over WireGuard,
     **private models only** unless config explicitly opts into anonymized.
3. The control loop (ADAD-owned) plans/acts using an in-tree execution engine
   and an official MCP protocol layer, executing bounded workspace tools and
   explicitly configured MCP servers. MCP stdio uses a direct child process;
   remote streamable HTTP uses certificate-verified HTTPS and is denied until
   the authoritative fallback egress state reports the WireGuard path active.
4. All egress is subject to `leakguard-rs`: local traffic to `llama-server`
   stays on loopback; any API traffic is forced through the WireGuard interface;
   everything else is Tor or dropped.

## Data flow
- **Amnesic state** lives in tmpfs (RAM). It is destroyed on shutdown/panic and
  never written to host disk.
- **Reloadable state** lives in the LUKS2 vault: config, API keys, wallet files,
  the persona identity, and repositories. It is decrypted into memory at unlock
  and re-sealed on lock.
- The boundary is absolute: a component that needs persistence writes to the
  vault path; a component that must leave no trace writes only to tmpfs. No
  component writes to a host-mounted filesystem. Tests assert no writes land
  outside tmpfs/vault image.

The shipped `git-spoof commit` command is the bounded Git runtime boundary: it
commits already-staged local changes with persona-provided author and
committer fields and fixed UTC timestamps. Remote pushes and arbitrary Git
subcommands remain outside this binary.

## Request/command flow (CLI/TUI)
- User drives everything via `ratatui` TUIs and CLIs (keyboard-only).
- A command validates inputs in the tool crate's library module, performs the
  effect, and returns a typed result rendered by the TUI/CLI layer. No business
  logic lives in the rendering layer.

## State management rules
- Session identity is owned solely by `persona-rs` and persisted as an
  `adad-core` type in the vault. `git-spoof-rs` and `metafuse-rs` READ it; they
  never mint their own identity.
- The killswitch state machine in `leakguard-rs` is authoritative for network
  posture. No other crate opens a raw egress socket without going through the
  posture it enforces.

## Persistence boundaries
- Only the vault (LUKS2 image in tests; production-device handling remains a
  separately gated backend) is persistent. Everything else is amnesic.
- The current `forge-rs` image boundary accepts only regular image files: it
  rejects symlinks, directories, block devices, and other non-regular targets
  before invoking `truncate`, `losetup`, `cryptsetup`, or image backup logic.
  This prevents the image-only implementation from being mistaken for a safe
  production-device writer.
- No SQL database. No ORM. No migrations. Vault layout changes are versioned in
  an `adad-core` config-version field and handled explicitly.

## External integration boundaries
- `llama-server`, OpenAI-compatible endpoints, and Venice are reached ONLY
  through `OpenAiCompatClient`. No crate calls an HTTP client for inference
  directly.
- `monero-wallet-rpc` is reached ONLY through `xmr-wallet-rs`, over Tor.
- VPS SSH is reached ONLY through `vps-deploy-rs`, over Tor.
- Tor control and WireGuard config are managed by `leakguard-rs`; its shipped
  monitor observes Linux link events and loads a fixed drop-only nftables
  ruleset on down/deleted links. Other crates request posture, they do not
  reconfigure the tunnels.

## Security boundaries
- Trust boundary 1: host hardware ↔ ADAD. ADAD trusts nothing on the host disk
  and writes nothing to it.
- Trust boundary 2: ADAD ↔ network. All egress is Tor or WireGuard; clearnet is
  never trusted and never permitted.
- Trust boundary 3: amnesic RAM ↔ LUKS vault. Secrets cross into RAM only while
  unlocked and are scrubbed on lock/shutdown/panic.
- Every input crossing a boundary is validated in the receiving crate.

The DMS state machine has a concrete image-only adapter in `leakguard-rs`.
`leakguard dms evaluate-image` accepts a caller-supplied authoritative Tor-NTP
time and can wipe only a regular LUKS2 image file after expiry; it rejects
devices and symlinks and never consults the local clock. Automatic Tor-NTP
acquisition, production block-device destruction, and panic `kexec` remain
release-gated behaviors rather than being implied by the image adapter.

## Validation boundaries
- Validate at the trust boundary where untrusted data enters (CLI args, config
  file, network responses, MCP tool outputs). Do not re-validate deep in pure
  logic; pass already-typed values inward.

## Error handling boundaries
- Library modules return typed `Result<_, adad-core::Error>`. Binaries map
  errors to exit codes and redacted user messages. No secret or identifier
  appears in an error string (see SPEC-006).

## Observability boundaries
- Logs go to RAM-only journald and are wiped on shutdown. Structured fields
  only; secrets and identifiers redacted (see OBSERVABILITY.md). No log is
  written to host disk or transmitted off-box.

## Architectural invariants
1. No component writes to host internal storage. Ever.
2. No clearnet egress. No IPv6. No DNS leak. No local-discovery chatter.
3. All core binaries are static musl.
4. Only `agent-coding` depends on MCP protocol/runtime crates.
5. Inference goes only through `OpenAiCompatClient`; local is the default.
6. There is exactly one session identity, owned by `persona-rs`, stable per
   session (not rotated per commit).
7. The killswitch is fail-closed: on interface drop or tunnel loss, DROP ALL.

## Forbidden architecture moves
- Adding a database, ORM, or migration system.
- Making a cloud API the default inference path.
- Letting a tool crate import another tool crate.
- Adding a dependency that breaks static-musl builds.
- Introducing MAC impersonation or per-push identity rotation (out of scope; see
  PROJECT_BRIEF.md).
- Any egress path that bypasses `leakguard-rs`.

## How to add a new feature
1. Write or update a spec in `.agent/specs/`.
2. Add or extend an ExecPlan in `.agent/execplans/` (front matter + milestones).
3. Implement in the owning crate's library module, with tests, behind the
   existing boundaries.
4. If it touches egress, add a leak-battery assertion.
5. Update ARCHITECTURE.md only if a boundary changed; add a DECISIONS.md entry.

## How to add a new dependency
Follow AGENTS.md §8. Confirm static-musl compatibility, record the decision,
update `Cargo.lock`, re-run `scripts/dependency-audit.sh`.

## How to modify data schema
There is no DB schema. To change vault layout: bump the `adad-core` config
version, write an explicit in-place upgrade path in `forge-rs`/`persona-rs`,
add a test that upgrades an old-version vault image, and record an ADR.

## How to add a new integration
Define its boundary crate (or extend the correct existing one), route it through
Tor or WireGuard via `leakguard-rs`, confirm the exact external contract against
pinned docs, add integration tests with a mock, and record an ADR.

## Architecture review checklist
- [ ] No new host-disk write path introduced.
- [ ] No new egress path bypasses `leakguard-rs`; no clearnet/IPv6/DNS leak.
- [ ] Core binaries still build as static musl.
- [ ] No tool crate imports another tool crate; only `agent-coding` imports MCP
      protocol/runtime crates.
- [ ] Inference still flows through `OpenAiCompatClient`, local by default.
- [ ] Session identity still single, stable, owned by `persona-rs`.
- [ ] Killswitch remains fail-closed.
- [ ] Spec + ExecPlan + DECISIONS updated as required.
