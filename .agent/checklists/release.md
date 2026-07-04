# Checklist — Release

- [ ] Version chosen (and vault config-version noted if changed).
- [ ] CHANGELOG updated (Added/Changed/Fixed/Security).
- [ ] Release candidate criteria met (RELEASE.md).
- [ ] Staging (QEMU) smoke: boot, Tor bootstrap, killswitch armed.
- [ ] Leak battery passes (`build/leak-battery.pass` present).
- [ ] `scripts/production-readiness-check.sh` exits 0.
- [ ] Artifact checksum recorded in release notes.
- [ ] Production imaging is human-performed (device target confirmed).
- [ ] Post-deploy on-hardware boot smoke passes.
- [ ] Post-release: session-level monitoring only (no fleet telemetry).
