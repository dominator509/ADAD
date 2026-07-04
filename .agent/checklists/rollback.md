# Checklist — Rollback

- [ ] Rollback trigger identified (leak regression / killswitch/DMS fault /
      vault fault / boot failure).
- [ ] Owner: operator; automated device writes are a STOP.
- [ ] Rollback method chosen (image re-image / config revert / crate re-pin).
- [ ] Vault compatibility confirmed; restore from vault image backup (never
      mutate in place); no compatible path → STOP.
- [ ] Verify after rollback: boot smoke + leak battery + vault unlock pass.
- [ ] Communication: CHANGELOG Security note if leak-related.
- [ ] Postmortem authored; regression test added; ADR if a boundary was involved.
