---
id: EP-012
status: complete
depends_on: [EP-011]
verify: scripts/min-system-sim.sh host32
---

# EP-012 — Real Model Throughput Smoke

## 1. Purpose / Big Picture
Pull a real `Qwen2.5-Coder-32B-Instruct-GGUF` quantization into a repo-local
cache, run the minimum-system simulator against the user's current host class,
and gather first-token / tok-s evidence instead of a skip marker.

## 2. Scope
- Add a repo-owned `llama.cpp` runtime fetch path.
- Extend the simulation harness to support Hugging Face model refs.
- Add a host-matching profile for the user's current CPU/RAM envelope.
- Run the harness against `Q3_K_M` and `Q4_K_M` as feasible.

## 3. Non-goals
- No system-wide installs.
- No real hardware writes.
- No claim that simulated throughput replaces real bare-metal validation.

## 4. Context and Orientation
EP-011 added the simulation harness but left inference timing at
`skipped:no-gguf` because the repo had no model fixture or local llama runtime.
This plan closes that specific gap with repo-local assets only.

## 5. Files to Read First
- AGENTS.md
- COMMANDS.md
- `scripts/min-system-sim.sh`
- `scripts/min-system-sim-inside.sh`
- `docs/minimum-system-simulation.md`
- EP-011

## 6. Files to Change
- `scripts/`
- `COMMANDS.md`
- `ENVIRONMENT.md`
- `docs/`

## 7. Interfaces and Contracts
- Repo-local runtime fetch script exits 0 and prints a success line.
- `scripts/min-system-sim.sh host32` exits 0 and records inference timing when a
  supported Hugging Face model ref is supplied.
- The model cache stays under `build/`.

## 8. Milestones

### M1 — Runtime/bootstrap path
- Goal: fetch a repo-local `llama.cpp` runtime and support Hugging Face model
  refs inside the simulator.
- Validation: runtime fetch command + `scripts/min-system-sim.sh host32`
- Expected: `min system sim: ok`

### M2 — Real-model evidence
- Goal: gather Q3/Q4 evidence or document the exact blocking resource limit.
- Validation: host32 simulation artifact + docs note.
- Expected: inference timing recorded or explicit model-specific failure reason.

## 9. Concrete Steps
1. Add the repo-local runtime fetch path.
2. Extend the simulator for HF refs and a host32 profile.
3. Pull Qwen2.5-Coder 32B Q3/Q4 into `build/` cache by running the simulator.
4. Record the measured result or the limiting factor.

## 10. Validation and Acceptance
- [x] Repo-local llama runtime fetch passes.
- [x] `scripts/min-system-sim.sh host32` passes.
- [x] Q3/Q4 evidence is recorded, or a precise resource/blocking reason is documented.
- [x] Docs and environment notes are updated.

## 11. Idempotence and Recovery
Downloads land under `build/` and can be reused across reruns. The simulator
remains image/QEMU-only and does not touch physical devices.

## 12. Progress
- [x] M1 — runtime/bootstrap path
- [x] M2 — real-model evidence
- [x] verify + status set to complete

## 13. Surprises & Discoveries
- The official `llama.cpp` Ubuntu x64 CPU asset at release `b9892` is not a
  standalone binary; `llama-server` needs its sibling shared objects, so the
  repo-local fetch path now writes a wrapper that exports `LD_LIBRARY_PATH`
  instead of copying the binary out of the extracted runtime tree.
- The original 60-second `llama-server` readiness wait was too small to
  distinguish runtime faults from large-model startup on this host class. The
  harness now persists logs under `build/min-system-sim-tmp/` and exposes
  `ADAD_LLAMA_READY_TIMEOUT`.
- `Qwen/Qwen2.5-Coder-32B-Instruct-GGUF:Q3_K_M` required a much longer cold
  start than the initial smoke budget. On the successful host32 run, the server
  log showed model load starting at `96.48.555.231`, then reaching
  `llama_server: model loaded` at `102.09.823.123`.
- `Qwen/Qwen2.5-Coder-32B-Instruct-GGUF:Q4_K_M` did not finish within a
  22,004,077 ms wall-clock budget (`Exit code 124` from the host command), so
  this host class still lacks a practical Q4 timing result.

## 14. Decision Log
- Runtime source: repo-local fetch from the official `ggml-org/llama.cpp`
  GitHub release tag `b9892`, cached under `build/tools/llama.cpp/`.
- Model acquisition path: official Hugging Face refs via
  `ADAD_PERF_HF_MODEL`, cached under `build/hf-cache` on the user's `C:` SSD.
- Host-matching profile: `host32` uses Docker `8` CPUs, `28g` memory and QEMU
  `8` vCPU / `16384` MB guest RAM.
- Evidence harness change: persist QEMU and `llama-server` logs under
  `build/min-system-sim-tmp/` and allow large-model startup via
  `ADAD_LLAMA_READY_TIMEOUT`.
- Measurement commands:
  - `ADAD_PERF_HF_MODEL=Qwen/Qwen2.5-Coder-32B-Instruct-GGUF:Q3_K_M`
    `ADAD_LLAMA_READY_TIMEOUT=14400 scripts/min-system-sim.sh host32`
  - `ADAD_PERF_HF_MODEL=Qwen/Qwen2.5-Coder-32B-Instruct-GGUF:Q4_K_M`
    `ADAD_LLAMA_READY_TIMEOUT=21600 scripts/min-system-sim.sh host32`

## 15. Outcomes & Retrospective
- Q3 evidence was captured successfully on the user's current host class. The
  latest successful host32 artifact recorded:
  - `boot_ms=54535`
  - `leak_ms=61556`
  - `inference_status=measured`
  - `inference_ms=102981`
  - `inference_tok_s=0.02`
  - `notes=completion_tokens=2`
- The persisted `llama-server` response for the successful Q3 run reported:
  - `prompt_tokens=35`
  - `completion_tokens=2`
  - `timings.prompt_per_second=0.020278617661791835`
  - `timings.predicted_per_second=0.037626382550855396`
- Q4 remains impractical on this host/simulation envelope as tested here. The
  long-run command exhausted a 6-hour wall-clock budget without producing a
  completed measurement, so EP-012 closes with Q3 floor evidence plus a
  host-specific Q4 limit instead of a Q4 throughput number.
