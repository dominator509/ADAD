# SPEC-004 — UI/UX Behavior (TUI/CLI)

- **Status:** active
- **Owner:** architect
- **Roadmap phase:** Phase 4
- **Linked ExecPlans:** EP-005

## User-visible goal
Users operate ADAD entirely by keyboard through `ratatui` TUIs and CLIs for the
agent, wallet, VPS deployment, and a status monitor, with clear loading/empty/
error states and a high-contrast theme.

## Non-goals
No GUI desktop; no mouse-required paths; no business logic in the rendering
layer.

## Terms
- **Status monitor:** the one-screen TUI showing daemon states, killswitch, DMS
  countdown, and current provider/model.

## Required behavior
- Every core workflow (agent chat, wallet, VPS deploy, status) MUST be fully
  operable by keyboard alone.
- Each TUI MUST render distinct **loading**, **empty**, and **error** states; a
  network/provider error MUST show a redacted, actionable message (never a raw
  key or onion address).
- A **high-contrast** theme MUST be available and selectable.
- The agent TUI MUST stream model output incrementally and show the current
  provider/model.
- The status monitor MUST reflect real daemon state (Tor/WireGuard/llama-server/
  Monero/Git) and show killswitch + DMS state.

## Inputs
Keystrokes; config; live daemon status; model output stream.

## Outputs
Rendered TUI frames; issued commands to the service layer.

## Error states
Provider/tunnel/vault errors render as high-contrast banners with redacted text
and a suggested next action.

## Data rules
The UI reads typed results from the service layer; it does not parse raw network
data itself.

## Security rules
Escape control sequences in model/tool output (no terminal injection). Never
render secrets or full onion addresses.

## Accessibility rules
Keyboard-only; high-contrast theme; no color-only signaling (pair color with a
label/symbol); works with standard terminal accessibility tooling.

## Performance rules
UI stays responsive during streaming and daemon polling (no blocking the render
loop on I/O).

## Observability rules
UI actions log at debug (redacted); errors shown are also logged (redacted).

## Required tests
- Headless acceptance: drive each TUI via scripted key events; assert reachable
  actions and rendered loading/empty/error states.
- Injection: crafted model output with control sequences is escaped.
- Theme: high-contrast theme selectable and applied.
- No color-only: error state carries a text label, not just color.

## Acceptance criteria
- [ ] Keyboard-only acceptance tests pass for agent/wallet/VPS/status.
- [ ] Loading/empty/error states asserted for each TUI.
- [ ] Injection escape test passes.
- [ ] High-contrast theme test passes.
- [ ] `scripts/verify.sh` exits 0.
