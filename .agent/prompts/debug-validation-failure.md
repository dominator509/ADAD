# Prompt: Debug a Failing Validation Command

Placeholders:
- [EXECPLAN_PATH] — the active ExecPlan.
- [FAILING_COMMAND] — the exact command that failed (from COMMANDS.md).

You are debugging ONE failing command. Stay narrow.

1. Read AGENTS.md and COMMANDS.md. Confirm [FAILING_COMMAND] is a real command
   from COMMANDS.md. If it is not, the fix is to correct the caller, not to
   invent a command.
2. Do NOT rewrite unrelated code. Do NOT refactor. Do NOT touch files outside
   the ExecPlan's "Files to Change" without recording justification.
3. Run [FAILING_COMMAND] and capture the EXACT error output.
4. Form ONE hypothesis about the root cause. Write it down (Surprises &
   Discoveries).
5. Make the SMALLEST change that would test that hypothesis.
6. Re-run the narrowest command that reproduces the failure (not the whole
   verify suite) to check the hypothesis.
7. Bounded retry: after three failures sharing the same root cause, stop.
   Record all three failed hypotheses in Surprises & Discoveries. If a simpler
   in-scope implementation avoids the failure, take it. Otherwise write a
   blocker per .agent/LOOP.md and set status: blocked.
8. Once the narrow command passes, run the ExecPlan's full `verify` command to
   confirm nothing else regressed.
9. Update the ExecPlan (Progress, Decision Log).
10. Write .agent/state/last-result.env per .agent/LOOP.md as your final write.
