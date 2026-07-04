# AGENTS.md — Control Plane for Coding Agents

This file governs every coding-agent session in this repository. Read it first,
every session. It outranks your own assumptions.

## 1. Mission

Build ADAD (Amnesic Decentralized AI Development Environment): a hardened
Debian-Live bootable OS plus a set of static Rust tools that provide a
footprint-free, leak-free, local-first AI coding environment. The build is
performed by an unattended loop of fresh agent sessions, one ExecPlan at a time,
from empty repository to production readiness.

You are one such session. Deliver correct, in-scope, validated work and hand
off cleanly.

## 2. Source-of-truth priority

When two instructions conflict, the higher item wins:

1. The current user instruction in your invocation prompt (if a human wrote it).
2. AGENTS.md (this file).
3. The active ExecPlan named in your prompt (its front matter and body).
4. Existing repository code and tests (ground truth over any prose).
5. ARCHITECTURE.md.
6. The relevant spec in `.agent/specs/`.
7. ROADMAP.md (strategic only — never implement directly from it).

If the ExecPlan contradicts the code, trust the code and record the discrepancy
in the ExecPlan's Surprises & Discoveries.

## 3. Required workflow

1. Read AGENTS.md.
2. Read COMMANDS.md.
3. Read the active ExecPlan.
4. Run `scripts/preflight.sh`.
5. Complete milestones in order.
6. Validate each milestone with its stated command and expected result.
7. Update the ExecPlan Progress and Decision Log after each milestone.
8. Continue autonomously.
9. Stop only under a STOP condition.

**Do not ask the user for next steps. Proceed autonomously through the active
ExecPlan unless a STOP condition applies.**

## 4. STOP conditions

Stop only when one of these holds. When you stop, record a blocker per
`.agent/LOOP.md`, set the ExecPlan `status: blocked`, and write `RESULT=stop`
(or `blocked`) to `.agent/state/last-result.env` before ending.

- A required secret, credential, paid service, or external account is missing
  (e.g. an XMR wallet seed, a VPS provider SSH key, an Anthropic/OpenAI/Venice
  API key needed to run a test that cannot be mocked).
- Any action that may destroy user, host, or production data (wiping a real
  disk/LUKS header, `dd` to a physical device, deleting a persistent vault).
- A legal, security, or financial judgment that the specs do not already
  resolve.
- A materially different user-visible behavior choice not resolved by a spec.
- Required tests cannot run after documented recovery attempts.
- A production deployment or an irreversible migration would be required without
  explicit human permission.

STOP conditions halt the LOOP, not just the session. The loop driver exits
nonzero and prints the blocker. Do not wait for a reply.

## 5. Anti-drift rules

- Implement only what the active ExecPlan's current milestone specifies.
- Do not perform broad refactors, renames, dependency swaps, file
  reorganizations, styling rewrites, or "while I'm here" cleanup.
- Every ExecPlan lists "Files to Change." Changing anything else requires a
  justification line in the Decision Log, or revert it.
- One session works on exactly one ExecPlan. Never jump plans.
- Never implement directly from ROADMAP.md.

## 6. Anti-hallucination rules

- Do not invent package APIs, function names, methods, CLI flags, routes,
  config keys, environment variable names, or file paths.
- Confirm every name by reading the repository (the crate source, `Cargo.toml`,
  `COMMANDS.md`, the vendored claw-code crates, `ENVIRONMENT.md`).
- Use only commands listed in COMMANDS.md. If one is missing, update COMMANDS.md
  with repository evidence first, then use it.
- Record every assumption in the ExecPlan Decision Log and, if it affects the
  whole build, in ASSUMPTIONS.md.
- For third-party surfaces (llama.cpp `llama-server`, Venice API, WireGuard,
  Tor control port, `monero-wallet-rpc`), confirm the exact request/flag shape
  against the pinned docs or a local `--help` before calling it. Do not guess
  endpoint paths or JSON field names.

## 7. Anti-fixation rules (bounded retry)

For any failing validation command:

1. First failure: read the exact error, identify the likely cause, make the
   smallest targeted fix.
2. Second same-root failure: create or run a narrower diagnostic to isolate the
   cause. No broad rewrites.
3. Third same-root failure: STOP that approach. Record all failed hypotheses in
   Surprises & Discoveries. If a simpler in-scope path exists, take it. If not,
   write a blocker per `.agent/LOOP.md` and set `status: blocked`.

Never patch blindly around the same error more than three times.

## 8. Dependency rules

Before adding any crate or system package:

