# How to Use This Blueprint Pack

This pack builds **ADAD** (Amnesic Decentralized AI Development Environment) —
a hardened Debian-Live bootable OS plus static Rust tools — via an unattended
loop of fresh coding-agent sessions, one ExecPlan at a time, from empty
repository to production readiness.

Architecture already decided and baked in: a hardened **Debian-Live base** built
over with `live-build`; an agent harness using the official Rust MCP SDK
(`rmcp`) with ADAD-owned execution and policy logic; a single
`OpenAiCompatClient` with **llama-server as the default** provider and
**OpenAI-compatible + Venice** fallbacks over **WireGuard**; full leak-free
posture (Tor-by-default, fail-closed killswitch, IPv6 off, no DNS/discovery
leaks); **MAC randomization** (not impersonation); and a **stable pseudonymous
git identity** (not per-push rotation).

The implementation is still experimental. Library contracts, models, mocks,
and a live-build recipe do not by themselves prove that the shipped
executables, Linux security adapters, external transports, or release image
provide those workflows. See `PRODUCTION_READINESS.md` for the outstanding
gates.

## 0. Run the fully unattended build

```sh
AGENT_CMD='codex --cd . --ask-for-approval never --sandbox workspace-write' \
  MAX_ITERATIONS=100 scripts/loop.sh
```

`AGENT_CMD` is any coding-agent CLI that accepts a single prompt string as its
final argument and can read files, edit files, and run terminal commands in the
repo. The loop runs until it prints `build: complete`, or halts on:

- a blocker or agent `stop`/`blocked` result → **exit 3**,
- three consecutive failed sessions on the same plan+milestone → **exit 4**,
- iteration budget exhausted → **exit 5**.

On any halt: run `scripts/loop-status.sh`, read `.agent/state/blockers.md`, fill
in the `Resolution:` section, set the blocked plan's front-matter `status` back
to `in-progress`, and rerun `scripts/loop.sh`. The loop resumes exactly where it
stopped.

If your agent CLI does not accept a prompt argument, paste the contents of
`.agent/prompts/loop-iteration.md` (with `[EXECPLAN_PATH]` filled from
`scripts/next-step.sh`) into any agent that can read files, edit files, and run
terminal commands — then apply the same result-file protocol manually (the
agent must write `.agent/state/last-result.env` at the end of every session).

## 1. Place files into the repository

Unzip the pack at the root of a fresh git repository so that `AGENTS.md`,
`COMMANDS.md`, `scripts/`, and `.agent/` sit at the top level. Commit the pack as
your first commit. `build/` and `target/` are gitignored.

## 2. Choose the active ExecPlan

Let the loop choose: `scripts/next-step.sh` prints the next runnable plan (or
`DONE`, or `BLOCKED:<path>`). To run one plan manually, pass its path to your
agent using `.agent/prompts/execute-active-execplan.md`. EP-000 ships
`complete` (greenfield), so the first real work is EP-001.

## 3. Run preflight

```sh
scripts/preflight.sh
```

Prints `preflight: ok` when the workspace and the active ExecPlan's front matter
are valid. Missing host tools (`cargo`, `lb`, `qemu`, ...) are surfaced here;
installing them is a human step (`scripts/install.sh`), never done mid-session.

## 4. Run a coding LLM against an ExecPlan

Use the generic prompt (below) or `.agent/prompts/execute-active-execplan.md`
with `[EXECPLAN_PATH]` set. The agent implements milestones in order, validates
each, updates the ExecPlan, and writes `.agent/state/last-result.env` before
ending.

## 5. Continue a partially completed plan

If a session ended `in-progress` (a normal outcome), the next invocation resumes
from the Progress checkboxes. Use `.agent/prompts/continue-execplan.md`. The loop
does this automatically.

## 6. Debug failing validation

Use `.agent/prompts/debug-validation-failure.md` with the failing command. It
enforces the bounded-retry rule: smallest fix → narrow diagnostic → after three
same-root failures, record failed hypotheses and take a simpler path or write a
blocker.

## 7. Perform final review

Use `.agent/prompts/final-review.md`. It runs full verification, compares
`git diff --name-only` to the ExecPlan's Files to Change, checks every acceptance
criterion, fills Outcomes & Retrospective, and sets `status: complete`.

## 8. Decide production readiness

The build is DONE only when all three hold: every ExecPlan `status: complete`,
`scripts/verify.sh` exits 0, and `scripts/production-readiness-check.sh` exits 0.
`scripts/loop.sh` prints `build: complete` only then. A final on-hardware boot
smoke (confirming the leak battery on a real machine) is a documented human step.

## 9. Avoid roadmap-only implementation

Never implement directly from `ROADMAP.md`. It is strategic. All implementation
happens through an ExecPlan. If a needed change has no ExecPlan, add or extend
one first.

## 10. Update plans as the repository evolves

When reality diverges from a plan, update the ExecPlan (Surprises & Discoveries,
Decision Log), and if a boundary/invariant changed, update ARCHITECTURE.md and
add a DECISIONS.md ADR. Keep front-matter `status` accurate — it is the single
source of truth the loop reads.

---

## Generic lower-tier coding LLM invocation prompt

```text
Read AGENTS.md, COMMANDS.md, .agent/PLANS.md, .agent/LOOP.md, and [EXECPLAN_PATH].
Implement [EXECPLAN_PATH] to completion.
Do not ask for next steps.
Do not implement from ROADMAP.md directly.
Do not broaden scope.
Complete milestones in order.
Validate after each milestone.
Update the ExecPlan as you work.
Use only commands from COMMANDS.md.
Stop only for STOP conditions in AGENTS.md.
At the end, run the required verification command, run git diff --name-only,
update Outcomes & Retrospective, update the ExecPlan front-matter status,
write .agent/state/last-result.env per .agent/LOOP.md as your final file write,
and report changed files, commands run, results, decisions, risks, and
acceptance status.
```

## Codex-style example

```sh
codex --cd . \
  --ask-for-approval never \
  --sandbox workspace-write \
  "Read AGENTS.md, COMMANDS.md, .agent/PLANS.md, .agent/LOOP.md, and .agent/execplans/EP-001-foundation.md. Implement EP-001-foundation.md to completion. Do not ask for next steps. Stop only for STOP conditions in AGENTS.md. Update the ExecPlan as you work. Run validation after each milestone. Write .agent/state/last-result.env before ending."
```

If your runner does not support those flags, the same instruction can be pasted
into any coding agent that can read files, edit files, and run terminal commands.

## Project-specific reminders

- **Never operate on a real device, remote, VPS, or wallet from an automated
  session** — those are STOP conditions. Tests use loopback images, QEMU, and
  mocks only.
- **The active MCP boundary is `rmcp` plus ADAD-owned execution:** do not add
  claw-code runtime imports or describe the superseded vendoring design as
  current.
- **The leak battery is the authoritative security gate.** It runs on the booted
  image in EP-009/EP-010; never weaken or skip a leak/security test to pass.
