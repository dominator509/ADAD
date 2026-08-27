# SECURITY.md — ADAD

## Security goals
Zero host forensic footprint; absolute network leak prevention (no clearnet, no
DNS leak, no IPv6, no local-discovery chatter); secrets confined to RAM and the
LUKS2 vault; believable *randomized* (not impersonated) device identity;
reliable self-destruct (panic wipe + Dead Man's Switch); pseudonymous,
metadata-stripped code publishing.

## Threat model summary
- **Adversary A — host forensics:** someone imaging the host disk after use.
  Mitigation: tmpfs RAM-only root; no host-disk writes; RAM scrub on shutdown;
  panic wipe.
- **Adversary B — network observer:** local network / ISP / exit-node observer
  correlating traffic. Mitigation: Tor-by-default; WireGuard split-tunnel to an
  XMR-paid VPS for APIs; fail-closed killswitch; IPv6 off; DNS only via Tor.
- **Adversary C — passive fingerprinter:** correlating device or commit
  metadata. Mitigation: MAC randomization; metafuse timestamp/EXIF/UID
  scrubbing; stable pseudonymous git identity with real metadata stripped.
- **Adversary D — seizure while unattended:** Mitigation: Dead Man's Switch
  auto-wipes the LUKS header if the vault is not accessed within the window,
  using Tor NTP to resist local clock-freeze bypass.
- **Out of scope:** defeating a network operator's access controls on networks
  the user does not own (explicitly NOT a goal — see PROJECT_BRIEF non-goals).

## Authentication rules
- Vault unlock: LUKS2 with Argon2id KDF. Passphrase entered at runtime, never
  stored. No passphrase in logs, env, or history.
- API keys: stored in the vault, loaded into process env at runtime, redacted
  everywhere.

## Authorization rules
- Single local user; no multi-tenant authz. The security boundary is the vault
  passphrase and the network posture, not per-user roles. See SPEC-005.

## Input validation rules
- Validate at every trust boundary: CLI args, vault config, network responses,
  MCP tool outputs. Reject malformed input with a typed error; never pass
  unvalidated network data into pure logic.

## Output encoding rules
- TUI/CLI output escapes control sequences from model/tool output (no terminal-
  injection via crafted responses).

## Secret management rules
- Secrets live only in the vault at rest and in RAM at runtime.
- Never commit: API keys, WireGuard private keys, XMR seeds/keys, LUKS headers,
  persona private material.
- `scripts/security-check.sh` greps for committed secrets and fails the build.

## Dependency security rules
- `cargo audit` runs in every verify. A known-vuln or yanked crate fails.
- The active MCP dependency is the official `rmcp` SDK; dependency changes are
  reviewed, locked in `Cargo.lock`, and recorded in an ADR or ExecPlan.

## Logging redaction rules
- Structured logs redact: passphrases, API keys, wallet addresses/keys,
  WireGuard keys, real identity fields, and full onion addresses. See
  OBSERVABILITY.md. Logs are RAM-only and wiped on shutdown.

## Data protection rules
- At rest: LUKS2/Argon2id. In memory: scrub key material on lock/shutdown/panic
  (zeroize sensitive buffers).
- No swap (constraint) — prevents secret spill to disk.

## Production data rules
- No agent session operates on a real device, real remote, real wallet, or real
  VPS. Those are STOP conditions (AGENTS.md §13).

## Safe migration rules
- No DB migrations exist. Vault-layout changes bump an `adad-core` config
  version and ship an explicit, tested in-place upgrade path with a downgrade
  note. Never destructive without a backup of the vault image in tests.

## API security rules
- All API egress is forced through WireGuard by `leakguard-rs`; a request that
  cannot use the tunnel is dropped, not sent in clear.
- Venice: private models by default; anonymized opt-in emits a warning.
- Bearer keys sent only over the tunnel; never logged.

## CSRF/CORS/session rules
- Not applicable (no web server surface exposed to third parties). The local
  `llama-server` binds to loopback only.

## Rate limiting
- Not a multi-user service. Client-side backoff on provider 429s; no inbound
  rate limiting needed.

## File upload rules
- `metafuse-rs` scrubs metadata (timestamps, EXIF, UIDs) on the user's own vault
  files. No third-party upload surface exists.

## Security checklist
- [ ] No committed secrets (security-check passes).
- [ ] IPv6 disabled; no clearnet DNS config present.
- [ ] All egress routes through Tor or WireGuard; killswitch fail-closed.
- [ ] Venice defaults to private models.
- [ ] Secrets zeroized on lock/shutdown/panic.
- [ ] Dependency audit clean.
- [ ] Leak battery passes against the image.

## STOP conditions for security-sensitive actions
- Any change that could create a clearnet/IPv6/DNS egress path.
- Any real-device wipe, real VPS provisioning, or real XMR spend.
- Any weakening of the killswitch, DMS, or redaction rules.
Record a blocker and set the ExecPlan `status: blocked` (AGENTS.md §4).
