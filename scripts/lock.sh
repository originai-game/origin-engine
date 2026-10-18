# shellcheck shell=sh
# The gate lock: one scripts/check.sh, or one release build.sh, per target
# directory at a time (docs/ci.md §Running the gate locally). Two of them at
# once only wait on cargo's lock and thrash memory. Sourced by lib.sh and by
# packages/engine/scripts/build.sh, so it is POSIX sh.
#
#   og_lock_acquire TARGET_DIR WHAT   waits for the lock, then holds it until exit
#
# The lock is the directory TARGET_DIR/og-gate.lock (mkdir is atomic on every
# filesystem) holding the owner's "pid what". A holder that no longer runs
# (`kill -0` fails, or the pid now belongs to something that is not a
# check.sh or build.sh, as after CI restores a cached target/) is stale and
# is taken over. The owner exports
# OG_LOCK_OWNER, so a build.sh the gate runs does not wait on its own gate.

og_lock_acquire() {
  og_lock_dir="$1/og-gate.lock"
  og_lock_said=""
  og_lock_empty=0
  mkdir -p "$1" || return 1
  while :; do
    if mkdir "$og_lock_dir" 2>/dev/null; then
      printf '%s %s\n' "$$" "$2" > "$og_lock_dir/owner"
      OG_LOCK_OWNER=$$
      export OG_LOCK_OWNER
      trap 'og_lock_release' EXIT
      trap 'exit 130' INT
      trap 'exit 143' TERM
      return 0
    fi
    og_lock_holder="$(cat "$og_lock_dir/owner" 2>/dev/null || true)"
    og_lock_pid="${og_lock_holder%% *}"
    if [ -z "$og_lock_pid" ]; then
      # Created but not yet written; a holder that died in between is stale
      # after ten seconds.
      og_lock_empty=$((og_lock_empty + 1))
      if [ "$og_lock_empty" -ge 10 ]; then og_lock_steal; og_lock_empty=0; fi
      sleep 1
      continue
    fi
    og_lock_empty=0
    if [ -n "${OG_LOCK_OWNER:-}" ] && [ "$og_lock_pid" = "$OG_LOCK_OWNER" ]; then
      return 0   # held by the gate that runs us
    fi
    if ! og_lock_alive "$og_lock_pid"; then
      printf '%s\n' "gate lock: pid $og_lock_pid (${og_lock_holder#* }) no longer runs a gate; taking over $og_lock_dir" >&2
      og_lock_steal
      continue
    fi
    if [ "$og_lock_said" != "$og_lock_pid" ]; then
      printf '%s\n' "another gate (pid $og_lock_pid: ${og_lock_holder#* }) is running on $1, waiting" >&2
      og_lock_said="$og_lock_pid"
    fi
    sleep 2
  done
}

og_lock_alive() {
  kill -0 "$1" 2>/dev/null || return 1
  command -v ps >/dev/null 2>&1 || return 0
  case "$(ps -p "$1" -o command= 2>/dev/null)" in
    *check.sh*|*build.sh*|*pack.sh*) return 0 ;;
    *) return 1 ;;
  esac
}

# Renaming first means two waiters that both saw the stale owner remove it
# once: the second rename fails and that waiter simply tries mkdir again.
og_lock_steal() {
  if mv "$og_lock_dir" "$og_lock_dir.stale.$$" 2>/dev/null; then
    rm -rf "$og_lock_dir.stale.$$"
  fi
}

og_lock_release() {
  if [ -n "${og_lock_dir:-}" ]; then
    og_lock_holder="$(cat "$og_lock_dir/owner" 2>/dev/null || true)"
    [ "${og_lock_holder%% *}" = "$$" ] && rm -rf "$og_lock_dir"
  fi
  return 0
}
