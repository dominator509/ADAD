You are one iteration of an unattended build loop. No human will answer
questions. Placeholder: [EXECPLAN_PATH]

Do these steps in order:

1. Read AGENTS.md, COMMANDS.md, .agent/PLANS.md, .agent/LOOP.md,
   .agent/EXECUTION_RULES.md, and [EXECPLAN_PATH].
2. Read .agent/state/blockers.md. If a blocker for this ExecPlan has a filled
   Resolution, apply it, mark that blocker RESOLVED, and continue.
3. Run scripts/preflight.sh. If it fails, fix the smallest cause using only
   COMMANDS.md commands (bounded retry: max 3 attempts per root cause).
4. If the ExecPlan status is not-started, set it to in-progress.
5. Read the Progress section. Resume at the first unchecked milestone.
6. For each milestone, in order: implement exactly what the milestone
   specifies, run its validation command, confirm the expected result, check
   the Progress box, and record decisions in the Decision Log. Do not skip
   milestones. Do not work on any other ExecPlan. Do not broaden scope.
7. Apply the bounded-retry rule from AGENTS.md to every failure. After the
   third same-root failure, record failed hypotheses in Surprises &
   Discoveries; if a simpler in-scope path exists, take it; otherwise write a
   blocker per .agent/LOOP.md, set status: blocked, and go to step 9 with
   RESULT=blocked.
8. When all milestones are checked: run the plan's verify command from the
   front matter, run git diff --name-only, compare against Files to Change
   (justify any extras in the Decision Log), complete Outcomes &
   Retrospective, and set status: complete.
9. ALWAYS end the session by writing .agent/state/last-result.env using the
   exact schema in .agent/LOOP.md. This must be your final file write.

Rules: do not ask for next steps; do not invent commands, APIs, env vars,
routes, tables, or config keys - verify names by reading repository files;
stop only for STOP conditions in AGENTS.md, and even then perform step 9
before ending.
