# PLANS.md — The ExecPlan Standard

An ExecPlan is a self-contained implementation document for one feature or
system change. **A new agent with no prior conversation must be able to continue
from the ExecPlan alone.**

If an instruction is not written into the ExecPlan (or a file it names), it does
not exist. Do not rely on chat history, prior sessions' memory, or this
conversation.

## Front matter (required section 0)

Every file in `.agent/execplans/` MUST begin with the YAML block defined in
`.agent/LOOP.md`:

```yaml
---
id: EP-003
status: not-started        # not-started | in-progress | blocked | complete
depends_on: [EP-001, EP-002]
verify: scripts/verify.sh
---
```

**Front-matter `status` is the single source of truth for plan status.** No
other file may claim a different status. `scripts/next-step.sh` and
`scripts/loop-status.sh` read this field directly.

## Required sections (every ExecPlan, in this order)

0. YAML front matter (id, status, depends_on, verify)
1. Purpose / Big Picture
2. Scope
3. Non-goals
4. Context and Orientation
5. Files to Read First
6. Files to Change
7. Interfaces and Contracts
8. Milestones
9. Concrete Steps
10. Validation and Acceptance
11. Idempotence and Recovery
12. Progress
13. Surprises & Discoveries
14. Decision Log
15. Outcomes & Retrospective

## Execution rules

- One session works on exactly one ExecPlan.
- Implement milestones strictly in order.
- A session may complete the plan OR end mid-plan with `status: in-progress`.
  Both are normal outcomes. The next session resumes from the Progress section.
- Do not broaden scope beyond "Scope." Anything in "Non-goals" is forbidden.

## Milestone rules

Every milestone must specify:

- **Goal** — one sentence, observable.
- **Files to read** — exact paths.
- **Files to change** — exact paths.
- **Exact edits expected** — what to add/modify, concretely.
- **Validation command** — a command from COMMANDS.md.
- **Expected result** — the exact success line or observable outcome.
- **Recovery** — what to do if validation fails (points at the bounded-retry
  rule and any milestone-specific hint).

## Validation rules

- After each milestone, run its validation command and confirm the exact
  expected result before ticking its Progress box.
- The plan's front-matter `verify` command is the final gate before
  `status: complete`.

## Acceptance rules

"Validation and Acceptance" lists objective criteria: command + expected
result. Acceptance is never "looks good" or "seems to work."

## Idempotence and recovery rules

- Re-running a completed milestone must not corrupt state (guards, existence
  checks, `--check` modes where possible).
- "Idempotence and Recovery" states how to safely re-run the plan and how to
  recover from a half-applied milestone.

## Progress update rules

- The Progress section is a checkbox list, one box per milestone (plus a final
  "verify + status" box).
- Tick a box only after its validation passed in THIS repository, not because
  it "should" pass.

## Decision Log rules

- Record every non-trivial in-scope decision: what was chosen, the alternatives,
  and why. Include any dependency added and any assumption made.
- Decisions are binding on later sessions unless explicitly superseded by a new
  dated entry.

## Completion rules

A plan is `complete` only when AGENTS.md §14 "Definition of done" holds:
all acceptance criteria pass, `verify` exits 0, Progress fully ticked,
front-matter status set, diff matches Files to Change, risks documented, and
`.agent/state/last-result.env` written with `RESULT=plan_complete`.
