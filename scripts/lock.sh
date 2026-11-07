# shellcheck shell=sh
# The gate lock: one scripts/check.sh, or one release build.sh, per target
# directory at a time (docs/ci.md §Running the gate locally). Two of them at
# once only wait on cargo's lock and thrash memory. Sourced by lib.sh and by
# packages/engine/scripts/build.sh, so it is POSIX sh.
#
#   og_lock_acquire TARGET_DIR WHAT   waits for the lock, then holds it until exit
#
# A process may hold more than one: og_lock_dirs lists them and
# og_lock_release frees every one it
# owns.
#
# The lock is the directory TARGET_DIR/og-gate.lock (mkdir is atomic on every
# filesystem) holding the owner's "pid what" and process-birth identity.
# A holder whose process died or whose PID was reused is stale and
# is taken over. The owner exports
# OG_LOCK_OWNER, so a build.sh the gate runs does not wait on its own gate.

og_gpu_root="${OG_GPU_ROOT:-$HOME/.cache/og-gate}"
og_gpu_impl() {
  cat <<'OG_GPU_IMPL'
og_gpu_lock_version=6

# Process birth, not command text, identifies an owner after PID reuse.
# Keep the human-readable owner file compatible with older lanes.
og_process_identity() {
  kill -0 "$1" 2>/dev/null || return 1
  case "$(ps -p "$1" -o stat= 2>/dev/null)" in *Z*) return 1 ;; esac
  og_birth="$(LC_ALL=C ps -p "$1" -o lstart= 2>/dev/null | tr -d '[:space:]')"
  # Git Bash and Cygwin (studio-win) have no `ps -o`: their /proc/PID/stat
  # gives the state and the start time (fields 3 and 22, after the
  # parenthesised command). Where ps answers (macOS, Linux) it is not read.
  if [ -z "$og_birth" ] && [ -r "/proc/$1/stat" ]; then
    og_stat="$(sed 's/^.*) //' "/proc/$1/stat" 2>/dev/null)"
    case "$og_stat" in Z*) return 1 ;; esac
    og_birth="$(printf '%s\n' "$og_stat" | cut -d' ' -f20)"
  fi
  [ -n "$og_birth" ] || return 1
  printf '%s:%s\n' "$1" "$og_birth"
}

og_owner_write() {
  og_process_identity "$$" > "$1/identity" || return 1
  printf '%s %s\n' "$$" "$2" > "$1/owner"
}

og_owner_alive() {
  [ -f "$1/owner" ] || return 1
  og_owner_pid="$(cut -d' ' -f1 "$1/owner" 2>/dev/null)"
  og_owner_current="$(og_process_identity "$og_owner_pid")" || return 1
  # Existing holders from before this version have no birth record. Do
  # not steal their live locks merely because their command is unfamiliar.
  [ ! -f "$1/identity" ] || [ "$(cat "$1/identity")" = "$og_owner_current" ]
}

og_owner_inherited() {
  og_owner_alive "$1" || return 1
  if [ -f "$1/identity" ]; then
    case " ${OG_LOCK_IDENTITIES:-} " in *" $(cat "$1/identity") "*) return 0 ;; esac
  fi
  [ "$(cut -d' ' -f1 "$1/owner" 2>/dev/null)" = "${OG_LOCK_OWNER:-}" ]
}

