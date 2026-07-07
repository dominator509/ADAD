#!/usr/bin/env sh
# Internal helper for scripts/min-system-sim.sh. Run inside adad-ep009-builder.
set -eu
cd /workspace

image="${ADAD_BOOT_IMAGE:-build/adad.img}"
profile="${ADAD_SIM_PROFILE:-unknown}"
qemu_mem="${ADAD_QEMU_MEM:-1536}"
qemu_smp="${ADAD_QEMU_SMP:-2}"
boot_timeout="${ADAD_BOOT_TIMEOUT:-180}"
leak_timeout="${ADAD_LEAK_BATTERY_TIMEOUT:-240}"
tmp_root="${ADAD_SIM_TMPDIR:-/workspace/build/min-system-sim-tmp}"

mkdir -p "$tmp_root"

[ -f "$image" ] || {
  echo "ERROR: $image not found. Run scripts/build-image.sh first." >&2
  exit 1
}

now_ms() {
  date +%s%3N
}

cleanup_pid() {
  pid="${1:-}"
  [ -n "$pid" ] || return 0
  if kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  fi
}

wait_for_markers() {
  log="$1"
  pid="$2"
  timeout_seconds="$3"
  shift 3
  start_ms="$(now_ms)"
  deadline_ms=$((start_ms + (timeout_seconds * 1000)))
  while :; do
    found=yes
    for marker in "$@"; do
      if ! grep -q "$marker" "$log" 2>/dev/null; then
        found=no
        break
      fi
    done
    if [ "$found" = yes ]; then
      elapsed_ms=$(( $(now_ms) - start_ms ))
      cleanup_pid "$pid"
      printf '%s\n' "$elapsed_ms"
      return 0
    fi
    if ! kill -0 "$pid" 2>/dev/null; then
      break
    fi
    if [ "$(now_ms)" -ge "$deadline_ms" ]; then
      break
    fi
    sleep 1
  done

  cleanup_pid "$pid"
  return 1
}

measure_boot_ms() {
  log="$tmp_root/adad-min-sim-boot-${profile}.log"
  rm -f "$log"

  qemu-system-x86_64 \
    -m "$qemu_mem" \
    -smp "$qemu_smp" \
    -machine accel=tcg \
    -display none \
    -serial stdio \
    -no-reboot \
    -nic user,model=virtio-net-pci \
    -cdrom "$image" \
    -boot d >"$log" 2>&1 &
  pid=$!

  if elapsed="$(wait_for_markers "$log" "$pid" "$boot_timeout" \
    'adad-killswitch: armed' \
    'adad-ipv6: disabled' \
    'adad-mac: randomized')"; then
    printf '%s\n' "$elapsed"
    return 0
  fi

  echo "ERROR: boot markers not observed for profile '$profile'." >&2
  tail -n 120 "$log" >&2 || true
  exit 1
}

measure_leak_ms() {
  tmp="$tmp_root/adad-min-sim-leak-${profile}.$$"
  log="$tmp_root/adad-min-sim-leak-${profile}.log"
  rm -rf "$tmp"
  mkdir -p "$tmp"
  rm -f "$log"
  trap 'rm -rf "$tmp"' EXIT INT TERM

  xorriso -osirrox on -indev "$image" \
    -extract /live/vmlinuz "$tmp/vmlinuz" >"$tmp_root/adad-min-sim-kernel.log" 2>&1
  xorriso -osirrox on -indev "$image" \
    -extract /live/initrd.img "$tmp/initrd.img" >"$tmp_root/adad-min-sim-initrd.log" 2>&1

  qemu-system-x86_64 \
    -m "$qemu_mem" \
    -smp "$qemu_smp" \
    -machine accel=tcg \
    -display none \
    -serial stdio \
    -no-reboot \
    -nic user,model=virtio-net-pci \
    -cdrom "$image" \
    -kernel "$tmp/vmlinuz" \
    -initrd "$tmp/initrd.img" \
    -append "boot=live components quiet toram console=ttyS0,115200n8 adad.leak_battery=1" \
    >"$log" 2>&1 &
  pid=$!

  if elapsed="$(wait_for_markers "$log" "$pid" "$leak_timeout" \
    'adad-leak-battery: all: pass')"; then
    printf '%s\n' "$elapsed"
    return 0
  fi

  echo "ERROR: leak battery markers not observed for profile '$profile'." >&2
  tail -n 160 "$log" >&2 || true
  exit 1
}

