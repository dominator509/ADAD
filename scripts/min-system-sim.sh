#!/usr/bin/env sh
# Simulate floor/target/comfort hardware profiles using the existing EP-009
# builder + QEMU path and record timing data under build/.
set -eu
cd "$(dirname "$0")/.."

builder="${ADAD_IMAGE_BUILDER:-adad-ep009-builder:local}"
workspace="$PWD"
case "$(uname -s 2>/dev/null || echo unknown)" in
  MINGW*|MSYS*|CYGWIN*)
    workspace="$(pwd -W)"
    export MSYS_NO_PATHCONV=1
    export MSYS2_ARG_CONV_EXCL='*'
    ;;
esac

require_inference="${ADAD_REQUIRE_INFERENCE:-0}"
case "$require_inference" in
  0|1) ;;
  *)
    echo "ERROR: ADAD_REQUIRE_INFERENCE must be 0 or 1." >&2
    exit 1
    ;;
esac
if [ "$require_inference" = "1" ] \
  && [ -z "${ADAD_PERF_GGUF:-}" ] \
  && [ -z "${ADAD_PERF_HF_MODEL:-}" ]; then
  echo "ERROR: required inference acceptance needs ADAD_PERF_GGUF or ADAD_PERF_HF_MODEL." >&2
  exit 1
fi

[ -f build/adad.img ] || {
  echo "ERROR: build/adad.img not found. Run scripts/build-image.sh first." >&2
  exit 1
}

if [ "$#" -eq 0 ]; then
  set -- floor target comfort
fi

mkdir -p build
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
latest="build/min-system-sim.latest.tsv"
archive="build/min-system-sim.${stamp}.tsv"

printf 'timestamp_utc\tprofile\tdocker_cpus\tdocker_memory\tqemu_smp\tqemu_mem_mb\tboot_ms\tleak_ms\tinference_status\tinference_ms\tinference_tok_s\tnotes\n' >"$latest"

profile_settings() {
  case "$1" in
    floor)
      docker_cpus=2
      docker_memory=10g
      qemu_smp=2
      qemu_mem=8192
      ;;
    target)
      docker_cpus=4
      docker_memory=18g
      qemu_smp=4
      qemu_mem=16384
      ;;
    comfort)
      docker_cpus=8
      docker_memory=34g
      qemu_smp=8
      qemu_mem=32768
      ;;
    host32)
      docker_cpus=8
      docker_memory=28g
      qemu_smp=8
      qemu_mem=16384
      ;;
    *)
      echo "ERROR: unknown profile '$1'. Use floor, target, comfort, or host32." >&2
      exit 1
      ;;
  esac
}

for profile in "$@"; do
  profile_settings "$profile"
  metrics="$(
    docker run --rm \
      --cpus "$docker_cpus" \
      --memory "$docker_memory" \
      -e ADAD_SIM_PROFILE="$profile" \
      -e ADAD_QEMU_SMP="$qemu_smp" \
      -e ADAD_QEMU_MEM="$qemu_mem" \
      -e ADAD_PERF_GGUF="${ADAD_PERF_GGUF:-}" \
      -e ADAD_PERF_HF_MODEL="${ADAD_PERF_HF_MODEL:-}" \
      -e ADAD_LLAMA_READY_TIMEOUT="${ADAD_LLAMA_READY_TIMEOUT:-300}" \
      -e ADAD_LLAMA_SERVER_BIN="${ADAD_LLAMA_SERVER_BIN:-llama-server}" \
      -e ADAD_LLAMA_CPP_RELEASE_TAG="${ADAD_LLAMA_CPP_RELEASE_TAG:-b9892}" \
      -e ADAD_LLAMA_CPP_ARCHIVE_SHA256="${ADAD_LLAMA_CPP_ARCHIVE_SHA256:-}" \
      -e ADAD_REQUIRE_INFERENCE="$require_inference" \
      -v "$workspace:/workspace" \
      -w /workspace \
      "$builder" \
      sh scripts/min-system-sim-inside.sh
  )"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$stamp" "$profile" "$docker_cpus" "$docker_memory" "$qemu_smp" "$qemu_mem" "$metrics" \
    >>"$latest"
done

cp "$latest" "$archive"
echo "min system sim: ok"