# Ordinary gates register briefly, then run concurrently. Measurements hold
# admission closed while existing gates drain. Call from a process, not a
# shell subshell: POSIX $$ identifies the enclosing shell in a subshell.
og_admission_acquire() {
  case "$1" in shared|exclusive) ;; *) return 2 ;; esac
  og_admission_root="$og_gpu_root/admission"
  if [ -n "${OG_ADMISSION_OWNER:-}" ]; then
    if og_owner_alive "$og_admission_root/active/$OG_ADMISSION_OWNER"; then
      [ "$1" = shared ] || { echo 'cannot promote an active gate to an exclusive measurement' >&2; return 1; }
      return 0
    fi
    if [ "$(cut -d' ' -f1 "$og_admission_root/og-gate.lock/owner" 2>/dev/null)" = "$OG_ADMISSION_OWNER" ] && og_owner_alive "$og_admission_root/og-gate.lock"; then
      return 0
    fi
  fi
  og_lock_acquire "$og_admission_root" "$2" || return 1
  mkdir -p "$og_admission_root/active" || return 1
  if [ "$1" = shared ]; then
    mkdir "$og_admission_root/active/$$" || return 1
    og_owner_write "$og_admission_root/active/$$" "$2" || return 1
    og_lock_dirs="$og_lock_dirs $og_admission_root/active/$$"
    rm -rf "$og_admission_root/og-gate.lock"
  else
    while :; do
      og_admission_readers=0
      for og_admission_active in "$og_admission_root/active/"*; do
        [ -d "$og_admission_active" ] || continue
        og_admission_pid="${og_admission_active##*/}"
        if og_owner_alive "$og_admission_active"; then
          og_admission_readers=$((og_admission_readers + 1))
        else
          rm -rf "$og_admission_active"
        fi
      done
      [ "$og_admission_readers" -gt 0 ] || break
      printf 'measurement admission: draining %s active gate(s)\n' "$og_admission_readers" >&2
      sleep 1
    done
  fi
  OG_ADMISSION_OWNER=$$
  export OG_ADMISSION_OWNER
}

# Ticket registration holds the same mkdir lock only long enough to advance
# the counter and publish a ticket. Waiting for the resource never holds it.
og_queue_join() {
  og_queue_internal=1
  og_lock_acquire "$1" "queue registration: $2" || return 1
  og_queue_internal=0
  og_queue_number="$(cat "$1/next" 2>/dev/null || echo 0)"
  og_queue_number=$((og_queue_number + 1))
  printf '%s\n' "$og_queue_number" > "$1/next.$$"
  mv -f "$1/next.$$" "$1/next"
  og_queue_ticket="$1/$(printf '%012d' "$og_queue_number").ticket"
  og_process_identity "$$" > "$og_queue_ticket.identity" || return 1
  printf '%s %s\n' "$$" "$2" > "$og_queue_ticket.$$"
  mv -f "$og_queue_ticket.$$" "$og_queue_ticket"
  og_queue_files="${og_queue_files:-}${og_queue_files:+ }$og_queue_ticket"
  rm -rf "$1/og-gate.lock"
  og_queue_said=""
}

og_queue_turn() {
  og_queue_position=0
  for og_queue_file in "${og_queue_ticket%/*}/"*.ticket; do
    [ -f "$og_queue_file" ] || continue
    og_queue_pid="$(cut -d' ' -f1 "$og_queue_file" 2>/dev/null || true)"
    [ -n "$og_queue_pid" ] || continue
    og_queue_current="$(og_process_identity "$og_queue_pid" || true)"
    if [ -z "$og_queue_current" ] || { [ -f "$og_queue_file.identity" ] && [ "$(cat "$og_queue_file.identity")" != "$og_queue_current" ]; }; then
      rm -f "$og_queue_file" "$og_queue_file.identity"
      continue
    fi
    og_queue_position=$((og_queue_position + 1))
    [ "$og_queue_file" != "$og_queue_ticket" ] || break
  done
  if [ "$og_queue_said" != "$og_queue_position" ]; then
    printf 'lock queue: position %s (%s)\n' "$og_queue_position" "$og_queue_ticket" >&2
    og_queue_said="$og_queue_position"
  fi
  [ "$og_queue_position" -eq 1 ]
}

og_queue_leave() { rm -f "$og_queue_ticket" "$og_queue_ticket.identity"; }

