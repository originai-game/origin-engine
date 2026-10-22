#!/bin/sh
# Builds both hosts for this package: the native addon and the og tool for
# this machine into native/origin-engine.<platform>-<arch>.node and
# native/og.<platform>-<arch>, and the web host (wasm-bindgen --target web)
# into dist/web/ (every member, both physics profiles), dist/web-publish/
# (the 3D publish variant, with the 3D lanes variant beside it:
# origin-engine-lanes.js, origin-engine-lanes_bg.wasm, snippets/; wasm
# atomics, scripts/wasm-lanes.sh, docs/web-export.md §Lanes) and
# dist/web-publish-2d/ (the 2D publish variant and the 2D lanes variant
# beside it; docs/web-export.md §Variants). Honours CARGO_TARGET_DIR.
# Incremental.
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
exe=""
case "$(uname -s)" in MINGW*|MSYS*) exe=.exe ;; esac
# A release ships only the wasm-opt output the size gate measures: without
# wasm-opt it stops here, before compiling anything.
if [ "${OG_RELEASE:-}" = 1 ] && ! command -v wasm-opt >/dev/null 2>&1; then
  echo "build.sh: OG_RELEASE=1 needs wasm-opt on PATH: a release must not ship un-optimised wasm (dist/web-publish, single and lanes) that the size gate never measured; scripts/install-tools.sh installs it" >&2
  exit 1
fi
if [ "${OG_GATE:-}" != 1 ] && [ "${OG_RELEASE:-}" != 1 ] && [ "${1:-}" != --linux-x64 ] \
    && [ -s "$gate_wasm/opt-all.txt" ] && [ -f "$target/debug/og$exe" ] \
    && { [ -f "$target/debug/libog_host_node.dylib" ] || [ -f "$target/debug/libog_host_node.so" ] || [ -f "$target/debug/og_host_node.dll" ]; }; then
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
# Remove before copying: macOS keeps a signed binary's code signature by
# inode, and an executable rewritten in place by cp is then killed on start
# (SIGKILL, exit 137, nothing printed) once its new bytes no longer match.
rm -f "$addon" "$here/native/og.$platform"
if [ -f "$bindir/libog_host_node.dylib" ]; then
  cp "$bindir/libog_host_node.dylib" "$addon"
  # ld-27037 can leave the string table off 8-byte alignment, which dyld refuses.
  codesign --remove-signature "$addon"
  bun "$here/scripts/align-macho.ts" "$addon"
  codesign -s - "$addon"
elif [ -f "$bindir/libog_host_node.so" ]; then
  cp "$bindir/libog_host_node.so" "$addon"
elif [ -f "$bindir/og_host_node.dll" ]; then
  cp "$bindir/og_host_node.dll" "$addon"
else
  echo "build.sh: no native addon in $bindir" >&2
  exit 1
fi
cp "$bindir/og$exe" "$here/native/og.$platform"
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
  # The 2D publish variant, when scripts/wasm-crates.txt declares it (a
  # 2d game's og build --publish ships it); a missing one stops the build.
  rm -rf "$here/dist/web-publish-2d"
  variant_2d="$(awk '{ sub(/#.*/, "") } $1 == "variant" && $2 == "publish2d" && $3 == "og-host-web" { print 1 }' "$root/scripts/wasm-crates.txt")"
  if [ -f "$gate_wasm/opt/og_host_web-publish2d.wasm" ]; then
    mkdir -p "$here/dist/web-publish-2d"
    cp "$gate_wasm/bindgen/og_host_web-publish2d/"* "$here/dist/web-publish-2d/"
    cp "$gate_wasm/opt/og_host_web-publish2d.wasm" "$here/dist/web-publish-2d/origin-engine_bg.wasm"
  elif [ -n "$variant_2d" ]; then
    echo "build.sh: $gate_wasm/opt/og_host_web-publish2d.wasm is missing, but scripts/wasm-crates.txt declares the publish2d variant: a 2d game's publish build would have no engine; run the gate's wasm-build and wasm-opt steps again (docs/web-export.md §Variants)" >&2
    # The functional lane runs with the gate's web module stale (OG_WASM_STALE=1,
    # scripts/check.sh): it ships what the last wasm-build produced and skips
    # the tests that load the web module; a module the gate has never built is
    # then a warning, not a stop.
    [ "${OG_WASM_STALE:-0}" = 1 ] || exit 1
  fi
  # The gate builds a lanes variant per game profile whenever
  # scripts/wasm-crates.txt marks og-host-web `lanes` (its `lanes` lines:
  # `lanes` the 3D one beside the 3D publish module, `lanes2d` the 2D one);
  # without them every publish build here would quietly ship
  # single-threaded, so a missing one stops the build.
  lanes_expected="$(awk '{ sub(/#.*/, "") } $1 == "og-host-web" && $3 == "publish" && $4 == "lanes" { print 1 }' "$root/scripts/wasm-crates.txt")"
  for v in "lanes web-publish" "lanes2d web-publish-2d"; do
    set -- $v
    if [ -f "$gate_wasm/opt/og_host_web-$1.wasm" ]; then
      cp "$gate_wasm/bindgen/og_host_web-$1/origin-engine-lanes.js" "$here/dist/$2/"
      cp -R "$gate_wasm/bindgen/og_host_web-$1/snippets" "$here/dist/$2/"
      cp "$gate_wasm/opt/og_host_web-$1.wasm" "$here/dist/$2/origin-engine-lanes_bg.wasm"
      # The pool Workers start the glue with its single-object init (scripts/rayon-worker.sh).
      sh "$here/scripts/rayon-worker.sh" "$here/dist/$2"
    elif [ -n "$lanes_expected" ]; then
      echo "build.sh: $gate_wasm/opt/og_host_web-$1.wasm is missing, but scripts/wasm-crates.txt marks og-host-web lanes: dist/$2 would ship single-threaded; run the gate's wasm-build and wasm-opt steps again (docs/ci.md §The lanes wasm)" >&2
      [ "${OG_WASM_STALE:-0}" = 1 ] || exit 1
    fi
  done
  exit 0
