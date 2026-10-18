#!/usr/bin/env bash
# A persistent Git worktree for a parallel round (AGENTS.md §Build and gate
# discipline).
#
#   scripts/worktree.sh NAME     create or reuse <main checkout>-NAME
#                                (/Volumes/app/origin-engine-NAME here) on branch NAME
#   scripts/worktree.sh --list   the worktrees and their branches
#
# Created: branch NAME from main (or the existing branch NAME), checked out
# beside the main checkout. Reused: left exactly as it is (its branch, its
# changes, its target/), with how far it is behind main. Its target/ is its
# own and stays: the gate never deletes it, and a crate is cleaned with
# `cargo clean -p <crate>`, never wholesale. sccache (.cargo/config.toml) is
# shared, so a new worktree's first test build takes the main checkout's
# dependencies from the cache (about a minute, docs/ci.md §Lanes) and every
# later one is incremental. Remove one with `git worktree remove <path>`
# when its stream of work is done for good.
set -eu -o pipefail

case "${1:-}" in
  "" | -h | --help) sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//'; exit 2 ;;
  --list) git worktree list; exit 0 ;;
esac
name="$1"
case "$name" in
  -* | *..* | *[[:space:]]*) echo "worktree.sh: '$name' is not a usable branch name" >&2; exit 2 ;;
esac

# The main checkout, also when this runs inside another worktree.
main="$(git worktree list --porcelain | awk '/^worktree / { print substr($0, 10); exit }')"
dir="$main-$(printf '%s' "$name" | tr '/' '-')"

if git worktree list --porcelain | grep -qx "worktree $dir"; then
  branch="$(git -C "$dir" rev-parse --abbrev-ref HEAD)"
  behind="$(git -C "$dir" rev-list --count HEAD..main 2>/dev/null || echo "?")"
  echo "worktree.sh: reusing $dir (branch $branch, $behind commit(s) behind main; target/ kept)"
elif [ -e "$dir" ]; then
  echo "worktree.sh: $dir exists but is not a worktree of this repository; move it away first" >&2
  exit 1
elif git -C "$main" show-ref --verify --quiet "refs/heads/$name"; then
  git -C "$main" worktree add "$dir" "$name"
  echo "worktree.sh: created $dir on the existing branch $name"
else
  git -C "$main" worktree add -b "$name" "$dir" main
  echo "worktree.sh: created $dir on a new branch $name from main"
fi
echo "$dir"
