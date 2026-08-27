# RELEASE.md — ADAD

## Release types
- **Image release:** a new bootable `adad.img` (the primary deliverable).
- **Tooling release:** static-binary updates folded into the next image.
- **Doc/blueprint release:** changes to `.agent/` and root docs only.

## Versioning
Semantic-ish: `vMAJOR.MINOR.PATCH`. MAJOR for vault-format or networking-posture
breaks; MINOR for features; PATCH for fixes. The vault config-version is tracked
separately and noted in release notes when it changes.

## Changelog
Keep `CHANGELOG.md` (created at first release) with Added/Changed/Fixed/Security
sections. Security-relevant changes (leak posture, DMS, killswitch) are always
called out.

## Branch strategy
- `main` is always releasable (verify passes).
- Feature work happens on short-lived branches named `ep-XXX-<slug>` matching the
  active ExecPlan.
- No long-lived divergent branches.

## Release candidate criteria
- [ ] `scripts/verify.sh` exits 0.
- [ ] `build/adad.img` builds; boots in QEMU.
- [ ] Leak battery passes (`build/leak-battery.pass` present).
- [ ] `build/adad-image.provenance` binds the image SHA-256 and source tree to
      the exact clean checkout under test.
- [ ] `scripts/production-readiness-check.sh` exits 0.

## Release checklist
See `.agent/checklists/release.md`.

## Smoke tests
Boot the RC image in QEMU; confirm the shipped application workflows, Tor
bootstrap, killswitch armed, leak battery pass, and that the provenance files
refer to the exact image being tested.

## Approvals
Device imaging and any real infra spend are human-gated. The blueprint loop
never cuts a production release autonomously past the readiness gate without the
human running the final on-hardware smoke.

## Release notes
State: version, vault config-version, changed leak/security posture (if any),
known risks, and the artifact checksum.

## Post-release monitoring
Per-session, on-box, RAM-only. There is no fleet telemetry (by design). Users
report issues out-of-band; fixes land in the next image.
