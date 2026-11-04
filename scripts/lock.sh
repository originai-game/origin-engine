# shellcheck shell=sh
# The gate lock: one scripts/check.sh, or one release build.sh, per target
# directory at a time (docs/ci.md §Running the gate locally;
# scripts/wave-end.sh holds it across its stages). Two of them at
# once only wait on cargo's lock and thrash memory. Sourced by lib.sh and by
# packages/engine/scripts/build.sh, so it is POSIX sh.
#
#   og_lock_acquire TARGET_DIR WHAT   waits for the lock, then holds it until exit
#
# A process may hold more than one (the gate holds the machine lock, then the
# target's): og_lock_dirs lists them and og_lock_release frees every one it
# owns.
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
      og_lock_dirs="${og_lock_dirs:-}${og_lock_dirs:+ }$og_lock_dir"
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
    *check.sh*|*build.sh*|*pack.sh*|*wave-end.sh*) return 0 ;;
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
  for og_lock_held in ${og_lock_dirs:-}; do
    og_lock_holder="$(cat "$og_lock_held/owner" 2>/dev/null || true)"
    [ "${og_lock_holder%% *}" = "$$" ] && rm -rf "$og_lock_held"
  done
  og_lock_dirs=""
  return 0
}

# GPU slots (the owner, 2026-11-04): Metal on macrunner runs a few GPU suites
# at once, so scoped GPU work (an area's og-tests, `og run --commands`
# frames, a sheet render) takes one of OG_GPU_SLOTS slots (default 3)
# instead of the whole machine. Reference pixels do not depend on what else
# runs; frame-time and perf measurements do, so they take every slot.
#
#   og_gpu_acquire WHAT            one slot, held until exit
#   og_gpu_acquire_all WHAT        every slot (perf, wave-end, the full gate)
#
# Slots are ~/.cache/og-gate/gpu-slot-N.lock directories, taken lowest first;
# a slot whose owner pid is gone is stale and taken over. Released by
# og_lock_release with the other locks.

og_gpu_root="${OG_GPU_ROOT:-$HOME/.cache/og-gate}"

