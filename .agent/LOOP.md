# LOOP.md — Unattended Build Loop Protocol

This repository (ADAD) is built by an unattended loop. Nothing here is
aspirational: the scripts referenced below exist in `scripts/` and the state
files exist in `.agent/state/`.

## Overview

`scripts/loop.sh` drives the build:

1. Calls `scripts/next-step.sh` to pick the next runnable ExecPlan.
2. Renders `.agent/prompts/loop-iteration.md`, substituting `[EXECPLAN_PATH]`.
3. Invokes exactly one fresh coding-agent session via `$AGENT_CMD "<prompt>"`.
4. Reads `.agent/state/last-result.env` (the session's final write).
5. Updates `.agent/state/build-state.env` and repeats.

The loop stops when `next-step.sh` reports `DONE`, when a session reports
`stop`/`blocked`, when the same plan+milestone fails 3 sessions in a row, or
when `MAX_ITERATIONS` is exhausted.

## ExecPlan front matter (required section 0 of every ExecPlan)

Every file in `.agent/execplans/` MUST begin with exactly this block:

```yaml
---
id: EP-003
status: not-started        # not-started | in-progress | blocked | complete
depends_on: [EP-001, EP-002]
verify: scripts/verify.sh
---
```

Rules:

- `id` matches the filename prefix (`EP-003-*.md` → `id: EP-003`).
- `status` is the SINGLE SOURCE OF TRUTH for that plan's state. No other file
  may claim a different status.
- `depends_on` is a list of ExecPlan ids that must be `complete` before this
  plan is runnable. May be empty (`[]`).
- `verify` is a command from `COMMANDS.md` that gates completion.
- Only the agent (or a human resolving a blocker) may change `status`.
- `status: complete` may be set ONLY after `verify` exits 0 AND every
  acceptance criterion in the plan is checked off.

## Session result schema

The agent MUST write `.agent/state/last-result.env` as its FINAL file
operation, EVERY session, in `KEY=VALUE` format — one per line, no quotes, no
spaces around `=`:

```text
RESULT=plan_complete       # plan_complete | in_progress | blocked | stop | failed
EXECPLAN=EP-003
MILESTONE=M4               # last milestone worked on
VERIFY_EXIT=0              # exit code of last validation command run
FAILING_COMMAND=none       # exact command if VERIFY_EXIT != 0, else none
NOTES=one line, no newlines
```

Meaning of `RESULT`:

- `plan_complete` — front-matter status set to `complete`, verify passed.
- `in_progress` — healthy partial progress; next session resumes. NORMAL.
- `blocked` — three-strike anti-fixation exhausted OR STOP condition; a blocker
  is recorded in `.agent/state/blockers.md`.
- `stop` — STOP condition per AGENTS.md; blocker recorded.
- `failed` — session could not make progress. Use `NOTES` to say why.

A session that does NOT write `last-result.env` is treated by the loop as a
failed session (`RESULT=failed`, `MILESTONE=unknown`).

## Blocker protocol

Each blocker in `.agent/state/blockers.md` is a section using this template:

```text
## BLK-007 (EP-003, M4) — OPEN
Blocker: exact description
Evidence: exact command output or file/line
Smallest decision needed: one sentence
Recommended default: one sentence
Resolution: (empty — human fills this in)
```

Lifecycle:

1. Agent hits a STOP condition or exhausts the 3-strike retry rule.
2. Agent appends the blocker (heading `— OPEN`), sets the ExecPlan
   `status: blocked`, writes `RESULT=blocked` (or `stop`), ends the session.
3. `scripts/loop.sh` exits nonzero and prints the blocker.
4. A human fills in `Resolution:`, changes the ExecPlan front-matter status
   from `blocked` back to `in-progress`, and reruns `scripts/loop.sh`.
5. The next agent session reads the filled Resolution, applies it, changes the
   blocker heading from `— OPEN` to `— RESOLVED`, and continues.

Humans may edit `.agent/state/blockers.md` ONLY to fill the `Resolution:`
section. No other hand-editing of `.agent/state/` files is permitted
(see COMMANDS.md → Forbidden actions).

## Build completion definition

The build is DONE only when all three are true:

1. Every ExecPlan has `status: complete`.
2. `scripts/verify.sh` exits 0.
3. `scripts/production-readiness-check.sh` exits 0.

`scripts/loop.sh` prints `build: complete` only in that case.

## Loop exit codes

- `0`  — build complete.
- `3`  — halted by a blocker or by an agent `stop`/`blocked` result.
- `4`  — halted by 3 consecutive failed sessions on the same plan+milestone.
- `5`  — iteration budget (`MAX_ITERATIONS`) exhausted.
- `1`  — internal error (e.g. `next-step.sh` failure).
