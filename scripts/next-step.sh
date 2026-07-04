#!/usr/bin/env sh
# Prints the path of the next ExecPlan to run, or DONE, or BLOCKED:<path>.
# Exit codes: 0 = plan printed, 10 = DONE, 20 = blocked, 1 = error.
set -eu
cd "$(dirname "$0")/.."

get_field() { # file key
  sed -n '/^---$/,/^---$/p' "$1" | grep "^$2:" | head -n1 | cut -d: -f2- | tr -d ' []' ;
}

status_of() { # EP id -> status
  f=$(ls .agent/execplans/"$1"-*.md 2>/dev/null | head -n1) || return 1
  [ -n "$f" ] || { echo "unknown"; return 0; }
  get_field "$f" status
}

for f in .agent/execplans/EP-*.md; do
  st=$(get_field "$f" status)
  [ "$st" = "complete" ] && continue
  if [ "$st" = "blocked" ]; then
    echo "BLOCKED:$f"
    exit 20
  fi
  deps=$(get_field "$f" depends_on)
  ready=yes
  if [ -n "$deps" ]; then
    old_ifs=$IFS; IFS=,
    for d in $deps; do
      [ "$(status_of "$d")" = "complete" ] || ready=no
    done
    IFS=$old_ifs
  fi
  if [ "$ready" = "yes" ]; then
    echo "$f"
    exit 0
  fi
done
echo "DONE"
exit 10
