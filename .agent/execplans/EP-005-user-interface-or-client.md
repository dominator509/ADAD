---
id: EP-005
status: complete
depends_on: [EP-004]
verify: scripts/verify.sh
---

# EP-005 — User Interface (ratatui TUIs + CLIs)

## 1. Purpose / Big Picture
Build the keyboard-only `ratatui` TUIs and CLIs for the agent, wallet, VPS
deploy, and a status monitor, with distinct loading/empty/error states, a
high-contrast theme, terminal-injection-safe rendering, and headless acceptance
tests. This is the user-facing layer over the EP-004 services.

## 2. Scope
- Agent chat TUI (streaming, shows provider/model).
- Wallet TUI, VPS-deploy TUI, and the status monitor dashboard.
- High-contrast theme; keyboard-only navigation; safe output escaping.
- Headless acceptance tests driving scripted key events.

## 3. Non-goals
- No new business logic (lives in service crates). No mouse paths. No GUI. No
  killswitch/DMS internals (EP-006) — the status monitor only DISPLAYS them.

## 4. Context and Orientation
SPEC-004 governs. ARCHITECTURE.md: no business logic in the rendering layer; UI
reads typed results from services. Accessibility rules: keyboard-only,
high-contrast, no color-only signaling.

## 5. Files to Read First
- SPEC-004, SPEC-006 (error messages), OBSERVABILITY.md (status fields),
  ARCHITECTURE.md (request/command flow).

## 6. Files to Change
- `crates/agent-coding/src/tui/*` (agent chat view)
- `crates/xmr-wallet/src/tui/*`, `crates/vps-deploy/src/tui/*`
- a status-monitor view (in `agent-coding` or a small `adad-tui` module) reading
  daemon status
- `crates/*/tests/tui_acceptance.rs` (headless key-event tests)
- a shared theme module (high-contrast)

## 7. Interfaces and Contracts
- Each TUI exposes a headless driver: `run_headless(events) -> FrameLog` for
  tests.
- Views render `Loading | Empty | Ready | Error(redacted_msg)` states.
- All model/tool output is escaped before rendering (no control-seq injection).

## 8. Milestones

### M1 — Shared theme + TUI scaffold
- Goal: high-contrast theme + a headless render harness.
- Files: theme module, headless driver.
- Validation: `cargo test -p agent-coding --test tui_acceptance -- --list`
- Expected: acceptance test binary compiles; theme selectable.
- Recovery: fix ratatui backend wiring for headless rendering.

### M2 — Agent chat TUI
- Goal: streaming chat view; shows provider/model; loading/empty/error states.
- Files: `agent-coding/src/tui/*`, acceptance test.
- Validation: `cargo test -p agent-coding --test tui_acceptance`
- Expected: scripted keys reach send; states render; error is redacted.
- Recovery: ensure streaming does not block the render loop.

### M3 — Wallet + VPS TUIs
- Goal: keyboard-only wallet + VPS deploy views over the mock services.
- Files: wallet/vps `src/tui/*`, acceptance tests.
- Validation: `cargo test -p xmr-wallet --test tui_acceptance && cargo test -p vps-deploy --test tui_acceptance`
- Expected: all actions keyboard-reachable; states render.
- Recovery: add missing key bindings; keep logic in services.

### M4 — Status monitor + injection/no-color-only tests
- Goal: dashboard reflects daemon states (mocked), including "unknown"; escaping
  + no-color-only assertions.
- Files: status view, `tests/tui_acceptance.rs` additions.
- Validation: `cargo test -p agent-coding --test tui_acceptance`
- Expected: monitor reflects mocked states; crafted control-seq output escaped;
  error state carries a text label (not color alone).
- Recovery: fix escaping; pair color with labels/symbols.

### M5 — Full verify
- Validation: `scripts/verify.sh`
- Expected: `verify: ok`
- Recovery: first failing gate; bounded retry.

## 9. Concrete Steps
1. Build the high-contrast theme + headless driver.
2. Implement the agent chat TUI with streaming + states; test.
3. Implement wallet + VPS TUIs over mocks; test keyboard reachability.
4. Implement the status monitor; add injection + no-color-only tests.
5. Run full verify.

## 10. Validation and Acceptance
- [x] Keyboard-only acceptance tests pass for agent/wallet/VPS/status.
- [x] Loading/empty/error states asserted per TUI.
- [x] Injection-escape test passes.
- [x] High-contrast theme + no-color-only tests pass.
- [x] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
TUI tests are headless and deterministic (scripted events). Re-runs are clean.
No real daemons required — status is mocked in tests.

