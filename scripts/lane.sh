#!/usr/bin/env bash
# The lane pool: persistent worktrees that outlive the streams they carry,
# so a stream starts on a warm target/ instead of a cold build (AGENTS.md
# §Build and gate discipline).
#
#   scripts/lane.sh take NAME [STREAM]   a free lane (or the one already on
#                                        STREAM) switched to a new branch
#                                        STREAM from main; prints its path
#   scripts/lane.sh free NAME            after the stream merged: the lane back
#                                        on main, the stream branch deleted
#   scripts/lane.sh --list               every lane and the branch it carries
#
# A lane is a worktree /Volumes/app/origin-engine-lane-N made once by
# scripts/worktree.sh (its target/ stays); NAME is lane-N or any existing
# worktree's name. With STREAM omitted, `take` only reports the lane.
set -eu -o pipefail
main="$(git worktree list --porcelain | awk '/^worktree / { print substr($0, 10); exit }')"
lanes() { git -C "$main" worktree list --porcelain | awk '/^worktree / { print substr($0, 10) }' | grep -- "-lane-" || true; }
case "${1:-}" in
  --list)
    for d in $(lanes); do
      b="$(git -C "$d" rev-parse --abbrev-ref HEAD)"
      n="$(git -C "$d" status --porcelain | wc -l | tr -d ' ')"
      printf '%s  %s  (%s uncommitted)\n' "$d" "$b" "$n"
    done; exit 0 ;;
  take)
    name="${2:?lane name}"; stream="${3:-}"
    dir="$main-$name"
    if ! git -C "$main" worktree list --porcelain | grep -qx "worktree $dir"; then
      "$main/scripts/worktree.sh" "$name" >/dev/null
      git -C "$dir" switch -q main 2>/dev/null || true
    fi
    if [ -n "$stream" ]; then
      cur="$(git -C "$dir" rev-parse --abbrev-ref HEAD)"
      if [ "$cur" != main ] && [ "$cur" != "$stream" ]; then
        echo "lane.sh: $dir carries $cur; free it first" >&2; exit 1
      fi
      if [ "$cur" = main ]; then
        git -C "$dir" switch -q -c "$stream" main
      fi
    fi
    echo "$dir" ;;
  free)
    name="${2:?lane name}"; dir="$main-$name"
    cur="$(git -C "$dir" rev-parse --abbrev-ref HEAD)"
    if [ "$cur" != main ]; then
      git -C "$main" merge-base --is-ancestor "$cur" main || { echo "lane.sh: $cur is not merged into main" >&2; exit 1; }
      git -C "$dir" switch -q main && git -C "$main" branch -q -d "$cur"
    fi
    git -C "$dir" merge -q --ff-only main 2>/dev/null || true
    echo "$dir on main" ;;
  *) sed -n '2,17p' "$0" | sed 's/^# \{0,1\}//'; exit 2 ;;
esac
