#!/bin/sh
# The one lanes wasm build: the gate's wasm-build step (scripts/check.sh) and
# packages/engine/scripts/build.sh both run cargo through it, so the lanes
# module the size gate measures is the one that ships (docs/web-export.md
# §Lanes).
#
#   scripts/wasm-lanes.sh CARGO_BUILD_ARGS...   cargo build ARGS for wasm32 with atomics
#   scripts/wasm-lanes.sh --target-dir          print the target directory it builds in
#
# The lanes variant is the publish engine with wasm atomics: rayon on a pool
# of Workers sharing the module's memory (wasm-bindgen-rayon). Shared memory
# needs std rebuilt with atomics (-Z build-std, hence RUSTC_BOOTSTRAP on the
# stable toolchain rust-toolchain.toml pins, and its rust-src component), a
# memory imported and shared with a maximum (2 GiB: browsers reserve it but
# commit what the module grows to), and the TLS symbols wasm-bindgen's
# thread transform reads. Its own target directory (<target>/wasm-mt) keeps
# these RUSTFLAGS from invalidating the single-threaded wasm32 build.
set -eu
root="$(cd "$(dirname "$0")/.." && pwd)"
target="${CARGO_TARGET_DIR:-$root/target}"
case "$target" in /*) ;; *) target="$root/$target" ;; esac
dir="$target/wasm-mt"
if [ "${1:-}" = --target-dir ]; then
  printf '%s\n' "$dir"
  exit 0
fi
RUSTC_BOOTSTRAP=1 CARGO_TARGET_DIR="$dir" \
RUSTFLAGS="-C target-feature=+atomics,+bulk-memory -C link-arg=--shared-memory -C link-arg=--import-memory -C link-arg=--max-memory=2147483648 -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base" \
  exec cargo build --target wasm32-unknown-unknown -Z build-std=panic_abort,std "$@"