# One implementation for the whole machine (the owner, 2026-11-04): every
# lane sources its own scripts/lock.sh, so a lane on an old commit (or a
# wrapper with a private copy) used to queue by old rules beside the new
# ones. The slot functions below are written to $og_gpu_root/gpu-lock.sh
# whenever this copy's og_gpu_lock_version is newer than the one there, and
# everyone sources that file, so the newest version on the machine wins for
# every lane at once. Bump og_gpu_lock_version with any change below.
og_gpu_impl() {
  cat <<'OG_GPU_IMPL'
og_gpu_lock_version=2

og_gpu_try() { # og_gpu_try N WHAT: take slot N now or fail
  og_gpu_slot="$og_gpu_root/gpu-slot-$1.lock"
  if mkdir "$og_gpu_slot" 2>/dev/null; then
    printf '%s %s\n' "$$" "$2" > "$og_gpu_slot/owner"
    og_lock_dirs="${og_lock_dirs:-}${og_lock_dirs:+ }$og_gpu_slot"
    trap 'og_lock_release' EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    return 0
  fi
  og_gpu_pid="$(cut -d' ' -f1 "$og_gpu_slot/owner" 2>/dev/null || true)"
  if [ -n "$og_gpu_pid" ] && ! kill -0 "$og_gpu_pid" 2>/dev/null; then
    if mv "$og_gpu_slot" "$og_gpu_slot.stale.$$" 2>/dev/null; then rm -rf "$og_gpu_slot.stale.$$"; fi
  fi
  return 1
}

og_gpu_acquire() {
  mkdir -p "$og_gpu_root/gpu-wait" || return 1
  og_gpu_said=""
  # A waiting mark, so an og_gpu_acquire_all that starts after us lets us in
  # first instead of re-taking every slot the moment it frees them.
  : > "$og_gpu_root/gpu-wait/$$"
  while :; do
    # An exclusive holder (og_gpu_acquire_all) blocks new single slots, so it is not starved.
    if [ ! -d "$og_gpu_root/gpu-all.lock" ] || [ "$(cut -d' ' -f1 "$og_gpu_root/gpu-all.lock/owner" 2>/dev/null)" = "$$" ]; then
      og_gpu_n=1
      while [ "$og_gpu_n" -le "${OG_GPU_SLOTS:-3}" ]; do
        if og_gpu_try "$og_gpu_n" "$1"; then
          rm -f "$og_gpu_root/gpu-wait/$$"
          return 0
        fi
        og_gpu_n=$((og_gpu_n + 1))
      done
    elif ! kill -0 "$(cut -d' ' -f1 "$og_gpu_root/gpu-all.lock/owner" 2>/dev/null || echo 0)" 2>/dev/null; then
      rm -rf "$og_gpu_root/gpu-all.lock"
    fi
    if [ -z "$og_gpu_said" ]; then
      printf '%s\n' "gpu: all ${OG_GPU_SLOTS:-3} slots busy, waiting ($1)" >&2
      og_gpu_said=1
    fi
    sleep 2
  done
}

og_gpu_acquire_all() {
  mkdir -p "$og_gpu_root/gpu-wait" || return 1
  # Single waiters already queued when we arrive go first (each waits for
  # a slot only once, so this is bounded); later ones queue behind us.
  for og_gpu_w in "$og_gpu_root"/gpu-wait/*; do
    [ -e "$og_gpu_w" ] || continue
    og_gpu_wpid="${og_gpu_w##*/}"
    if ! kill -0 "$og_gpu_wpid" 2>/dev/null; then rm -f "$og_gpu_w"; continue; fi
    while [ -e "$og_gpu_w" ] && kill -0 "$og_gpu_wpid" 2>/dev/null; do sleep 1; done
    rm -f "$og_gpu_w" 2>/dev/null || true
  done
  while ! mkdir "$og_gpu_root/gpu-all.lock" 2>/dev/null; do
    og_gpu_pid="$(cut -d' ' -f1 "$og_gpu_root/gpu-all.lock/owner" 2>/dev/null || true)"
    if [ -n "$og_gpu_pid" ] && ! kill -0 "$og_gpu_pid" 2>/dev/null; then rm -rf "$og_gpu_root/gpu-all.lock"; fi
    sleep 2
  done
  printf '%s %s\n' "$$" "$1" > "$og_gpu_root/gpu-all.lock/owner"
  og_lock_dirs="${og_lock_dirs:-}${og_lock_dirs:+ }$og_gpu_root/gpu-all.lock"
  og_gpu_n=1
  while [ "$og_gpu_n" -le "${OG_GPU_SLOTS:-3}" ]; do
    until og_gpu_try "$og_gpu_n" "$1"; do sleep 2; done
    og_gpu_n=$((og_gpu_n + 1))
  done
}
OG_GPU_IMPL
}

og_gpu_machine="$og_gpu_root/gpu-lock.sh"
og_gpu_mine="$(og_gpu_impl | sed -n 's/^og_gpu_lock_version=//p')"
og_gpu_theirs="$(sed -n 's/^og_gpu_lock_version=//p' "$og_gpu_machine" 2>/dev/null | head -n 1)"
if [ "${og_gpu_theirs:-0}" -lt "$og_gpu_mine" ] && mkdir -p "$og_gpu_root" 2>/dev/null; then
  og_gpu_impl > "$og_gpu_machine.$$" && mv -f "$og_gpu_machine.$$" "$og_gpu_machine"
fi
if [ -f "$og_gpu_machine" ]; then
  # shellcheck disable=SC1090
  . "$og_gpu_machine"
else
  eval "$(og_gpu_impl)"
fi
