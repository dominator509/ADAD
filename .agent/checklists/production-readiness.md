# Checklist — Production Readiness (executable)

- [ ] Functionality: all core outcomes work; non-goals excluded.
- [ ] Tests: verify passes; leak battery passes against the image; regressions
      cover killswitch/DMS/vault-upgrade.
- [ ] Security: no secrets; audit clean; all egress via Tor/WireGuard;
      killswitch fail-closed; IPv6 off; Venice private-by-default.
- [ ] Privacy: tmpfs RAM-only confirmed; metafuse scrubbing; MAC randomized;
      logs RAM-only + redacted.
- [ ] Performance: inference tok/s in band; killswitch latency in target; VPS
      provisioning < 2 min (mock).
- [ ] Accessibility: keyboard-only reachable; high-contrast theme.
- [ ] Observability: redaction verified; TUI status reflects daemons; alerts
      render.
- [ ] Deployment: image builds; imaging runbook present; env vars documented.
- [ ] Rollback: re-image path documented; rollback drill done.
- [ ] Backups: vault backup/restore + upgrade tested (image-level).
- [ ] Docs: all root docs + runbooks current.
- [ ] Support: incident-response checklist present; known risks documented.
- [ ] `scripts/production-readiness-check.sh` exits 0.
