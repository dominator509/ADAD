# Prompt: Execute an Active ExecPlan to Completion

Placeholders:
- [EXECPLAN_PATH] — path to the ExecPlan you must execute.
- [OPTIONAL_USER_REQUEST] — optional extra instruction; ignore if empty. It may
  narrow priorities but must never broaden scope beyond the ExecPlan.

You are a coding agent with no memory of any prior conversation. Everything you
need is in the repository files. Do these in order:

1. Read AGENTS.md (control plane and STOP conditions).
2. Read COMMANDS.md (the only commands you may run).
3. Read .agent/PLANS.md (the ExecPlan standard).
4. Read [EXECPLAN_PATH] in full, including front matter and Progress.
5. Run scripts/preflight.sh and resolve any failure with the smallest fix
   using only COMMANDS.md commands.
6. If status is not-started, set it to in-progress.
7. Implement milestones strictly in order. After each milestone: run its
   validation command, confirm the expected result, tick its Progress box, and
   append any decisions to the Decision Log.
8. Obey the bounded-retry rule: 1st failure smallest fix; 2nd same-root failure
   narrower diagnostic; 3rd same-root failure stop that approach, record failed
   hypotheses, take a simpler in-scope path, or write a blocker and set
   status: blocked.
9. When all milestones pass: run the front-matter `verify` command, run
   `git diff --name-only`, compare changed files against "Files to Change"
   (justify extras in the Decision Log), fill Outcomes & Retrospective, and set
   status: complete.
10. Write .agent/state/last-result.env per .agent/LOOP.md as your FINAL file
    write — every session, no exceptions.

Do NOT ask the user for next steps. Proceed autonomously. Stop only under a
STOP condition in AGENTS.md, and even then, write last-result.env first.

[OPTIONAL_USER_REQUEST]
