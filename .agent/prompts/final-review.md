# Prompt: Final Review of an ExecPlan

Placeholder: [EXECPLAN_PATH]

The ExecPlan's milestones are believed complete. Verify that rigorously before
allowing status: complete.

1. Read AGENTS.md, COMMANDS.md, and [EXECPLAN_PATH].
2. Run scripts/verify.sh (full suite). It must exit 0.
3. If this is the final plan (EP-010) or the plan touches release surface, also
   run scripts/production-readiness-check.sh.
4. Run `git diff --name-only`. Compare the changed-file list against the
   ExecPlan's "Files to Change". Every extra file must have a justification in
   the Decision Log; if any does not, either justify it or revert it.
5. Walk the "Validation and Acceptance" section. Confirm EVERY acceptance
   criterion is objectively met (command + expected result), not assumed.
6. Confirm no secrets are committed and no production/host data was mutated.
7. Fill in Outcomes & Retrospective: what shipped, what changed vs. plan,
   remaining risks.
8. Set front-matter status: complete ONLY if steps 2–6 all passed.
9. Produce a final report: status, changed files, commands run + results,
   acceptance criteria status, decisions, remaining risks, whether
   production-readiness passed.
10. Write .agent/state/last-result.env per .agent/LOOP.md as your final write.
