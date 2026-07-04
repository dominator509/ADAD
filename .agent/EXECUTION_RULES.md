# EXECUTION_RULES.md — Consolidated Rules for Coding Agents

These rules are binding every session. AGENTS.md is the full control plane; this
is the quick, enforceable checklist version.

1. **One active ExecPlan.** Work only on the plan named in your prompt.
2. **One session, one plan.** Never jump plans mid-session. Completing the plan
   or ending mid-plan `in-progress` are both normal.
3. **No hidden context.** Everything you need is in the repo. Do not rely on
   chat history or a previous session's memory. If it is not written down, it
   does not exist.
4. **No roadmap-only implementation.** ROADMAP.md is strategic. Implement only
   through an ExecPlan.
5. **Continue by default.** Do not stop after partial work or ask for next
   steps. Drive the plan to completion.
6. **STOP only for STOP conditions** (AGENTS.md §4). When you stop, record a
   blocker, set `status: blocked`, and write `RESULT=stop`.
7. **Anti-drift.** No refactors, renames, dep swaps, reorganizations, or cleanup
   outside the milestone. Extra changed files must be justified in the Decision
   Log.
8. **Anti-hallucination.** Do not invent commands, APIs, env vars, routes,
   config keys, or paths. Confirm names by reading repository files. Use only
   COMMANDS.md commands.
9. **Anti-fixation (bounded retry).** 1st failure: smallest fix. 2nd same-root:
   narrow diagnostic. 3rd same-root: stop the approach, record failed
   hypotheses, take a simpler in-scope path or write a blocker.
10. **Test before completion.** Every behavior change ships a test. Run the
    milestone's validation and confirm the exact expected result before ticking
    its box. Never weaken a leak/security test to pass a gate.
11. **Diff review.** Before `status: complete`, run `git diff --name-only` and
    reconcile against Files to Change.
12. **Session-result rule.** ALWAYS write `.agent/state/last-result.env` (schema
    in .agent/LOOP.md) as the FINAL file operation of every session, regardless
    of outcome.
13. **Clean mid-plan exit.** `RESULT=in_progress` after a validated milestone is
    a valid, healthy outcome. Record progress and end cleanly rather than
    rushing the whole plan or stalling.
14. **Final response rule.** End with the report described in AGENTS.md §15.