1. Check existing dependencies (`Cargo.toml`, the vendored claw-code crates).
2. Check whether existing tools already provide the capability.
3. Add a dependency only if genuinely necessary.
4. Prefer `no_std`-friendly, static-musl-compatible, well-audited crates.
5. Record the decision (crate, version, why) in the Decision Log.
6. Update `Cargo.lock` and re-run `scripts/dependency-audit.sh`.

Never add a dependency that forces dynamic linking of a core tool (violates the
static-musl constraint).

## 9. File creation rules

- Create files only where the ExecPlan's "Files to Change" says, or where
  standard project structure requires (and record it).
- Never hand-edit files in `.agent/state/` except filling the `Resolution:`
  section of `.agent/state/blockers.md`.
- Never create files outside the repository root.
- Keep secrets out of the tree. Secrets live in the LUKS vault at runtime and in
  environment variables, never committed. See SECURITY.md.

## 10. Testing rules

- Every behavior change ships with a test (see TESTING.md).
- Run the milestone's validation command and confirm its exact expected result
  before ticking the Progress box.
- Do not weaken, skip, or delete a test to make a gate pass. If a test is wrong,
  fix the test deliberately and record why in the Decision Log.
- Leak-prevention behaviors (no clearnet, no DNS leak, no IPv6, killswitch)
  MUST be asserted by automated tests, never by manual inspection alone.

## 11. Documentation update rules

- When you change a command, update COMMANDS.md in the same session.
- When you change a boundary or invariant, update ARCHITECTURE.md.
- When you make an architecture decision, add an entry to DECISIONS.md.
- When you confirm or overturn an assumption, update ASSUMPTIONS.md.
- Keep the ExecPlan's Progress, Decision Log, and Surprises current within the
  session that changes them — not later.

## 12. Security rules

- Enforce the leak-free posture at all times: Tor-by-default, fail-closed
  killswitch, IPv6 disabled, no DNS leaks, no local-discovery chatter.
- Default the cloud API fallback to privacy-preserving settings: Venice
  *private* models only unless a config explicitly opts into anonymized ones.
- Redact secrets and identifiers from all logs (see OBSERVABILITY.md).
- Never commit API keys, wallet seeds, WireGuard private keys, or LUKS headers.
- Treat every network egress path as security-critical: a change that could
  route traffic outside Tor/WireGuard is a STOP-worthy security judgment unless
  the spec already authorizes it.

## 13. Production data rules

- Never operate on a real block device, real USB drive, or real LUKS vault in a
  test or build step. Use image files and loopback/QEMU only.
- Never push to a real remote repository, provision a real VPS, or spend real
  XMR from an automated session. Those are STOP conditions.
- Any `dd`, `mkfs`, `cryptsetup luksFormat`, `wipefs`, or `kexec` against a
  non-image target is forbidden in an agent session.

## 14. Definition of done (per ExecPlan)

A plan is done only when ALL of these hold:

- All acceptance criteria in the plan pass (command + expected result).
- The front-matter `verify` command exits 0.
- ExecPlan Progress checkboxes are all ticked.
- Front-matter `status` is set to `complete`.
- Final `git diff --name-only` matches "Files to Change" (extras justified).
- Remaining risks are documented in Outcomes & Retrospective.
- `.agent/state/last-result.env` is written with `RESULT=plan_complete`.

## 15. Final response requirements

Your final message every session must include:

- ExecPlan status (complete / in-progress / blocked).
- Changed files.
- Commands run and their results.
- Acceptance-criteria status.
- Decisions made and assumptions confirmed or changed.
- Remaining risks.
- Whether production-readiness criteria passed (if you ran that check).
- Confirmation that `.agent/state/last-result.env` was written.

## 16. Loop Session Contract

- You may be running as one iteration of an unattended loop. Assume no human is
  watching and no follow-up question will be answered.
- At session start: read `.agent/LOOP.md`, then `.agent/state/blockers.md`. If
  the active ExecPlan has a blocker with a filled Resolution section, apply the
  resolution first, mark the blocker RESOLVED, and continue.
- Work only on the ExecPlan named in your invocation prompt.
- Before ending the session, in this order: (1) update the ExecPlan Progress
  checkboxes and Decision Log, (2) update the ExecPlan front-matter status,
  (3) write `.agent/state/last-result.env` with the exact schema in
  `.agent/LOOP.md`. Never end a session without writing last-result.env.
- If you hit a STOP condition: write the blocker to `.agent/state/blockers.md`
  using the blocker template, set the ExecPlan status to blocked, write
  `RESULT=stop` in last-result.env, then end the session. Do not wait for a
  reply.
- Running out of time or context mid-plan is not a failure. Finish the current
  milestone if possible, validate it, record progress, write
  `RESULT=in_progress`, and end cleanly. The next session will resume.
