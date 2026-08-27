# PRODUCTION_READINESS.md — ADAD

## Definition of production readiness

Current status: **NO-GO**. The checklist is a release gate, not a statement
that the completed historical ExecPlans prove production behavior. Pure models,
mock transports, headless UI tests, and source-only verification do not satisfy
the image, Linux-backend, external-service, or hardware criteria below.

ADAD is production-ready when every ExecPlan has `status: complete`,
`scripts/verify.sh` exits 0, and `scripts/production-readiness-check.sh` exits 0
(which requires a clean checkout, a bootable `build/adad.img`, provenance
matching the current source tree, and an on-image leak battery marker whose
image digest matches the tested image), and all operational docs/runbooks are
present. The executable readiness check also refuses a pass while any checklist
item below remains unchecked; plan completion and source-only tests are not
substitutes for that evidence. Real hardware and external-service validation
remain required.

## Functional readiness
- [ ] Boot → use local/cloud agent → shutdown leaves zero host-disk trace.
- [ ] Local `llama-server` inference works; provider switching (openai/venice)
      works over WireGuard.
- [ ] Production XMR wallet RPC and VPS transports exist; mock tests alone do
      not establish this. Real transfers/provisioning remain human-gated.
- [ ] LUKS2 vault create/unlock/lock/seal works; DMS + panic wipe work.
- [ ] Git publish uses the stable pseudonym with real metadata stripped.
- [ ] All non-goals remain excluded (no MAC impersonation, no per-push rotation,
      no clearnet, no IPv6, no DB).

## Test readiness
- [ ] format, lint, typecheck, unit, integration, build, security, audit, smoke
      all pass.
- [ ] E2E leak battery passes against the booted image.
- [ ] Regression tests cover killswitch, DMS clock-freeze bypass, vault upgrade.

## Security readiness
- [ ] No committed secrets; dependency audit clean.
- [ ] All egress via Tor/WireGuard; killswitch fail-closed; IPv6 off; no DNS
      leak; no local-discovery chatter.
- [ ] Venice defaults to private models.
- [ ] Secrets zeroized on lock/shutdown/panic.

## Privacy readiness
- [ ] tmpfs root confirmed RAM-only; no host writes across a boot cycle.
- [ ] metafuse scrubs timestamps/EXIF/UIDs on vault files.
- [ ] MAC randomized per session.
- [ ] Logs RAM-only and redacted.

## Performance readiness
- [ ] Inference tok/s within tier band (smoke).
- [ ] Killswitch latency within target.
- [ ] VPS provisioning < 2 min (mock-timed).

## Accessibility readiness
- [ ] Keyboard-only reachability of every TUI action (tested).
- [ ] High-contrast theme present.

## Observability readiness
- [ ] Structured RAM-only logs with verified redaction.
- [ ] TUI status monitor reflects real daemon state.
- [ ] Alerts render for killswitch/tunnel/DMS/vault events.

## Deployment readiness
- [ ] `live-build` inputs are immutably pinned and two clean isolated builds
      produce the same image digest; deterministic timestamps alone are not
      sufficient.
- [ ] Imaging runbook present; device-write is human-gated.
- [ ] Environment variables documented in ENVIRONMENT.md.

## Rollback readiness
- [ ] Re-image rollback documented; vault version compatibility window stated.
- [ ] Rollback drill performed (EP-010).

## Data readiness
- [ ] Vault backup/restore documented and tested (image-level).
- [ ] Vault-layout upgrade path tested; no destructive change without backup.

## Documentation readiness
- [ ] All root docs and runbooks present and current.
- [ ] DECISIONS/ASSUMPTIONS reconciled with the built system.

## Support readiness
- [ ] Incident-response checklist present.
- [ ] Known risks documented in Outcomes & Retrospective of EP-010.

## Final launch gate
All boxes above checked AND the three loop-completion conditions hold AND a
manual on-hardware boot smoke confirms the leak battery on real hardware.

## Checklist
See `.agent/checklists/production-readiness.md` for the executable version.
