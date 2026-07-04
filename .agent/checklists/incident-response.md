# Checklist — Incident Response

- [ ] **Detect:** identify the incident (leak observed, killswitch fault, DMS
      misfire, vault issue). Capture the redacted TUI/log state now — logs are
      amnesic.
- [ ] **Triage:** classify severity. Any active leak or self-destruct fault is
      top severity.
- [ ] **Mitigate:** for an active leak, expect the killswitch to have dropped
      traffic (fail-closed); if it did not, that is the incident. Do not disable
      protections to investigate.
- [ ] **Communicate:** single-user — record locally; note in next CHANGELOG if
      security-relevant.
- [ ] **Resolve:** smallest fix; add a regression test that reproduces it.
- [ ] **Verify:** leak battery + verify pass; the specific failure no longer
      reproduces.
- [ ] **Document:** postmortem; ADR if an invariant/boundary changed.
- [ ] **Follow up:** confirm the regression test runs in CI/verify going forward.
