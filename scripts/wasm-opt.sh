#!/bin/sh
# The one wasm-opt invocation: the gate's wasm-opt step (scripts/check.sh)
# and packages/engine/scripts/build.sh (what pack.sh ships) both call it, so
# the module the size gate measures is the module that ships.
#
#   scripts/wasm-opt.sh [--threads] IN.wasm OUT.wasm
#                                          optimise IN into OUT (they may be one file)
#   scripts/wasm-opt.sh --flags [--threads]
#                                          print the flags (the gate's cache key)
#
# --threads: the lanes variant (scripts/wasm-lanes.sh), whose shared memory
# and atomics wasm-opt reads only with --enable-threads.
#
# --low-memory-unused: rustc links wasm32 stack-first (the stack is
# [0, 1 MiB), data from 1 MiB), so the low 1 KiB is only ever the bottom of
# a stack about to overflow; it lets wasm-opt fold constant offsets into
# loads and stores (binaryen 133: -100 KB, -23.7 KB brotli on the publish
# variant). --converge, a second -Oz and the strip flags gained nothing.
set -eu
flags="-Oz --enable-bulk-memory --enable-nontrapping-float-to-int --low-memory-unused"
if [ "${1:-}" = --flags ]; then
  [ "${2:-}" != --threads ] || flags="$flags --enable-threads"
  printf '%s\n' "$flags"
  exit 0
fi
if [ "${1:-}" = --threads ]; then
  flags="$flags --enable-threads"
  shift
fi
if [ "$#" -ne 2 ]; then
  echo "usage: scripts/wasm-opt.sh [--threads] IN.wasm OUT.wasm | --flags [--threads]" >&2
  exit 2
fi
# shellcheck disable=SC2086 # the flags are words
exec wasm-opt $flags "$1" -o "$2"
