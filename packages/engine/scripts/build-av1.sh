#!/bin/sh
# The web host's AV1 fallback (G3-2): crates/og-video-wasm, rav1d built for
# wasm32-wasip1 with simd128, into dist/av1/og-av1.wasm. The decoder Worker
# (src/web/video.ts) fetches it only where WebCodecs cannot decode AV1; it is
# never part of the engine's wasm, its size gate or a page's boot download.
# build.sh runs it after the engine's wasm; standalone it builds only this.
# Honours CARGO_TARGET_DIR (its own wasip1 directory under it).
set -eu
here="$(cd "$(dirname "$0")/.." && pwd)"
root="$(cd "$here/../.." && pwd)"
target="${CARGO_TARGET_DIR:-$root/target}"
case "$target" in /*|[A-Za-z]:[/\\]*) ;; *) target="$root/$target" ;; esac
# Release at opt-level 3 (rav1d's loops run every frame; z halves its speed),
# simd128 everywhere the fallback runs (every browser with VideoFrame has it).
RUSTFLAGS="-C target-feature=+simd128" cargo build --release -q --target wasm32-wasip1 \
  -p og-video-wasm --lib --target-dir "$target/wasip1"
mkdir -p "$here/dist/av1"
out="$here/dist/av1/og-av1.wasm"
built="$target/wasip1/wasm32-wasip1/release/og_video_wasm.wasm"
if [ "${OG_RELEASE:-}" = 1 ] && command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -O3 --enable-simd --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --strip-debug --strip-producers "$built" -o "$out"
else
  echo "build-av1.sh: development decoder, without wasm-opt" >&2
  cp "$built" "$out"
fi
