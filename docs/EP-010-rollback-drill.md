# EP-010 Rollback Drill

## Scope
This drill verifies the automated, non-destructive rollback path available to
agents. It does not write a USB drive, physical block device, production vault,
or remote system.

## Command
`tests/os/rollback-drill.sh`

## Procedure
1. Use the EP-009 Docker image builder.
2. Copy `build/adad.img` inside the container to a temporary rollback artifact.
3. Boot the temporary artifact in QEMU.
4. Require the same hardening markers as boot smoke:
   - `adad-killswitch: armed`
   - `adad-ipv6: disabled`
   - `adad-mac: randomized`

## Result
Passed. The command returned `rollback drill: ok`.

## Boundary
Real image rollback remains human-run per `ROLLBACK.md`; automated sessions only
prove image-artifact bootability through QEMU.
