#!/usr/bin/env bash
# The lane pool: persistent worktrees that outlive the streams they carry,
# so a stream starts on a warm target/ instead of a cold build (AGENTS.md
# §Build and gate discipline).
#
#   scripts/lane.sh take NAME [STREAM]   a free lane (or the one already on
#                                        STREAM) switched to a new branch
#                                        STREAM from main; prints its path
#   scripts/lane.sh free NAME            after the stream merged: the lane idle
#                                        (detached at main), the branch deleted
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
      [ "$b" != HEAD ] || b="(idle)"
      n="$(git -C "$d" status --porcelain | wc -l | tr -d ' ')"
      printf '%s  %s  (%s uncommitted)\n' "$d" "$b" "$n"
    done; exit 0 ;;
  take)
    name="${2:?lane name}"; stream="${3:-}"
    dir="$main-$name"
    if ! git -C "$main" worktree list --porcelain | grep -qx "worktree $dir"; then
      "$main/scripts/worktree.sh" "$name" >/dev/null
      git -C "$dir" switch -q --detach main 2>/dev/null || true
    fi
    if [ -n "$stream" ]; then
      cur="$(git -C "$dir" rev-parse --abbrev-ref HEAD)"
      # An idle lane sits detached at main (main itself is the main checkout's).
      if [ "$cur" != HEAD ] && [ "$cur" != "$stream" ]; then
        echo "lane.sh: $dir carries $cur; free it first" >&2; exit 1
      fi
      if [ "$cur" = HEAD ]; then
        git -C "$dir" switch -q -c "$stream" main
      fi
    fi
    echo "$dir" ;;
  free)
    name="${2:?lane name}"; dir="$main-$name"
    cur="$(git -C "$dir" rev-parse --abbrev-ref HEAD)"
    if [ "$cur" != HEAD ]; then
      git -C "$main" merge-base --is-ancestor "$cur" main || { echo "lane.sh: $cur is not merged into main" >&2; exit 1; }
    fi
    git -C "$dir" switch -q --detach main
    [ "$cur" = HEAD ] || git -C "$main" branch -q -d "$cur"
    echo "$dir idle at main" ;;
  *) sed -n '2,17p' "$0" | sed 's/^# \{0,1\}//'; exit 2 ;;
esac
