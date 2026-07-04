---
id: EP-006
status: not-started
depends_on: [EP-005]
verify: scripts/verify.sh
---

# EP-006 — Auth, Security & Leak-Free Posture

## 1. Purpose / Big Picture
Implement the security baseline that replaces multi-user auth: LUKS2 unlock +
secret-in-memory handling; the leak-free networking posture (`leakguard-rs`
fail-closed killswitch, Tor-by-default, WireGuard split-tunnel, IPv6 off, no
DNS/discovery leaks); MAC randomization; Dead Man's Switch; panic wipe;
`metafuse-rs` metadata scrubbing; and `git-spoof-rs` stable-pseudonym
enforcement. This plan turns the reframed privacy goals into enforced, tested
behavior. It is the security core of ADAD.

## 2. Scope
- `leakguard-rs`: netlink interface monitor + fail-closed firewall; Tor-default
  routing; WireGuard split-tunnel; IPv6 disabled; DNS via Tor; block
  mDNS/SSDP/NetBIOS.
- MAC randomization (locally-administered random address, per session).
- `git-spoof-rs`: rewrite author/committer to the persona pseudonym, normalize
  timestamps (fixed UTC), strip real name/email/tz — no per-push rotation.
- `metafuse-rs`: randomized timestamps, stripped EXIF, fake UIDs on vault files.
- Dead Man's Switch (Tor-NTP-anchored) + panic wipe (kexec), image-only in tests.
- Wire the EP-004 egress guard to real leakguard state.

## 3. Non-goals
- No MAC impersonation (randomization only). No per-push git identity rotation.
- No real-device wipe (image/QEMU only). No user accounts/roles/tokens.

## 4. Context and Orientation
SPEC-005 + SPEC-006 govern. SECURITY.md threat model. AGENTS.md §12/§13: any
change opening a clearnet/IPv6/DNS path or weakening killswitch/DMS is a STOP;
never wipe a real device. The leak battery (tests/e2e) is the authoritative gate.

## 5. Files to Read First
- SPEC-005, SPEC-006, SECURITY.md, OBSERVABILITY.md (redaction), ARCHITECTURE.md
  (security boundaries), DECISIONS.md (ADR-004,005,006).

## 6. Files to Change
- `crates/leakguard/src/{killswitch.rs,netlink.rs,firewall.rs,routing.rs,mac.rs}`
- `crates/git-spoof/src/rewrite.rs`, `crates/metafuse/src/fuse.rs`
- `crates/leakguard/src/dms.rs` (+ panic hook)
- `crates/agent-coding/src/client.rs` (wire real EgressState)
- `tests/e2e/run-leak-battery.sh` + `tests/e2e/*` assertions (create)
- `crates/*/tests/*` for unit/integration of each mechanism

## 7. Interfaces and Contracts
- `Killswitch::arm()` installs a default-drop posture allowing only Tor +
  (when up) WireGuard; `on_interface_change()` (netlink) → DROP ALL on
  drop/tunnel loss within target latency.
- `mac::randomize(iface)` sets a locally-administered random MAC (U/L bit set,
  multicast bit clear); changes per session.
- `git_spoof::rewrite(commit, identity)` sets author+committer name/email to the
  pseudonym and timestamp to fixed UTC; strips real fields; identity is stable.
- `metafuse` presents scrubbed timestamps/EXIF/UIDs on vault-backed files.
- `Dms::new(window, tor_ntp)`; expiry wipes the LUKS header (image in tests);
  local clock freeze does NOT extend the window.
- `EgressState` implemented by leakguard; the EP-004 client consults it.

## 8. Milestones

### M1 — Killswitch state machine (unit)
- Goal: fail-closed transitions; default-drop; allow Tor/WireGuard only.
- Files: `killswitch.rs`, `firewall.rs`, unit tests.
- Validation: `cargo test -p leakguard --lib`
- Expected: state transitions prove DROP ALL on interface/tunnel loss.
- Recovery: model transitions explicitly; fail-closed on any ambiguity.

### M2 — Netlink monitor + latency
- Goal: react to interface drop via netlink within target latency.
- Files: `netlink.rs`, integration test (simulated drop).
- Validation: `cargo test -p leakguard --test netlink_drop`
- Expected: DROP ALL within target on simulated drop.
- Recovery: confirm netlink socket usage by reading crate docs; no guessed API.

### M3 — Routing posture: Tor default, WG split, IPv6 off, no discovery
- Goal: routing/firewall config enforces the posture.
- Files: `routing.rs`, `firewall.rs`, config assets.
- Validation: `cargo test -p leakguard --test routing`
- Expected: general→Tor, API→WG, IPv6 disabled, mDNS/SSDP/NetBIOS blocked, DNS
  via Tor (asserted on a config/model level here; on-image in the battery).
- Recovery: fix rules; a rule that could allow clearnet is a STOP.

