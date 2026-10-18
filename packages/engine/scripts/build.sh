#!/bin/sh
# Builds both hosts for this package: the native addon and the og tool for
# this machine into native/origin-engine.<platform>-<arch>.node and
# native/og.<platform>-<arch>, and the web host (wasm-bindgen --target web)
# into dist/web/ and dist/web-publish/. Honours CARGO_TARGET_DIR. Incremental.
#
#   sh scripts/build.sh                     dev when the gate has built, else release
#   OG_RELEASE=1 sh scripts/build.sh        release: what ships (scripts/pack.sh sets it)
#   OG_RELEASE=1 sh scripts/build.sh --linux-x64
#                                           also the Linux x86_64 addon and og for
#                                           executors (cargo zigbuild against glibc
#                                           2.31, from macOS or Linux); always release
#
# Three modes (AGENTS.md §Build and gate discipline, docs/ci.md):
# - OG_GATE=1 (scripts/check.sh's bun-test): the addon and og are the dev
#   profile the test step already built, the wasm is what the gate's
#   wasm-build and wasm-opt steps left in $OG_WASM_OUT (default
#   <target>/og-wasm).
# - Standalone, when the gate has run here (<target>/og-wasm/opt-all.txt and
#   the dev addon and og exist) and OG_RELEASE is not 1: exactly the OG_GATE=1
#   mode, announced in one line.
# - Otherwise, OG_RELEASE=1 or --linux-x64: release addon and og and the wasm
#   compiled here, holding the gate lock (scripts/lock.sh) so it never
#   compiles beside a running gate.
#
# native/build-profile records the mode (dev or release); scripts/pack.sh
# packs only a release build.
set -eu
here="$(cd "$(dirname "$0")/.." && pwd)"
root="$(cd "$here/../.." && pwd)"
target="${CARGO_TARGET_DIR:-$root/target}"
case "$target" in /*) ;; *) target="$root/$target" ;; esac
cd "$root"
. "$root/scripts/lock.sh"
gate_wasm="${OG_WASM_OUT:-$target/og-wasm}"
if [ "${OG_GATE:-}" != 1 ] && [ "${OG_RELEASE:-}" != 1 ] && [ "${1:-}" != --linux-x64 ] \
    && [ -s "$gate_wasm/opt-all.txt" ] && [ -f "$target/debug/og" ] \
    && { [ -f "$target/debug/libog_host_node.dylib" ] || [ -f "$target/debug/libog_host_node.so" ]; }; then
  echo "build.sh: the gate has built here, so this is the OG_GATE=1 build (dev addon and og, the gate's wasm); OG_RELEASE=1 sh scripts/build.sh builds release" >&2
  OG_GATE=1
fi
if [ "${OG_GATE:-}" = 1 ]; then
  cargo build -q -p og-host-node -p og-cli
  bindir="$target/debug"
  profile=dev
else
  og_lock_acquire "$target" "packages/engine/scripts/build.sh (release)"
  cargo build --release -q -p og-host-node -p og-cli
  bindir="$target/release"
  profile=release
fi
platform="$(node -p 'process.platform + "-" + process.arch' 2>/dev/null || bun -e 'console.log(process.platform + "-" + process.arch)')"
mkdir -p "$here/native" "$here/dist/web" "$here/dist/web-publish"
echo "$profile" > "$here/native/build-profile"
addon="$here/native/origin-engine.$platform.node"
if [ -f "$bindir/libog_host_node.dylib" ]; then
  cp "$bindir/libog_host_node.dylib" "$addon"
  # ld-27037 can leave the string table off 8-byte alignment, which dyld refuses.
  codesign --remove-signature "$addon"
  bun "$here/scripts/align-macho.ts" "$addon"
  codesign -s - "$addon"
elif [ -f "$bindir/libog_host_node.so" ]; then
  cp "$bindir/libog_host_node.so" "$addon"
fi
cp "$bindir/og" "$here/native/og.$platform"
if [ "${1:-}" = --linux-x64 ]; then
  triple=x86_64-unknown-linux-gnu
  cargo zigbuild --release -q -p og-host-node -p og-cli --target "$triple.2.31"
  cp "$target/$triple/release/libog_host_node.so" "$here/native/origin-engine.linux-x64.node"
  cp "$target/$triple/release/og" "$here/native/og.linux-x64"
fi
# The web host twice: every member (dist/web), then a publish build that keeps
# only the player-facing probe members (dist/web-publish, `og build --publish`).
# Both builds write the same og_host_web.wasm, so each is bound before the next.
if [ "${OG_GATE:-}" = 1 ] && [ -f "$gate_wasm/opt/og_host_web-publish.wasm" ] && [ -f "$gate_wasm/bindgen/og_host_web/origin-engine_bg.wasm" ]; then
  rm -rf "$here/dist/web-publish"/* "$here/dist/web"/*
  cp "$gate_wasm/bindgen/og_host_web-publish/"* "$here/dist/web-publish/"
  cp "$gate_wasm/opt/og_host_web-publish.wasm" "$here/dist/web-publish/origin-engine_bg.wasm"
  cp "$gate_wasm/bindgen/og_host_web/"* "$here/dist/web/"
  exit 0
fi
# From here on the wasm compiles in release: never beside a running gate.
og_lock_acquire "$target" "packages/engine/scripts/build.sh (wasm)"
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