## 12. Progress
- [x] M1 — theme + scaffold
- [x] M2 — agent chat TUI
- [x] M3 — wallet + VPS TUIs
- [x] M4 — status monitor + injection/no-color tests
- [x] M5 — full verify
- [x] verify + status set to complete

## 13. Surprises & Discoveries
- M1: `ratatui` 0.30.x splits core/widgets and supports headless rendering via
  `ratatui::backend::TestBackend` plus `Terminal::draw`. The first
  `default-features = false, features = ["layout-cache"]` attempt linked
  `critical-section` without a Windows implementation and failed on MSVC with
  unresolved `_critical_section_1_0_acquire/release`.
- M2: Agent chat rendering can be tested without a live model by feeding
  scripted key, stream-delta, finish, and typed-error events into the headless
  driver. This proves streaming display behavior while leaving inference logic
  in the EP-004 service layer.
- M3: Wallet/VPS TUI drivers live inside their owning crates and take scripted
  events/results rather than importing test mocks from service tests. This keeps
  UI tests deterministic and avoids a real wallet RPC or SSH backend.
- M4: Status monitor testing is mocked at the status-snapshot boundary. It
  renders Tor/WireGuard/llama-server/Monero/Git/killswitch/DMS/provider/model
  values, including `unknown`, without polling real daemons in EP-005.
- M5: Full verify passed on this Windows/Git-Bash host. As documented by the
  scripts, static-musl verification and smoke execution were skipped here
  (`MSYS_NT-10.0-19045`); Linux remains authoritative for those checks.

## 14. Decision Log
- M1: Added `ratatui = { version = "0.30.0", default-features = false,
  features = ["std"] }` to keep the M1 scaffold on the supported current API
  without pulling a terminal backend or layout-cache critical-section path yet.
  `scripts/dependency-audit.sh` remained green after the new dependency graph.
- M1: High-contrast theme uses white on black, cyan accent, yellow loading, and
  light-red error. The initial `run_headless(events) -> FrameLog` records
  rendered frame snapshots and keyboard input while using `TestBackend` for the
  actual render pass.
- M2: Added `run_agent_chat_headless(events) -> AgentChatFrameLog` with default
  local provider/model display, Enter-to-send keyboard behavior, typed loading/
  empty/ready/error states, and redacted `adad_core::Error::user_message()`
  rendering.
- M3: Added direct `ratatui` dependencies to `xmr-wallet` and `vps-deploy`
  instead of importing `agent-coding`'s theme module, preserving the architecture
  rule that tool crates do not import each other. Wallet/VPS views duplicate the
  small high-contrast style constants for now; a dedicated shared UI crate can
  be considered later if duplication becomes material.
- M4: Added `escape_terminal_text` for model/status output before rendering.
  Control characters are rendered as visible escape text (for example `\x1b`,
  `\r`, `\n`) so crafted model output cannot inject terminal control sequences.
- M5: Files outside the initial EP-005 list were required for dependency wiring:
  `Cargo.lock`, `crates/agent-coding/Cargo.toml`,
  `crates/xmr-wallet/Cargo.toml`, and `crates/vps-deploy/Cargo.toml`.
- M5: Ran `scripts/verify.sh` with host-cache/network access because Cargo
  needed the new `ratatui` graph and cargo-audit needs the user advisory cache
  outside the workspace sandbox.

## 15. Outcomes & Retrospective
- EP-005 completed deterministic, headless TUI coverage for the agent chat,
  wallet, VPS deploy, and status monitor surfaces. Acceptance tests prove
  keyboard reachability, loading/empty/error states, high-contrast theme
  selection, terminal-control escaping, and text-labeled error states.
- No real provider, wallet RPC daemon, SSH target, status daemon, or host UI was
  contacted. Every UI test runs against scripted events and typed mock data.
- Remaining risks: status monitor polling is still mocked until EP-006/EP-008
  provide real daemon state plumbing; the high-contrast style constants are
  duplicated in wallet/VPS to preserve current crate boundaries; static-musl and
  smoke execution remain Linux-authoritative; the e2e leak battery remains
  skipped until its harness exists.
- `scripts/verify.sh` passed on 2026-07-03 with `verify: ok`.