### M4 — MAC randomization
- Goal: per-session locally-administered random MAC (not impersonation).
- Files: `mac.rs`, test.
- Validation: `cargo test -p leakguard --test mac`
- Expected: address has U/L bit set, multicast clear, differs across sessions;
  NOT a real-vendor OUI blend.
- Recovery: fix bit-setting; keep randomization semantics.

### M5 — git-spoof stable pseudonym
- Goal: commits carry the pseudonym + normalized timestamp; real metadata
  stripped; identity stable (not rotated).
- Files: `git-spoof/src/rewrite.rs`, test.
- Validation: `cargo test -p git-spoof`
- Expected: author/email = pseudonym; fixed-UTC timestamp; no real fields;
  two commits share the identity (proving no rotation).
- Recovery: read identity from persona (adad-core type); do not mint new ids.

### M6 — metafuse scrubbing
- Goal: scrubbed timestamps/EXIF/UIDs on vault files.
- Files: `metafuse/src/fuse.rs`, test.
- Validation: `cargo test -p metafuse`
- Expected: presented metadata randomized/stripped; real values not exposed.
- Recovery: fix the FUSE getattr/xattr handling; confirm fuser API by reading it.

### M7 — DMS (Tor-NTP) + panic + clock-freeze resistance
- Goal: DMS wipes the LUKS header on expiry using Tor NTP; local clock freeze
  does not extend the window; panic wipes RAM. Image-only.
- Files: `dms.rs` (+ panic hook), tests (against an image, never a real device).
- Validation: `cargo test -p leakguard --test dms`
- Expected: expiry → header wipe on the image; frozen local clock → DMS still
  fires on Tor-NTP schedule; panic path invoked in a sandboxed test.
- Recovery / **STOP**: never target a real device; if a test could, halt with a
  blocker. Weakening the DMS is a STOP.

### M8 — Wire egress guard + full leak battery
- Goal: agent-coding client consults real leakguard EgressState; the E2E leak
  battery exists and passes against a booted image (or is marked pending image
  from EP-009 — see recovery).
- Files: `agent-coding/src/client.rs`, `tests/e2e/run-leak-battery.sh`, e2e
  assertions.
- Validation: `scripts/verify.sh` (runs e2e if the harness+image exist)
- Expected: `verify: ok`; egress guard now backed by real state.
- Recovery: the on-image battery needs the EP-009 image; if not yet built, the
  battery harness is authored and unit-level posture tests pass now, with the
  on-image run gated to EP-009/EP-010. Record this sequencing in the Decision Log.

## 9. Concrete Steps
1. Build the killswitch state machine + firewall model; unit-test fail-closed.
2. Add netlink monitoring; test simulated drop latency.
3. Encode routing posture (Tor/WG/IPv6-off/no-discovery/DNS-via-Tor); test.
4. Implement MAC randomization; test bits + per-session change.
5. Implement git-spoof rewrite (stable pseudonym); test stability.
6. Implement metafuse scrubbing; test.
7. Implement DMS (Tor-NTP) + panic; test on an image; prove clock-freeze
   resistance.
8. Wire the real EgressState into the client; author the leak battery; run verify.

## 10. Validation and Acceptance
- [ ] Killswitch fail-closed + netlink-drop latency tests pass.
- [ ] Routing posture test passes (no clearnet/IPv6/discovery; DNS via Tor).
- [ ] MAC randomization test passes (randomized, not impersonation).
- [ ] git-spoof proves stable pseudonym + stripped metadata (no rotation).
- [ ] metafuse scrubbing test passes.
- [ ] DMS wipes header on image at expiry; clock-freeze proven ineffective;
      panic path tested; no real device touched.
- [ ] Egress guard backed by real leakguard state.
- [ ] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
Firewall/routing apply idempotently (flush-then-set). Tests use images/network
namespaces, never real devices/interfaces where destructive. DMS/panic tests are
sandboxed. Re-runs are clean.

## 12. Progress
- [ ] M1 — killswitch state machine
- [ ] M2 — netlink monitor + latency
- [ ] M3 — routing posture (Tor/WG/IPv6-off/no-discovery)
- [ ] M4 — MAC randomization
- [ ] M5 — git-spoof stable pseudonym
- [ ] M6 — metafuse scrubbing
- [ ] M7 — DMS (Tor-NTP) + panic + clock-freeze resistance
- [ ] M8 — wire egress guard + leak battery
- [ ] verify + status set to complete

## 13. Surprises & Discoveries
(Record netlink/fuser/nftables API specifics; Tor-NTP source; any posture edge
cases. Record failed hypotheses so later sessions don't repeat them.)

## 14. Decision Log
(Record firewall backend chosen (e.g. nftables), killswitch latency target,
Tor-NTP mechanism, sequencing of the on-image battery to EP-009/EP-010.)

## 15. Outcomes & Retrospective
(Filled at completion — the security posture is the project's crux; document
residual risks explicitly.)
