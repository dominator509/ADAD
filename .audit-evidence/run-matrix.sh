#!/usr/bin/env bash
# ADAD final matrix runner — captures command line, combined stdout+stderr,
# exit code, and env header for each executable check, writing evidence under
# .audit-evidence/final/. Mirrors the baseline harness (task t_bf7d4ee4) so the
# two runs are directly comparable.
set -u

cd /root/ADAD || exit 1
export RUSTUP_HOME=/root/.rustup
export CARGO_HOME=/root/.cargo
export PATH=/root/.cargo/bin:$PATH

FINAL_DIR=/root/ADAD/.audit-evidence/final
mkdir -p "$FINAL_DIR"

REPO_HEAD=$(/usr/bin/git rev-parse HEAD)
LOOP_DEVS=$(ls /dev/loop* 2>/dev/null | tr '\n' ' ')
CAP=$(grep CapEff /proc/self/status 2>/dev/null | awk '{print $2}')
GIT_STATUS_N=$(/usr/bin/git status --porcelain | wc -l)

# Each entry: log_filename|label|command (run via eval to preserve args)
run_check() {
  local log="$1"; shift
  local label="$1"; shift
  local cmdline="$1"; shift
  local logpath="$FINAL_DIR/$log"

  {
    echo "================================================================================"
    echo "ADAD FINAL EVIDENCE  (task t_bd3bc731 — fix + rerun)"
    echo "log:            $logpath"
    echo "started:        $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "command_line:   $cmdline"
    echo "working_dir:    /root/ADAD"
    echo "shell:          bash (real cargo via /root/.cargo/bin; scripts via /bin/sh)"
    echo "env:            RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo"
    echo "host:           $(uname -srm)  $(uname -o 2>/dev/null)  UTC"
    echo "repo_head:      $REPO_HEAD"
    echo "git_status:     $GIT_STATUS_N modified tracked paths"
    echo "loop_devices:   $LOOP_DEVS"
    echo "cap_eff:        ${CAP:-n/a}"
    echo "================================================================================ "
  } > "$logpath"

  local start_epoch end_epoch
  start_epoch=$(date +%s)
  # shellcheck disable=SC2086
  eval "$cmdline" >> "$logpath" 2>&1
  local code=$?
  end_epoch=$(date +%s)

  {
    echo "================================================================================ "
    echo "exit_code:      $code"
    echo "finished:       $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "wall_seconds:   $((end_epoch - start_epoch))"
    echo "================================================================================ "
  } >> "$logpath"

  echo "[$label] exit=$code  ->  $logpath"
  return 0
}

run_check "01-cargo-build-locked.log" "build-locked" "/root/.cargo/bin/cargo build --locked"
run_check "02-cargo-test-workspace-locked.log" "test-workspace" "/root/.cargo/bin/cargo test --workspace --locked"
run_check "03-cargo-fmt-check.log" "fmt-check" "/root/.cargo/bin/cargo fmt --all -- --check"
run_check "04-scripts-build-sh.log" "scripts-build" "/bin/sh scripts/build.sh"
run_check "05-scripts-lint-sh.log" "scripts-lint" "/bin/sh scripts/lint.sh"
run_check "06-scripts-format-check-sh.log" "scripts-format-check" "/bin/sh scripts/format-check.sh"
run_check "07-scripts-dependency-audit-sh.log" "scripts-dependency-audit" "/bin/sh scripts/dependency-audit.sh"

echo "=== FINAL GIT STATUS ==="
/usr/bin/git status --porcelain
echo "=== done ==="
