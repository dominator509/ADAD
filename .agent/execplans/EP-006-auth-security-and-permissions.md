---
id: EP-006
status: complete
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
- [x] Killswitch fail-closed + netlink-drop latency tests pass.
- [x] Routing posture test passes (no clearnet/IPv6/discovery; DNS via Tor).
- [x] MAC randomization test passes (randomized, not impersonation).
- [x] git-spoof proves stable pseudonym + stripped metadata (no rotation).
- [x] metafuse scrubbing test passes.
- [x] DMS wipes header on image at expiry; clock-freeze proven ineffective;
      panic path tested; no real device touched.
- [x] Egress guard backed by real leakguard state.
- [x] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
Firewall/routing apply idempotently (flush-then-set). Tests use images/network
namespaces, never real devices/interfaces where destructive. DMS/panic tests are
sandboxed. Re-runs are clean.

## 12. Progress
- [x] M1 — killswitch state machine
- [x] M2 — netlink monitor + latency
- [x] M3 — routing posture (Tor/WG/IPv6-off/no-discovery)
- [x] M4 — MAC randomization
- [x] M5 — git-spoof stable pseudonym
- [x] M6 — metafuse scrubbing
- [x] M7 — DMS (Tor-NTP) + panic + clock-freeze resistance
- [x] M8 — wire egress guard + leak battery
- [x] verify + status set to complete

## 13. Surprises & Discoveries
(Record netlink/fuser/nftables API specifics; Tor-NTP source; any posture edge
cases. Record failed hypotheses so later sessions don't repeat them.)
- M1: `leakguard` was still binary-only at plan start. A standard library
  surface was required so the killswitch/firewall state machine could be tested
  with `cargo test -p leakguard --lib`.
- M1: The in-memory model treats `Unknown` tunnel health as unsafe and collapses
  to DROP ALL, matching the fail-closed rule instead of guessing a permissive
  posture.
- M2: The netlink milestone is implemented as a simulated event boundary for
  now: `NetlinkEvent` maps interface/tunnel changes into the M1 killswitch
  state machine and measures reaction latency without opening a host netlink
  socket.
- M3: Routing is currently asserted as a pure posture model: general traffic
  routes to Tor, API traffic routes to WireGuard, DNS routes via Tor, IPv6 is
  disabled, and mDNS/SSDP/NetBIOS/direct-DNS classes are blocked by the
  firewall model.
- M4: MAC randomization is modeled from per-session entropy and interface name
  without changing any real host interface. The resulting address has the local
  bit set, multicast bit clear, changes across sessions, and is stable within a
  session.
- M5: `git-spoof` was still binary-only at plan start. A standard library
  surface plus rewrite model was required so stable pseudonym behavior could be
  tested without invoking real git.
- M6: `metafuse` was still binary-only at plan start. The first implementation
  adds a pure metadata presentation model instead of a host FUSE mount: it
  rewrites owner ids, replaces real timestamps with deterministic session
  scrubbed values, and removes EXIF tags before presentation.
- M7: DMS is implemented as an image-only state machine. `TorNtpTime` is the
  authority for expiry, local clock values are accepted only to prove they do
  not influence the deadline, and the only wipe target type is an in-memory
  `LuksHeaderImage`.
- M8: The client now consumes an `adad_core::EgressSnapshot` produced by
  leakguard's routing/firewall posture instead of importing leakguard directly
  into `agent-coding`. This preserves the cross-tool ownership boundary while
  giving the fallback guard real leakguard state.
- M8: The leak battery harness exists and passes at the model level. Because
  `build/adad.img` does not exist yet, on-image QEMU assertions are explicitly
  reported as pending EP-009/EP-010 work rather than silently claimed.

## 14. Decision Log
(Record firewall backend chosen (e.g. nftables), killswitch latency target,
Tor-NTP mechanism, sequencing of the on-image battery to EP-009/EP-010.)
- M1: Implemented only a pure firewall posture model, not host nftables. The
  posture defaults to DROP, permits Tor and optionally WireGuard when explicitly
  healthy, and never permits `Other` egress. Real firewall backend selection is
  deferred to M3.
- M1: `Killswitch::arm` requires Tor to be active. WireGuard may be inactive for
  Tor-only default posture, but `Unknown` is treated as unsafe and triggers
  DROP ALL. Interface-down, tunnel-loss, and ambiguous events all force DROP
  ALL.
- M2: Set the first killswitch reaction target to 250 ms in
  `DROP_ALL_TARGET_LATENCY`. The simulated drop tests assert both interface
  down and tunnel loss transition to DROP ALL within that target.
- M3: Real nftables/routing assets are deferred; this milestone adds the
  leak-free policy model and tests without touching host firewall/routing state.
  A posture that could permit direct DNS, IPv6, local discovery, or generic
  clearnet returns `Error::Killswitch`.
- M4: `mac::randomize(iface, SessionSeed)` returns a `MacAssignment` model. A
  future Linux/on-image backend can apply that assignment to the interface, but
  EP-006 M4 deliberately avoids host mutation in agent tests.
- M5: `git_spoof::rewrite` uses `adad_core::SessionIdentity` directly,
  normalizes both author and committer timestamps to `2000-01-01T00:00:00Z`,
  and proves the same persona identity is reused across commits rather than
  rotated.
- M6: Real `fuser` integration is deferred to the Linux/on-image backend. The
  tested contract lives in `metafuse::scrub_metadata`, so future `getattr` and
  xattr handling can call the same code without reimplementing privacy rules.
- M7: The panic path is tested through `panic_wipe`, which zeros modeled RAM
  secrets and wipes the image header. Real `kexec` integration remains a
  Linux/on-image backend concern; no path or block-device wipe API was added in
  EP-006.
- M8: Added `EgressSnapshot` to `adad-core` as a shared pure data contract. This
  is an extra file outside the initial list, justified because it avoids a
  direct `agent-coding` to `leakguard` dependency and keeps the architecture's
  tool-crate boundary intact.
- M8: `scripts/test-e2e.sh` and `scripts/verify.sh` now invoke the harness via
  `sh` when the file exists rather than requiring an executable bit. This keeps
  the E2E gate reliable on Windows/Git-Bash while preserving the same command
  names and success lines.
- M8: Full verify required one unsandboxed rerun because `cargo audit` needs to
  lock/update `C:\Users\domin\.cargo\advisory-db`, which is outside the
  workspace sandbox. The rerun completed with `verify: ok`.

## 15. Outcomes & Retrospective
- EP-006 completed the model-level security core: fail-closed killswitch,
  simulated netlink latency, Tor/WireGuard/no-leak routing posture, MAC
  randomization, stable git pseudonym rewriting, metafuse metadata scrubbing,
  DMS image-header wipe with local-clock-freeze resistance, panic wipe model,
  leakguard-backed fallback egress guard, and the first leak battery harness.
- No real block device, real LUKS vault, real network interface, real `kexec`,
  real Tor NTP query, or real firewall/routing table was touched. Destructive
  behavior is image-only in tests.
- Remaining risks: host nftables/netlink/FUSE/kexec/Tor-NTP backends are still
  Linux/on-image work; `build/adad.img` is not present yet, so the leak battery
  reports QEMU/on-image assertions as pending EP-009/EP-010 work; static-musl
  verification and smoke execution were skipped on `MSYS_NT-10.0-19045` and
  remain Linux-authoritative.
- `scripts/verify.sh` passed on 2026-07-03 with `verify: ok`.
