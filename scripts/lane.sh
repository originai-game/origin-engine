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
#
# A detached HEAD alone never means idle: a stream mid-bisect or on a
# detached checkout is busy. Idle is the `lane-idle` mark that `free` (and
# a new lane) leaves in the worktree's git dir and `take` removes; a lane
# freed before the mark existed counts as idle only when its last HEAD move
# was the detach to main, and it is clean and not bisecting, rebasing or
# merging.
set -eu -o pipefail
main="$(git worktree list --porcelain | awk '/^worktree / { print substr($0, 10); exit }')"
lanes() { git -C "$main" worktree list --porcelain | awk '/^worktree / { print substr($0, 10) }' | grep -- "-lane-" || true; }
mark() { echo "$(git -C "$1" rev-parse --absolute-git-dir)/lane-idle"; }
idle() {
  local d="$1" g
  [ "$(git -C "$d" rev-parse --abbrev-ref HEAD)" = HEAD ] || return 1
  g="$(git -C "$d" rev-parse --absolute-git-dir)"
  [ ! -e "$g/BISECT_LOG" ] && [ ! -d "$g/rebase-merge" ] && [ ! -d "$g/rebase-apply" ] && [ ! -e "$g/MERGE_HEAD" ] || return 1
  [ -z "$(git -C "$d" status --porcelain)" ] || return 1
  [ ! -d "$g/lane-claim" ] || return 1
  [ -e "$g/lane-idle" ] || git -C "$d" reflog -1 --format=%gs HEAD | grep -q ' to main$'
}
case "${1:-}" in
  --list)
    for d in $(lanes); do
      b="$(git -C "$d" rev-parse --abbrev-ref HEAD)"
      if [ "$b" = HEAD ]; then
        if idle "$d"; then b="(idle)"; else b="(busy: detached at $(git -C "$d" rev-parse --short HEAD))"; fi
      fi
      n="$(git -C "$d" status --porcelain | wc -l | tr -d ' ')"
      printf '%s  %s  (%s uncommitted)\n' "$d" "$b" "$n"
    done; exit 0 ;;
  take)
    name="${2:?lane name}"; stream="${3:-}"
    dir="$main-$name"
    if ! git -C "$main" worktree list --porcelain | grep -qx "worktree $dir"; then
      "$main/scripts/worktree.sh" "$name" >/dev/null
      git -C "$dir" switch -q --detach main 2>/dev/null || true
      touch "$(mark "$dir")"
    fi
    if [ -n "$stream" ]; then
      cur="$(git -C "$dir" rev-parse --abbrev-ref HEAD)"
      if [ "$cur" != HEAD ] && [ "$cur" != "$stream" ]; then
        echo "lane.sh: $dir carries $cur; free it first" >&2; exit 1
      fi
      if [ "$cur" = HEAD ]; then
        # Detached is not idle: a stream may be bisecting or on a detached checkout.
        idle "$dir" || { echo "lane.sh: $dir is detached at $(git -C "$dir" rev-parse --short HEAD) but not idle (bisect, rebase, uncommitted work, or never freed); take another lane" >&2; exit 1; }
        # Claim atomically (mkdir), so two sessions taking the same idle lane at
        # once cannot both switch it: the second one is refused here.
        claim="$(git -C "$dir" rev-parse --absolute-git-dir)/lane-claim"
        mkdir "$claim" 2>/dev/null || { echo "lane.sh: $dir is being taken by another session; take another lane" >&2; exit 1; }
        echo "$$ $stream $(date +%s)" > "$claim/owner"
        git -C "$dir" switch -q -c "$stream" main
        rm -f "$(mark "$dir")"
      fi
    fi
    echo "$dir" ;;
  free)
    name="${2:?lane name}"; dir="$main-$name"
    cur="$(git -C "$dir" rev-parse --abbrev-ref HEAD)"
    if [ "$cur" != HEAD ]; then
      git -C "$main" merge-base --is-ancestor "$cur" main || { echo "lane.sh: $cur is not merged into main" >&2; exit 1; }
    fi
    if [ "$cur" = HEAD ] && ! idle "$dir"; then
      g="$(git -C "$dir" rev-parse --absolute-git-dir)"
      if [ -e "$g/BISECT_LOG" ] || [ -d "$g/rebase-merge" ] || [ -d "$g/rebase-apply" ] || [ -n "$(git -C "$dir" status --porcelain)" ]; then
        echo "lane.sh: $dir is detached mid-bisect, mid-rebase or with uncommitted work; finish it first" >&2; exit 1
      fi
    fi
    git -C "$dir" switch -q --detach main
    rm -rf "$(git -C "$dir" rev-parse --absolute-git-dir)/lane-claim"
    [ "$cur" = HEAD ] || git -C "$main" branch -q -d "$cur"
    touch "$(mark "$dir")"
    echo "$dir idle at main" ;;
  *) sed -n '2,17p' "$0" | sed 's/^# \{0,1\}//'; exit 2 ;;
esac