measure_inference() {
  gguf="${ADAD_PERF_GGUF:-}"
  hf_model="${ADAD_PERF_HF_MODEL:-}"
  ready_timeout="${ADAD_LLAMA_READY_TIMEOUT:-300}"
  llama_server_bin="${ADAD_LLAMA_SERVER_BIN:-llama-server}"

  if [ -n "$hf_model" ]; then
    sh scripts/fetch-llama-cpp-runtime.sh >"$tmp_root/adad-min-sim-llama-runtime.log"
    llama_server_bin="build/tools/llama.cpp/${ADAD_LLAMA_CPP_RELEASE_TAG:-b9892}/llama-server"
  elif [ -z "$gguf" ]; then
    printf 'skipped:no-gguf\t\t\tno GGUF fixture or HF model supplied'
    return 0
  elif [ ! -f "$gguf" ]; then
    printf 'skipped:missing-gguf\t\t\tmissing GGUF fixture: %s' "$gguf"
    return 0
  fi

  if ! command -v "$llama_server_bin" >/dev/null 2>&1 && [ ! -x "$llama_server_bin" ]; then
    printf 'skipped:no-llama-server\t\t\tllama-server unavailable in builder'
    return 0
  fi

  server_log="$tmp_root/adad-min-sim-llama-${profile}.log"
  response="$tmp_root/adad-min-sim-llama-${profile}.json"
  rm -f "$server_log" "$response"

  if [ -n "$hf_model" ]; then
    HF_HOME=/workspace/build/hf-cache \
      "$llama_server_bin" -hf "$hf_model" --host 127.0.0.1 --port 8080 >"$server_log" 2>&1 &
  else
    "$llama_server_bin" --model "$gguf" --host 127.0.0.1 --port 8080 >"$server_log" 2>&1 &
  fi
  server_pid=$!
  trap 'cleanup_pid "$server_pid"' EXIT INT TERM

  ready=no
  tries=0
  while [ "$tries" -lt "$ready_timeout" ]; do
    tries=$((tries + 1))
    if curl -fsS http://127.0.0.1:8080/v1/chat/completions \
      -H 'Content-Type: application/json' \
      -d '{"model":"adad-perf-smoke","messages":[{"role":"user","content":"Write exactly the word ok."}]}' \
      >"$response" 2>/dev/null; then
      ready=yes
      break
    fi
    sleep 1
  done

  if [ "$ready" != yes ]; then
    note="llama-server did not answer within ${ready_timeout}s"
    if ! kill -0 "$server_pid" 2>/dev/null; then
      note="llama-server exited before ready; see $server_log"
    else
      note="$note; see $server_log"
    fi
    cleanup_pid "$server_pid"
    printf 'skipped:llama-server-not-ready\t\t\t%s' "$note"
    return 0
  fi

  start_ms="$(now_ms)"
  curl -fsS http://127.0.0.1:8080/v1/chat/completions \
    -H 'Content-Type: application/json' \
    -d '{"model":"adad-perf-smoke","messages":[{"role":"user","content":"Write exactly the word ok."}]}' \
    >"$response"
  elapsed_ms=$(( $(now_ms) - start_ms ))
  completion_tokens="$(sed -n 's/.*"completion_tokens"[[:space:]]*:[[:space:]]*\([0-9][0-9]*\).*/\1/p' "$response" | head -n 1)"
  cleanup_pid "$server_pid"

  if [ -z "$completion_tokens" ] || [ "$elapsed_ms" -le 0 ]; then
    printf 'skipped:parse-failed\t\t\tcould not parse completion_tokens from %s' "$response"
    return 0
  fi

  tok_s="$(awk -v tokens="$completion_tokens" -v elapsed="$elapsed_ms" 'BEGIN { printf "%.2f", (tokens * 1000) / elapsed }')"
  printf 'measured\t%s\t%s\tcompletion_tokens=%s' "$elapsed_ms" "$tok_s" "$completion_tokens"
}

boot_ms="$(measure_boot_ms)"
leak_ms="$(measure_leak_ms)"
inference_metrics="$(measure_inference)"

printf '%s\t%s\t%s\n' "$boot_ms" "$leak_ms" "$inference_metrics"
