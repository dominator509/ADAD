# Checklist — Agent Readiness (before starting an ExecPlan)

- [ ] Exactly one active ExecPlan is named in the invocation prompt.
- [ ] The ExecPlan is self-contained (no reliance on chat history).
- [ ] Front matter present and valid: `id`, `status`, `depends_on`, `verify`.
- [ ] The `verify` command exists in COMMANDS.md.
- [ ] Every `depends_on` plan has `status: complete` (check with
      `scripts/loop-status.sh` or `scripts/next-step.sh`).
- [ ] "Files to Read First" lists exact paths.
- [ ] "Files to Change" lists exact paths.
- [ ] Every milestone has a validation command from COMMANDS.md.
- [ ] Every milestone states an expected command output / observable result.
- [ ] Acceptance criteria are observable (command + expected result).
- [ ] Non-goals are explicit.
- [ ] STOP conditions understood (AGENTS.md §4).
- [ ] Recovery rules and the bounded-retry rule understood.
- [ ] Diff-review requirement understood.
- [ ] No hidden context required; no vague requirements remain.
