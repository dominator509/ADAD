# Prompt: Continue a Partially Completed ExecPlan

Placeholder: [EXECPLAN_PATH]

This ExecPlan was started by an earlier session and left in-progress. That is
normal. Resume it cleanly:

1. Read AGENTS.md, COMMANDS.md, .agent/PLANS.md, .agent/LOOP.md, and
   [EXECPLAN_PATH].
2. Read the Progress section — it is the source of truth for where work
   stopped. Identify the first unchecked milestone.
3. Read Surprises & Discoveries — prior sessions recorded failed hypotheses and
   environment facts here. Do not repeat a failed hypothesis.
4. Read the Decision Log — prior in-scope decisions are binding; do not reverse
   them without recording a superseding entry.
5. Re-validate the last completed milestone by running its validation command
   before building on it. If it now fails, treat that as the first failure
   under the bounded-retry rule and fix it before proceeding.
6. Resume at the first unchecked milestone and continue in order.
7. Follow the same completion and reporting steps as
   execute-active-execplan.md, including setting front-matter status.
8. Write .agent/state/last-result.env per .agent/LOOP.md as your final write.

Do not ask for next steps. Stop only for STOP conditions.
