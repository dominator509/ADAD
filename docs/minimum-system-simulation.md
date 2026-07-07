# Minimum System Simulation

## Purpose
Use `scripts/min-system-sim.sh` to gather pre-hardware timing data before
buying a test device. The harness does not replace real-hardware validation; it
constrains the existing Docker + QEMU path to approximate lower-end profiles.

## Command
`scripts/min-system-sim.sh [floor|target|comfort|host32 ...]`

If no profiles are supplied, it runs `floor`, `target`, and `comfort`.

## Profiles
- `floor`: Docker `2` CPUs, `10g` memory; QEMU `2` vCPU, `8192` MB guest RAM.
- `target`: Docker `4` CPUs, `18g` memory; QEMU `4` vCPU, `16384` MB guest RAM.
- `comfort`: Docker `8` CPUs, `34g` memory; QEMU `8` vCPU, `32768` MB guest RAM.
- `host32`: Docker `8` CPUs, `28g` memory; QEMU `8` vCPU, `16384` MB guest RAM.

## Output
The harness writes:
- `build/min-system-sim.latest.tsv`
- `build/min-system-sim.<timestamp>.tsv`
- `build/min-system-sim-tmp/` (latest QEMU + llama-server logs)

Columns:
- `timestamp_utc`
- `profile`
- `docker_cpus`
- `docker_memory`
- `qemu_smp`
- `qemu_mem_mb`
- `boot_ms`
- `leak_ms`
- `inference_status`
- `inference_ms`
- `inference_tok_s`
- `notes`

## Inference Timing
Inference timing is optional. To enable it, provide a repo-visible tiny GGUF:
- `ADAD_PERF_GGUF=<repo-relative-path-to-tiny.gguf>`

Or provide a Hugging Face model ref that `llama.cpp` can fetch into the
repo-local cache under `build/hf-cache`:
- `ADAD_PERF_HF_MODEL=Qwen/Qwen2.5-Coder-32B-Instruct-GGUF:Q3_K_M`

Optional override for the server binary visible inside the builder container:
- `ADAD_LLAMA_SERVER_BIN=<binary-or-repo-relative-path>`

Optional ready-wait budget for large models:
- `ADAD_LLAMA_READY_TIMEOUT=300`

If either input is unavailable, the harness records an explicit skip reason
instead of failing the simulation run.

## Limits
- The harness approximates CPU and RAM pressure only.
- It does not prove USB boot behavior, firmware quirks, Secure Boot behavior,
  real external-drive throughput, Wi-Fi/NIC compatibility, or thermal limits.
- Final device selection still requires a real-hardware boot smoke.
