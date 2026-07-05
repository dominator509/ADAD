# CHANGELOG

## 0.1.0-rc.1

### Added
- Bootable Debian-Live ADAD image artifact at `build/adad.img`.
- Containerized EP-009 image builder for hosts without native `live-build`,
  `squashfs-tools`, or QEMU packages.
- QEMU boot smoke, QEMU on-image leak battery, and EP-010 rollback drill.
- Production readiness evidence notes:
  - `docs/EP-010-performance-review.md`
  - `docs/EP-010-rollback-drill.md`

### Changed
- MCP substrate uses the official Rust MCP SDK with ADAD-owned execution logic.
- Image hardening is applied at boot: IPv6 disabled, nftables default-drop
  posture, Tor-default firewall ownership, and MAC randomization.

### Fixed
- Replaced the blocked host-only EP-009 image-build path with a repo-owned
  Docker builder that does not mutate host packages or touch physical devices.

### Security
- Leak battery now runs against the booted image and writes
  `build/leak-battery.pass` only after passing all required markers.
- Vault upgrade tests verify backup creation and payload preservation.
- Direct physical-device imaging, real VPS provisioning, real XMR movement, and
  final on-hardware smoke remain human-gated.

### Artifact
- `build/adad.img`
- SHA-256:
  `2f1e3e5c0e3c5facf30eb5c8c296f718b362551583f567b72b4b8d10156c1904`
