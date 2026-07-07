---
id: EP-011
status: complete
depends_on: [EP-010]
verify: scripts/min-system-sim.sh
---

# EP-011 — Minimum System Simulation Harness

## 1. Purpose / Big Picture
Add a reproducible harness that simulates floor/target/comfort hardware profiles
in the existing QEMU + Docker validation path, records timing data, and narrows
the minimum-device purchase decision before real-hardware testing.

## 2. Scope
- Add a repo-owned simulation command.
- Reuse the built image and existing boot/leak battery markers.
- Record per-profile boot and leak-battery timings.
- Support optional local inference timing when a tiny GGUF fixture is supplied.
- Document the command, profiles, and output artifact.

## 3. Non-goals
- No real hardware writes.
- No shipped GGUF weight in the repository.
- No attempt to prove firmware, USB controller, or real-drive behavior.

## 4. Context and Orientation
ENVIRONMENT.md + TESTING.md + PRODUCTION_READINESS.md already define the target
hardware shape and throughput expectations, but the repo lacks a reproducible
perf smoke command and a way to collect minimum-profile data before buying test
hardware.

## 5. Files to Read First
- AGENTS.md
- COMMANDS.md
- ENVIRONMENT.md
- TESTING.md
- PRODUCTION_READINESS.md
- `tests/os/boot-smoke.sh`
- `tests/os/boot-smoke-inside.sh`
- `tests/os/run-qemu-leak-battery-inside.sh`

## 6. Files to Change
- `scripts/min-system-sim.sh`
- `scripts/min-system-sim-inside.sh`
- `COMMANDS.md`
- `ENVIRONMENT.md`
- `TESTING.md`
- `docs/`

## 7. Interfaces and Contracts
- `scripts/min-system-sim.sh` exits 0 and prints `min system sim: ok`.
- The command writes a machine-readable artifact under `build/`.
- The command must not require a real device.
- The command may skip inference timing explicitly if no GGUF fixture is
  provided.

## 8. Milestones

### M1 — Harness command
- Goal: add the command and profile model.
- Validation: `scripts/min-system-sim.sh`
- Expected: `min system sim: ok`

### M2 — Documentation and output artifact
- Goal: document the profiles, optional GGUF input, and output format.
- Validation: inspect the generated artifact and docs for consistency.
- Expected: docs and artifact agree.

## 9. Concrete Steps
1. Add the simulation command and container/QEMU profile logic.
2. Record boot/leak timings and optional inference timing.
3. Document the command and its data format.
4. Run the harness and record results.

## 10. Validation and Acceptance
- [x] `scripts/min-system-sim.sh` → `min system sim: ok`
- [x] Floor/target/comfort profiles produce timing data.
- [x] Inference timing is either recorded or skipped with an explicit reason.
- [x] Command and environment documentation are updated.

## 11. Idempotence and Recovery
The harness is read-mostly apart from `build/` artifacts and uses QEMU + image
files only. Re-runs overwrite the latest simulation artifact safely.

## 12. Progress
- [x] M1 — harness command
- [x] M2 — documentation and output artifact
- [x] verify + status set to complete

## 13. Surprises & Discoveries
(Record host limitations, missing fixtures, or profile caveats.)
- The repository still ships no GGUF fixture, so inference timing correctly
  records `skipped:no-gguf` instead of inventing throughput numbers.
- On this Windows-hosted Docker/QEMU path (`accel=tcg`), higher-profile results
  were not perfectly monotonic: the `target` profile measured slightly faster
  than `floor` and `comfort` in the final sweep. That makes the harness useful
  for comparative data gathering, but not a substitute for real hardware.

## 14. Decision Log
(Record profile shapes, output format, and inference-fixture handling.)
- Added `scripts/min-system-sim.sh` as the documented command and
  `scripts/min-system-sim-inside.sh` as the builder-local helper.
- Chose three named profiles:
  - `floor`: Docker `2` CPUs / `10g`, QEMU `2` vCPU / `8192` MB
  - `target`: Docker `4` CPUs / `18g`, QEMU `4` vCPU / `16384` MB
  - `comfort`: Docker `8` CPUs / `34g`, QEMU `8` vCPU / `32768` MB
- Wrote machine-readable output to `build/min-system-sim.latest.tsv` plus a
  timestamped archive copy under `build/`.
- Made inference timing optional via `ADAD_PERF_GGUF`; if no tiny GGUF fixture
  is present, the harness records an explicit skip reason and still returns
  usable QEMU timing data.
- Touched `tests/os/boot-smoke-inside.sh` and
  `tests/os/run-qemu-leak-battery-inside.sh` outside the initial file list to
  add the already-implied `ADAD_QEMU_SMP` configurability needed by the new
  profile harness without forking the existing QEMU defaults.
- Updated `ASSUMPTIONS.md` outside the initial file list because A5 is the
  repository-wide minimum-hardware assumption this harness now partially
  validates.

## 15. Outcomes & Retrospective
(Filled at completion — summarize usable purchase guidance and residual limits.)
- `scripts/min-system-sim.sh` passed and wrote
  `build/min-system-sim.latest.tsv`.
- Final default sweep data:
  - `floor`: boot `92760` ms, leak `133099` ms
  - `target`: boot `105690` ms, leak `104986` ms
  - `comfort`: boot `118335` ms, leak `122707` ms
- The harness now provides pre-hardware CPU/RAM profile data for purchase
  screening without touching a real device.
- Residual limits: inference tok/s remains unmeasured until a tiny GGUF fixture
  is supplied; firmware, USB controller behavior, Secure Boot, thermal limits,
  and real-drive throughput still require physical-hardware validation.
