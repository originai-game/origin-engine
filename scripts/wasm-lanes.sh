#!/bin/sh
# The one lanes wasm build: the gate's wasm-build step (scripts/check.sh),
# packages/engine/scripts/build.sh and the determinism tool
# (tools/wasm-determinism/run.sh) all run cargo through it, so the lanes
# modules the size gate measures are the ones that ship and the ones whose
# physics the determinism step compares (docs/web-export.md §Lanes).
#
#   scripts/wasm-lanes.sh CARGO_BUILD_ARGS...   cargo build ARGS for wasm32 with atomics
#   scripts/wasm-lanes.sh --target-dir          print the target directory it builds in
#
# A lanes variant is a publish engine with wasm atomics (one per physics
# profile, scripts/wasm-crates.txt `lanes` lines): rayon on a pool
# of Workers sharing the module's memory (wasm-bindgen-rayon). Shared memory
# needs std rebuilt with atomics (-Z build-std, hence RUSTC_BOOTSTRAP on the
# stable toolchain rust-toolchain.toml pins, and its rust-src component), a
# memory imported and shared with a maximum (2 GiB: browsers reserve it but
# commit what the module grows to; the single-threaded module may grow to
# 4 GiB, so a game past 2 GiB of wasm memory runs out on lanes only), and the TLS symbols wasm-bindgen's
# thread transform reads. Its own target directory (<target>/wasm-mt) keeps
# these RUSTFLAGS from invalidating the single-threaded wasm32 build; the 3D
# and 2D lanes modules share it (cargo keys each unit by its features, so
# they share std and every dependency and only og-physics, og-runtime and
# og-host-web's own units are built once per profile).
set -eu
root="$(cd "$(dirname "$0")/.." && pwd)"
target="${CARGO_TARGET_DIR:-$root/target}"
case "$target" in /*) ;; *) target="$root/$target" ;; esac
dir="$target/wasm-mt"
if [ "${1:-}" = --target-dir ]; then
  printf '%s\n' "$dir"
  exit 0
fi
lanes_flags="-C target-feature=+atomics,+bulk-memory -C link-arg=--shared-memory -C link-arg=--import-memory -C link-arg=--max-memory=2147483648 -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base"
# The caller's RUSTFLAGS stay, with the lanes flags after them. Cargo reads
# CARGO_ENCODED_RUSTFLAGS before RUSTFLAGS, so one inherited from an outer
# cargo (a build script, `cargo run` of a tool that builds) would drop the
# lanes flags silently: it is unset, and its flags are not carried over.
unset CARGO_ENCODED_RUSTFLAGS
RUSTC_BOOTSTRAP=1 CARGO_TARGET_DIR="$dir" \
RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }$lanes_flags" \
  exec cargo build --target wasm32-unknown-unknown -Z build-std=panic_abort,std "$@"
