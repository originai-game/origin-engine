#!/bin/sh
# Builds both hosts for this package: the native addon for this machine into
# native/origin-engine.<platform>-<arch>.node, and the web host (wasm-bindgen
# --target web) into dist/web/. Honours CARGO_TARGET_DIR. Incremental.
#
# Linux x86_64 addon (executors), from macOS or Linux:
#   cargo zigbuild --release -p og-host-node --target x86_64-unknown-linux-gnu
#   cp $CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/libog_host_node.so \
#      native/origin-engine.linux-x64.node
set -eu
here="$(cd "$(dirname "$0")/.." && pwd)"
root="$(cd "$here/../.." && pwd)"
target="${CARGO_TARGET_DIR:-$root/target}"
case "$target" in /*) ;; *) target="$root/$target" ;; esac
cd "$root"
cargo build --release -q -p og-host-node
platform="$(node -p 'process.platform + "-" + process.arch' 2>/dev/null || bun -e 'console.log(process.platform + "-" + process.arch)')"
mkdir -p "$here/native" "$here/dist/web"
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
cargo build --release -q --target wasm32-unknown-unknown -p og-host-web --lib
wasm-bindgen --target web --out-dir "$here/dist/web" --out-name origin-engine \
  "$target/wasm32-unknown-unknown/release/og_host_web.wasm"
