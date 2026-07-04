# SPEC-005 — Auth, Permissions & Security Baseline

- **Status:** active
- **Owner:** security
- **Roadmap phase:** Phase 5
- **Linked ExecPlans:** EP-006

## User-visible goal
The only authentication is the vault passphrase; the security "permissions" are
the leak-free network posture and the self-destruct mechanisms. Traditional
multi-user auth is **out of scope**; this spec defines the security baseline
that replaces it.

## Non-goals
No user accounts, roles, sessions, tokens, CSRF/CORS, or rate limiting (no
multi-tenant surface). No MAC impersonation; no per-push identity rotation.

## Terms
- **Killswitch:** `leakguard-rs` fail-closed firewall + netlink interface
  monitor.
- **DMS:** Dead Man's Switch — Tor-NTP-anchored timer that wipes the LUKS header
  if the vault is not accessed within the window.

## Required behavior
### Authentication
- Vault unlock MUST use LUKS2/Argon2id; passphrase never stored or logged.
- Provider API keys MUST load from the vault into env at runtime and be redacted
  everywhere.

### Network posture (the real "authorization")
- General traffic MUST default to Tor; API traffic MUST use WireGuard.
- IPv6 MUST be disabled at the kernel level; DNS MUST resolve only via Tor; no
  mDNS/SSDP/NetBIOS traffic MUST leave the host.
- The killswitch MUST be fail-closed: on interface drop or tunnel loss it MUST
  DROP ALL egress within the target latency, monitored via netlink.

### Identity & metadata
- MAC MUST be randomized (locally-administered random address) per session — NOT
  a believable-OUI impersonation.
- There MUST be exactly one stable `SessionIdentity` (persona-owned).
  `git-spoof-rs` MUST rewrite commit author/committer name+email to that
  pseudonym and normalize timestamps (fixed UTC), stripping real name/email/
  local-timezone — WITHOUT per-push rotation.
- `metafuse-rs` MUST present randomized timestamps, stripped EXIF, and fake UIDs
  on the user's own vault files.

### Self-destruct
- The panic button MUST wipe RAM (kexec) on demand.
- The DMS MUST use Tor NTP (not the local clock) so freezing the local clock
  does NOT extend the window; on expiry it MUST wipe the LUKS header.

## Inputs
Passphrase; interface state (netlink); Tor NTP time; vault-access events; commit
operations; file operations.

## Outputs
An unlocked vault; an enforced network posture; a randomized MAC; spoofed file
metadata; pseudonymous commits; timely self-destruct.

## Error states
Wrong passphrase → clean failure; tunnel loss → DROP ALL; clock-freeze attempt →
DMS still fires on Tor-NTP schedule; anonymized-Venice-without-optin → refused.

## Data rules
Identity is single and stable; secrets zeroized on lock/shutdown/panic.

## Security rules
All of SECURITY.md applies. Any change that could open a clearnet/IPv6/DNS path,
weaken the killswitch, or weaken the DMS is a STOP.

## Accessibility rules
Passphrase and all security prompts are keyboard-only, high-contrast.

## Performance rules
Killswitch DROP-ALL within target latency on interface change.

## Observability rules
Log posture changes, killswitch fires, DMS countdown, MAC randomization — all
redacted (no keys, no full onion, no real identity).

## Required tests
- Killswitch: simulate interface drop; assert DROP ALL within target.
- No-leak: assert no clearnet/DNS/IPv6/discovery in the battery.
- MAC: assert a randomized locally-administered address, changed per session.
- git-spoof: commit; assert author/email = pseudonym, timestamp normalized, no
  real metadata; assert identity is stable (not rotated) across pushes.
- DMS: simulate local clock freeze; assert DMS still fires on Tor-NTP schedule;
  assert header wipe on expiry (against an image, never a real device).
- metafuse: assert scrubbed timestamps/EXIF/UIDs on vault files.

## Acceptance criteria
- [ ] Killswitch, no-leak, MAC, git-spoof, DMS, metafuse tests all pass.
- [ ] git identity proven stable (not per-push).
- [ ] DMS clock-freeze bypass proven ineffective.
- [ ] No real device is ever wiped in tests (image-only).
- [ ] `scripts/verify.sh` and the leak battery pass.
