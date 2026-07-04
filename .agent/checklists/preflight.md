# Checklist — Preflight (start of every session)

- [ ] At repo root (AGENTS.md, COMMANDS.md, `.agent/` visible).
- [ ] `scripts/preflight.sh` exits `preflight: ok`.
- [ ] Dependencies available (`cargo`; musl target; host tools as the phase
      needs). Missing host tools → STOP, not `apt install`.
- [ ] Required env vars for this phase are documented in ENVIRONMENT.md.
- [ ] Command availability: every command the plan needs is in COMMANDS.md.
- [ ] Test harness available for this phase (unit always; integration/e2e as
      the phase provides them).
- [ ] Required secrets for any non-mockable test are present, else STOP.
- [ ] Local services needed (e.g. mock inference server) can start.
- [ ] `.agent/state/` directory present with all three state files.
- [ ] Active ExecPlan front matter valid.
- [ ] `.agent/state/blockers.md` reviewed for a filled Resolution to apply.
