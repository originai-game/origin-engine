#!/bin/sh
# One measured build at a time, excluding gates and GPU suites on this machine.
# Run from the repository root; the command owns its timer, after the locks.
set -eu
. scripts/lock.sh
og_lock_acquire "${OG_GATE_MACHINE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/og-gate}" "build-speed ($PWD)"
og_gpu_acquire_all build-speed
"$@"
