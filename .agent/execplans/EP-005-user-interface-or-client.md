---
id: EP-005
status: not-started
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
- [ ] Keyboard-only acceptance tests pass for agent/wallet/VPS/status.
- [ ] Loading/empty/error states asserted per TUI.
- [ ] Injection-escape test passes.
- [ ] High-contrast theme + no-color-only tests pass.
- [ ] `scripts/verify.sh` → `verify: ok`

## 11. Idempotence and Recovery
TUI tests are headless and deterministic (scripted events). Re-runs are clean.
No real daemons required — status is mocked in tests.

## 12. Progress
- [ ] M1 — theme + scaffold
- [ ] M2 — agent chat TUI
- [ ] M3 — wallet + VPS TUIs
- [ ] M4 — status monitor + injection/no-color tests
- [ ] M5 — full verify
- [ ] verify + status set to complete

## 13. Surprises & Discoveries
(Record ratatui headless-testing approach and any escaping edge cases.)

## 14. Decision Log
(Record theme palette, key-binding scheme, headless harness design.)

## 15. Outcomes & Retrospective
(Filled at completion.)
