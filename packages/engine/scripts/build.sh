#!/bin/sh
# Builds both hosts for this package: the native addon and the og tool for
# this machine into native/origin-engine.<platform>-<arch>.node and
# native/og.<platform>-<arch>, and the web host (wasm-bindgen --target web)
# into dist/web/ and dist/web-publish/. Honours CARGO_TARGET_DIR. Incremental.
#
#   sh scripts/build.sh               this machine and the web
#   sh scripts/build.sh --linux-x64   also the Linux x86_64 addon and og for
#                                     executors (cargo zigbuild against glibc
#                                     2.31, from macOS or Linux)
#
# scripts/pack.sh packs what this leaves in native/ and dist/.
set -eu
here="$(cd "$(dirname "$0")/.." && pwd)"
root="$(cd "$here/../.." && pwd)"
target="${CARGO_TARGET_DIR:-$root/target}"
case "$target" in /*) ;; *) target="$root/$target" ;; esac
cd "$root"
cargo build --release -q -p og-host-node -p og-cli
platform="$(node -p 'process.platform + "-" + process.arch' 2>/dev/null || bun -e 'console.log(process.platform + "-" + process.arch)')"
mkdir -p "$here/native" "$here/dist/web" "$here/dist/web-publish"
addon="$here/native/origin-engine.$platform.node"
if [ -f "$target/release/libog_host_node.dylib" ]; then
  cp "$target/release/libog_host_node.dylib" "$addon"
  # ld-27037 can leave the string table off 8-byte alignment, which dyld refuses.
  codesign --remove-signature "$addon"
  bun "$here/scripts/align-macho.ts" "$addon"
  codesign -s - "$addon"
elif [ -f "$target/release/libog_host_node.so" ]; then
  cp "$target/release/libog_host_node.so" "$addon"
fi
cp "$target/release/og" "$here/native/og.$platform"
if [ "${1:-}" = --linux-x64 ]; then
  triple=x86_64-unknown-linux-gnu
  cargo zigbuild --release -q -p og-host-node -p og-cli --target "$triple.2.31"
  cp "$target/$triple/release/libog_host_node.so" "$here/native/origin-engine.linux-x64.node"
  cp "$target/$triple/release/og" "$here/native/og.linux-x64"
fi
# The web host twice: every member (dist/web), then a publish build that keeps
# only the player-facing probe members (dist/web-publish, `og build --publish`).
# Both builds write the same og_host_web.wasm, so each is bound before the next.
wasm="$target/wasm32-unknown-unknown/release/og_host_web.wasm"
cargo build --release -q --target wasm32-unknown-unknown -p og-host-web --lib --features publish
wasm-bindgen --target web --out-dir "$here/dist/web-publish" --out-name origin-engine "$wasm"
# What ships is optimised for size (wasm-opt -Oz, as CI's size gate measures);
# without wasm-opt on PATH the publish wasm stays as wasm-bindgen wrote it.
if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int \
    "$here/dist/web-publish/origin-engine_bg.wasm" -o "$here/dist/web-publish/origin-engine_bg.wasm"
else
  echo "build.sh: no wasm-opt; dist/web-publish is not size-optimised (scripts/install-tools.sh installs it)" >&2
fi
cargo build --release -q --target wasm32-unknown-unknown -p og-host-web --lib
wasm-bindgen --target web --out-dir "$here/dist/web" --out-name origin-engine "$wasm"
