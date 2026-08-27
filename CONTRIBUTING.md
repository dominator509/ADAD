# CONTRIBUTING.md — ADAD (Humans and Coding Agents)

## Setup
See ENVIRONMENT.md. Install rustup + musl target + cargo-audit + host tools,
then `scripts/preflight.sh` and `scripts/install.sh`.

## Branch rules
- `main` stays releasable. Work on `ep-XXX-<slug>` branches tied to the active
  ExecPlan. No unrelated changes on a plan branch.

## Coding standards
- Rust 2021; `cargo fmt` clean; `clippy -D warnings` clean.
- Library modules hold testable logic; binaries are thin. No business logic in
  TUI/CLI rendering.
- `adad-core` stays I/O-free. Tool crates do not import each other. Only
  `agent-coding` imports the official `rmcp` SDK and owns MCP qualification,
  execution, and policy logic in-tree.
- Static-musl must keep building.

## Test requirements
- Every behavior change ships a test (TESTING.md). Egress-touching changes add a
  leak-battery assertion. Never weaken a leak/security test to pass a gate.

## Documentation requirements
- Update COMMANDS.md when commands change, ARCHITECTURE.md when boundaries
  change, DECISIONS.md for architecture decisions, ASSUMPTIONS.md when an
  assumption is confirmed/overturned, and the active ExecPlan as you work.

## Commit guidance
- Small, focused commits. Message: `EP-XXX: <what changed> (<why>)`.
- git-spoof enforces the stable pseudonym and strips real metadata; do not
  override it. Never commit secrets.

## Pull request checklist
- [ ] Scope matches one ExecPlan; no drift.
- [ ] `scripts/verify.sh` passes.
- [ ] `git diff --name-only` matches the ExecPlan's Files to Change (extras
      justified in the Decision Log).
- [ ] Docs updated; DECISIONS/ASSUMPTIONS reconciled.
- [ ] No secrets; no real device/remote/wallet operations.

## Code review checklist
- [ ] Boundaries and invariants (ARCHITECTURE.md) respected.
- [ ] No new egress path bypasses leakguard; no clearnet/IPv6/DNS leak.
- [ ] Tests meaningful; leak/security tests intact.
- [ ] Static-musl build intact; deps audited.

## Agent-specific contribution rules
- Obey AGENTS.md and EXECUTION_RULES.md. One session, one ExecPlan. Continue by
  default; STOP only for STOP conditions. Write `.agent/state/last-result.env`
  as the final file operation every session.