fi
# From here on the wasm compiles (the wasm profile, release-based): never beside a running gate.
og_lock_acquire "$target" "packages/engine/scripts/build.sh (wasm)"
wasm="$target/wasm32-unknown-unknown/wasm/og_host_web.wasm"
# Each publish variant has its own target dir, as in the gate's wasm-build
# (scripts/check.sh wasm_build): cargo keeps one fingerprint per unit, so two
# feature sets in one dir recompile each other every time. One per game
# profile, each with only its profile's physics (scripts/wasm-crates.txt
# `variant`): 3D into dist/web-publish, 2D into dist/web-publish-2d.
rm -rf "$here/dist/web-publish-2d"
mkdir -p "$here/dist/web-publish-2d"
for v in "publish physics3d web-publish" "publish2d physics2d web-publish-2d"; do
  set -- $v
  cargo build --profile wasm -q --target wasm32-unknown-unknown -p og-host-web --lib --no-default-features --features "publish,$2,shaping" --target-dir "$target/wasm-$1"
  wasm-bindgen --target web --out-dir "$here/dist/$3" --out-name origin-engine "$target/wasm-$1/wasm32-unknown-unknown/wasm/og_host_web.wasm"
  # What ships is optimised for size by the gate's own wasm-opt invocation
  # (scripts/wasm-opt.sh), so it is the module the size gate measures; without
  # wasm-opt on PATH (allowed only outside OG_RELEASE=1, checked above) the
  # publish wasm stays as wasm-bindgen wrote it.
  if command -v wasm-opt >/dev/null 2>&1; then
    "$root/scripts/wasm-opt.sh" "$here/dist/$3/origin-engine_bg.wasm" "$here/dist/$3/origin-engine_bg.wasm"
  else
    echo "build.sh: no wasm-opt; dist/$3 is not size-optimised (scripts/install-tools.sh installs it)" >&2
  fi
done
set --
# The lanes variants, one per game profile beside its publish module: the
# publish build again with wasm atomics, in their own target directory
# (scripts/wasm-lanes.sh), bound as origin-engine-lanes and optimised with
# --enable-threads. The 2D one first, so cargo's last output there is the
# 3D one, as in the gate's wasm-build.
for v in "physics2d web-publish-2d" "physics3d web-publish"; do
  set -- $v
  rm -rf "$here/dist/$2/snippets" "$here/dist/$2/origin-engine-lanes"*
  "$root/scripts/wasm-lanes.sh" --profile wasm -q -p og-host-web --lib --no-default-features --features "publish,$1,shaping"
  lanes="$("$root/scripts/wasm-lanes.sh" --target-dir)/wasm32-unknown-unknown/wasm/og_host_web.wasm"
  wasm-bindgen --target web --out-dir "$here/dist/$2" --out-name origin-engine-lanes "$lanes"
  rm -f "$here/dist/$2/origin-engine-lanes"*.d.ts
  # The pool Workers start the glue with its single-object init (scripts/rayon-worker.sh).
  sh "$here/scripts/rayon-worker.sh" "$here/dist/$2"
  if command -v wasm-opt >/dev/null 2>&1; then
    "$root/scripts/wasm-opt.sh" --threads "$here/dist/$2/origin-engine-lanes_bg.wasm" "$here/dist/$2/origin-engine-lanes_bg.wasm"
  else
    echo "build.sh: no wasm-opt; the lanes variant in dist/$2 is not size-optimised (scripts/install-tools.sh installs it)" >&2
  fi
done
set --
cargo build --profile wasm -q --target wasm32-unknown-unknown -p og-host-web --lib
wasm-bindgen --target web --out-dir "$here/dist/web" --out-name origin-engine "$wasm"
