# SPEC-000 — Product Scope

- **Status:** active
- **Owner:** architect
- **Roadmap phase:** Phase 0
- **Linked ExecPlans:** EP-000, EP-001

## User-visible goal
A privacy-conscious developer can boot ADAD from USB on x86_64 hardware, use a
local-first AI coding agent (with optional cloud fallback over WireGuard), manage
XMR-funded private infrastructure, and shut down leaving zero trace on the host's
internal storage.

## Non-goals
- Local full Monero node; GPU/unified-memory inference; a GUI desktop; reliance
  on `torsocks`; any clearnet traffic; IPv6; KYC-required services.
- MAC impersonation (only randomization); per-push git identity rotation (only a
  stable pseudonym with stripped metadata).

## Terms
- **Amnesic state:** RAM-only (tmpfs) data destroyed on shutdown/panic.
- **Vault:** the LUKS2-encrypted persistent store (image in tests, partition on
  device in production).
- **Leak battery:** the E2E test suite asserting no clearnet/DNS/IPv6/discovery
  leaks and a fail-closed killswitch.

## Required behavior
- The system MUST run entirely from a RAM-only root; no writes to host internal
  storage at any point.
- The default AI provider MUST be local `llama-server`; cloud providers are
  opt-in fallbacks routed only over WireGuard.
- General network traffic MUST default to Tor; API traffic MUST use WireGuard.
- On loss of the protective tunnel/interface, the system MUST drop all traffic
  (fail-closed).
- Persistent secrets MUST live only in the vault and RAM, never on host disk.

## Inputs
Host hardware (x86_64, AVX, 16–64 GB RAM), a USB/NVMe-in-USB drive, an optional
vault passphrase, optional provider API keys (stored in vault).

## Outputs
A running amnesic environment; optionally provisioned VPS infra; code pushed to
privacy-respecting remotes with stripped metadata; zero host trace on shutdown.

## Error states
- Missing host tool at build → STOP (human installs).
- Missing required secret for a non-mockable operation → STOP.
- Any attempt to write host disk / real device from automation → forbidden.

## Data rules
No database. Persistence is the vault only. Amnesic/vault boundary is absolute.

## Security rules
See SECURITY.md. Leak-free posture and self-destruct (panic + DMS) are core.

## Accessibility rules
Keyboard-only operation for all core workflows; high-contrast TUI themes.

## Performance rules
Inference 4–50 tok/s by tier; killswitch DROP-ALL within target on interface
change; VPS provisioning < 2 min.

## Observability rules
RAM-only redacted logs; TUI status monitors for daemons.

## Required tests
Zero-host-write boot cycle; leak battery; provider-default (local) test;
keyboard-reachability test.

## Acceptance criteria
- [ ] Booted image performs a full use cycle with zero host-disk writes
      (QEMU monitored disk).
- [ ] Default provider is local; cloud requires explicit config.
- [ ] Leak battery passes.
- [ ] `scripts/verify.sh` exits 0 on the scaffolded workspace.
