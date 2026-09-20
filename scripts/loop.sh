#!/usr/bin/env sh
# Unattended build loop. Requires AGENT_CMD, e.g.:
#   AGENT_CMD='codex --cd . --ask-for-approval never --sandbox workspace-write' \
#     scripts/loop.sh
# AGENT_CMD must accept a single prompt string as its final argument.
set -eu
cd "$(dirname "$0")/.."

: "${AGENT_CMD:?ERROR: set AGENT_CMD to your coding-agent CLI invocation}"
: "${MAX_ITERATIONS:=100}"
STATE=.agent/state/build-state.env
RESULT=.agent/state/last-result.env

read_kv() { grep "^$2=" "$1" | head -n1 | cut -d= -f2-; }

i=0
fail_streak=0
last_fail_key=""

while [ "$i" -lt "$MAX_ITERATIONS" ]; do
  i=$((i + 1))
  set +e
  next=$(scripts/next-step.sh); next_rc=$?
  set -e
  case "$next_rc" in
    10)
      scripts/verify.sh
      scripts/production-readiness-check.sh
      echo "build: complete"
      exit 0 ;;
    20)
      echo "loop: halted - $next" >&2
      echo "loop: see .agent/state/blockers.md, fill Resolution, set plan" >&2
      echo "loop: status back to in-progress, then rerun scripts/loop.sh" >&2
      exit 3 ;;
    0) : ;;
    *) echo "loop: next-step.sh error" >&2; exit 1 ;;
  esac

  echo "loop: iteration $i - $next"
  prompt=$(sed "s|\[EXECPLAN_PATH\]|$next|g" .agent/prompts/loop-iteration.md)

  set +e
  $AGENT_CMD "$prompt"
  agent_rc=$?
  set -e

  if [ ! -f "$RESULT" ]; then
    echo "loop: session ended without last-result.env (rc=$agent_rc)" >&2
    result=failed; ep=$next; ms=unknown
  else
    result=$(read_kv "$RESULT" RESULT)
    ep=$(read_kv "$RESULT" EXECPLAN)
    ms=$(read_kv "$RESULT" MILESTONE)
  fi

  case "$result" in
    plan_complete|in_progress)
      fail_streak=0; last_fail_key="" ;;
    stop|blocked)
      echo "loop: halted by agent ($result) on $ep $ms" >&2
      sed -n '/OPEN/,$p' .agent/state/blockers.md >&2 || true
      exit 3 ;;
    *)
      key="$ep:$ms"
      if [ "$key" = "$last_fail_key" ]; then
        fail_streak=$((fail_streak + 1))
      else
        fail_streak=1; last_fail_key=$key
      fi
      if [ "$fail_streak" -ge 3 ]; then
        echo "loop: 3 consecutive failed sessions on $key - halting" >&2
        echo "loop: mark plan blocked, record blocker, get human input" >&2
        exit 4
      fi ;;
  esac

  {
    echo "ITERATION=$i"
    echo "LAST_EXECPLAN=$ep"
    echo "LAST_MILESTONE=$ms"
    echo "CONSECUTIVE_FAILURES=$fail_streak"
  } > "$STATE"
done

echo "loop: iteration budget ($MAX_ITERATIONS) exhausted - halting" >&2
exit 5