og_lock_acquire() {
  og_lock_holder="$(cat "$1/og-gate.lock/owner" 2>/dev/null || true)"
  if og_owner_inherited "$1/og-gate.lock"; then
    return 0
  fi
  if [ "${og_queue_internal:-0}" != 1 ]; then og_queue_join "$1/og-wait" "$2" || return 1; fi
  og_lock_dir="$1/og-gate.lock"
  og_lock_said=""
  og_lock_empty=0
  mkdir -p "$1" || return 1
  while :; do
    if [ "${og_queue_internal:-0}" != 1 ] && ! og_queue_turn; then sleep 1; continue; fi
    if mkdir "$og_lock_dir" 2>/dev/null; then
      og_owner_write "$og_lock_dir" "$2" || return 1
      og_lock_dirs="${og_lock_dirs:-}${og_lock_dirs:+ }$og_lock_dir"
      OG_LOCK_OWNER=$$
      OG_LOCK_IDENTITIES="${OG_LOCK_IDENTITIES:-} $(cat "$og_lock_dir/identity")"
      export OG_LOCK_OWNER OG_LOCK_IDENTITIES
      trap 'og_lock_release' EXIT
      trap 'exit 130' INT
      trap 'exit 143' TERM
      if [ "${og_queue_internal:-0}" != 1 ]; then og_queue_leave; fi
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
    if og_owner_inherited "$og_lock_dir"; then
      return 0   # held by the gate that runs us
    fi
    if ! og_owner_alive "$og_lock_dir"; then
      printf '%s\n' "gate lock: owner $og_lock_pid (${og_lock_holder#* }) is gone; taking over $og_lock_dir" >&2
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

# Renaming first means two waiters that both saw the stale owner remove it
# once: the second rename fails and that waiter simply tries mkdir again.
og_lock_steal() {
  if mv "$og_lock_dir" "$og_lock_dir.stale.$$" 2>/dev/null; then
    rm -rf "$og_lock_dir.stale.$$"
  fi
}

og_lock_release() {
  for og_queue_file in ${og_queue_files:-}; do
    [ "$(cut -d' ' -f1 "$og_queue_file" 2>/dev/null)" != "$$" ] || rm -f "$og_queue_file" "$og_queue_file.identity"
  done
  og_queue_files=""
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
#   og_gpu_acquire_all WHAT        every slot (perf, the full gate)
#
# Slots are ~/.cache/og-gate/gpu-slot-N.lock directories, taken lowest first;
# a slot whose owner pid is gone is stale and taken over. Released by
# og_lock_release with the other locks.

# One implementation for the whole machine (the owner, 2026-11-04): every
# lane sources its own scripts/lock.sh, so a lane on an old commit (or a
# wrapper with a private copy) used to queue by old rules beside the new
# ones. The gate, queue and slot functions are written to $og_gpu_root/gpu-lock.sh
# whenever this copy's og_gpu_lock_version is newer than the one there, and
# everyone sources that file, so the newest version on the machine wins for
# every newly started caller. Already waiting shells retain their functions.
# Bump og_gpu_lock_version with any change in this implementation.
og_gpu_dead() {
  og_gpu_pid="$(cut -d' ' -f1 "$1/owner" 2>/dev/null || true)"
  if [ -n "$og_gpu_pid" ]; then ! og_owner_alive "$1"; return; fi
  # A process can die between mkdir and publishing its owner.
  if [ "$(uname)" = Darwin ]; then
    og_gpu_created="$(stat -f %m "$1" 2>/dev/null)" || return 1
  else
    og_gpu_created="$(stat -c %Y "$1" 2>/dev/null)" || return 1
  fi
  [ "$(( $(date +%s) - og_gpu_created ))" -ge 10 ]
}

og_gpu_try() { # og_gpu_try N WHAT: take slot N now or fail
  og_gpu_slot="$og_gpu_root/gpu-slot-$1.lock"
  if mkdir "$og_gpu_slot" 2>/dev/null; then
    og_owner_write "$og_gpu_slot" "$2" || return 1
    og_lock_dirs="${og_lock_dirs:-}${og_lock_dirs:+ }$og_gpu_slot"
    trap 'og_lock_release' EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    return 0
  fi
  if og_gpu_dead "$og_gpu_slot"; then
    if mv "$og_gpu_slot" "$og_gpu_slot.stale.$$" 2>/dev/null; then rm -rf "$og_gpu_slot.stale.$$"; fi
  fi
  return 1
}

og_gpu_acquire() {
  if [ -n "${OG_GPU_OWNER:-}" ] && og_owner_inherited "$og_gpu_root/gpu-all.lock"; then return 0; fi
  if [ -n "${OG_GPU_SLOT:-}" ] && og_owner_inherited "$og_gpu_root/gpu-slot-$OG_GPU_SLOT.lock"; then return 0; fi
  og_queue_join "$og_gpu_root/gpu-wait-fifo" "$1" || return 1
  og_gpu_said=""
  while :; do
    if ! og_queue_turn; then sleep 1; continue; fi
    # An exclusive holder (og_gpu_acquire_all) blocks new single slots, so it is not starved.
    if [ ! -d "$og_gpu_root/gpu-all.lock" ] || [ "$(cut -d' ' -f1 "$og_gpu_root/gpu-all.lock/owner" 2>/dev/null)" = "$$" ]; then
      og_gpu_n=1
      while [ "$og_gpu_n" -le "${OG_GPU_SLOTS:-3}" ]; do
        if og_gpu_try "$og_gpu_n" "$1"; then
          OG_GPU_SLOT="$og_gpu_n"
          OG_LOCK_IDENTITIES="${OG_LOCK_IDENTITIES:-} $(cat "$og_gpu_slot/identity")"
          export OG_GPU_SLOT OG_LOCK_IDENTITIES
          og_queue_leave
          return 0
        fi
        og_gpu_n=$((og_gpu_n + 1))
      done
    elif og_gpu_dead "$og_gpu_root/gpu-all.lock"; then
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
  if [ -n "${OG_GPU_OWNER:-}" ] && og_owner_inherited "$og_gpu_root/gpu-all.lock"; then return 0; fi
  og_queue_join "$og_gpu_root/gpu-wait-fifo" "$1" || return 1
  until og_queue_turn; do sleep 1; done
  while ! mkdir "$og_gpu_root/gpu-all.lock" 2>/dev/null; do
    if og_gpu_dead "$og_gpu_root/gpu-all.lock"; then rm -rf "$og_gpu_root/gpu-all.lock"; fi
    sleep 2
  done
  og_owner_write "$og_gpu_root/gpu-all.lock" "$1" || return 1
  OG_GPU_OWNER=$$
  OG_LOCK_IDENTITIES="${OG_LOCK_IDENTITIES:-} $(cat "$og_gpu_root/gpu-all.lock/identity")"
  export OG_GPU_OWNER OG_LOCK_IDENTITIES
  og_lock_dirs="${og_lock_dirs:-}${og_lock_dirs:+ }$og_gpu_root/gpu-all.lock"
  og_gpu_n=1
  while [ "$og_gpu_n" -le "${OG_GPU_SLOTS:-3}" ]; do
    until og_gpu_try "$og_gpu_n" "$1"; do sleep 2; done
    og_gpu_n=$((og_gpu_n + 1))
  done
  og_queue_leave
}
OG_GPU_IMPL
}

og_gpu_machine="$og_gpu_root/gpu-lock.sh"
og_gpu_mine="$(og_gpu_impl | sed -n 's/^og_gpu_lock_version=//p')"
# No machine copy yet (a fresh orb): sed's exit 2 would end a caller under
# set -e -o pipefail (install-tools.sh), so it is read only when it exists.
og_gpu_theirs=""
[ ! -f "$og_gpu_machine" ] || og_gpu_theirs="$(sed -n 's/^og_gpu_lock_version=//p' "$og_gpu_machine" | head -n 1)"
if [ "${og_gpu_theirs:-0}" -lt "$og_gpu_mine" ] && mkdir -p "$og_gpu_root" 2>/dev/null; then
  og_gpu_impl > "$og_gpu_machine.$$" && mv -f "$og_gpu_machine.$$" "$og_gpu_machine"
fi
if [ -f "$og_gpu_machine" ]; then
  # shellcheck disable=SC1090
  . "$og_gpu_machine"
else
  eval "$(og_gpu_impl)"
fi
