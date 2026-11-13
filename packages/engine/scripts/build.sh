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
#                                           2.31, from macOS or Linux; on a Linux
#                                           x86_64 host that is the only native
#                                           build); always release
#   OG_GATE=1 sh scripts/build.sh --native  the gate's dev addon and og only (its
#                                           fixtures step, which runs og test)
#   OG_RELEASE=1 sh scripts/build.sh --native [--linux-x64]
#                                           the native addon and og only, no wasm:
#                                           what a platform's job of the release
#                                           workflow builds (.github/workflows/
#                                           release.yml; the wasm is the same bytes
#                                           on every platform and is built once)
#
# Three modes (AGENTS.md §Build and gate discipline, docs/ci.md):
# - OG_GATE=1 (scripts/check.sh's bun-test): the addon and og are the dev
#   profile the gate's rust-build step already built, copied without running
#   cargo; the wasm is what the gate's wasm-build and wasm-bindgen steps left
#   in $OG_WASM_OUT (default <target>/og-wasm).
# - Standalone, when the gate has run here (<target>/og-wasm/dev-all.txt and
#   the dev addon and og exist) and OG_RELEASE is not 1: the OG_GATE=1 mode,
#   announced in one line, after a cargo build brings the dev addon and og up
#   to date.
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
case "$target" in /*|[A-Za-z]:[/\\]*) ;; *) target="$root/$target" ;; esac
cd "$root"
. "$root/scripts/lock.sh"
gate_wasm="${OG_WASM_OUT:-$target/og-wasm}"
exe=""
case "$(uname -s)" in MINGW*|MSYS*) exe=.exe ;; esac
native_only=0; linux_x64=0
for a in "$@"; do
  case "$a" in
    --native) native_only=1 ;;
    --linux-x64) linux_x64=1 ;;
    *) echo "build.sh: unknown argument '$a' (takes --native and --linux-x64)" >&2; exit 2 ;;
  esac
done
if [ "$native_only" = 1 ] && [ "${OG_RELEASE:-}" != 1 ] && [ "${OG_GATE:-}" != 1 ]; then
  echo "build.sh: --native is a release build (OG_RELEASE=1 sh scripts/build.sh --native), or the gate's dev addon and og (OG_GATE=1)" >&2
  exit 2
fi
gate_built=0
[ "${OG_GATE:-}" != 1 ] || gate_built=1
if [ "${OG_GATE:-}" != 1 ] && [ "${OG_RELEASE:-}" != 1 ] && [ "$linux_x64" != 1 ] \
    && [ -s "$gate_wasm/dev-all.txt" ] && [ -f "$target/debug/og$exe" ] \
    && { [ -f "$target/debug/libog_host_node.dylib" ] || [ -f "$target/debug/libog_host_node.so" ] || [ -f "$target/debug/og_host_node.dll" ]; }; then
  echo "build.sh: the gate has built here, so this is the OG_GATE=1 build (dev addon and og, the gate's wasm); OG_RELEASE=1 sh scripts/build.sh builds release" >&2
  OG_GATE=1
fi
# A development cook needs the same pinned weights as a packaged engine.
# Native-only release jobs ship only the platform binary, not engine assets.
if [ "$native_only" != 1 ] || [ "${OG_GATE:-}" = 1 ]; then
  bun "$here/scripts/motion-model.ts" ensure
fi
platform="$(node -p 'process.platform + "-" + process.arch' 2>/dev/null || bun -e 'console.log(process.platform + "-" + process.arch)')"
# --linux-x64 on a Linux x86_64 host: the zigbuild below (glibc 2.31) is the
# one native build; the host's own would be the same addon against the
# host's newer glibc, overwritten a moment later.
host_build=1
[ "$linux_x64" = 1 ] && [ "$platform" = linux-x64 ] && [ "${OG_GATE:-}" != 1 ] && host_build=0
if [ "${OG_GATE:-}" = 1 ]; then
  # Under the gate, rust-build (scripts/gate/native.py) built the dev addon
  # and og before any step that runs this one: a cargo build here re-linked
  # target/debug/og while other steps ran it. Standalone, the same command
  # brings them up to date, compiling only what changed since the gate.
  [ "$gate_built" = 1 ] || cargo build -q -p og-host-node -p og-cli
  [ -f "$target/debug/og$exe" ] || { echo "build.sh: no dev og in $target/debug: the gate's rust-build (scripts/check.sh) builds it" >&2; exit 1; }
  bindir="$target/debug"
  profile=dev
else
  og_admission_acquire shared "packages/engine/scripts/build.sh (release)"
  og_lock_acquire "$target" "packages/engine/scripts/build.sh (release)"
  [ "$host_build" = 0 ] || cargo build --release -q -p og-host-node -p og-cli
  bindir="$target/release"
  profile=release
fi
mkdir -p "$here/native" "$here/dist/web" "$here/dist/web-publish"
echo "$profile" > "$here/native/build-profile"
addon="$here/native/origin-engine.$platform.node"
og_bin="$here/native/og.$platform"
# Each binary is written beside its place and renamed over it: never
# rewritten in place, since macOS keeps a signed binary's code signature by
# inode and an executable rewritten by cp is then killed on start (SIGKILL,
# exit 137, nothing printed); and never removed first, since a gate step
# running beside this one (bun-functional beside ui-styles' build) would
# find no og or addon in between.
if [ "$host_build" = 0 ]; then
  rm -f "$addon" "$og_bin"
elif [ -f "$bindir/libog_host_node.dylib" ]; then
  cp "$bindir/libog_host_node.dylib" "$addon.$$"
  # ld-27037 can leave the string table off 8-byte alignment, which dyld refuses.
  codesign --remove-signature "$addon.$$"
  bun "$here/scripts/align-macho.ts" "$addon.$$"
  codesign -s - "$addon.$$"
elif [ -f "$bindir/libog_host_node.so" ]; then
  cp "$bindir/libog_host_node.so" "$addon.$$"
elif [ -f "$bindir/og_host_node.dll" ]; then
  cp "$bindir/og_host_node.dll" "$addon.$$"
else
  echo "build.sh: no native addon in $bindir" >&2
  exit 1
fi
if [ "$host_build" != 0 ]; then
  mv -f "$addon.$$" "$addon"
  cp "$bindir/og$exe" "$og_bin.$$"
  mv -f "$og_bin.$$" "$og_bin"
fi
if [ "$linux_x64" = 1 ]; then
  triple=x86_64-unknown-linux-gnu
  cargo zigbuild --release -q -p og-host-node -p og-cli --target "$triple.2.31"
  cp "$target/$triple/release/libog_host_node.so" "$here/native/origin-engine.linux-x64.node"
  cp "$target/$triple/release/og" "$here/native/og.linux-x64"
fi
# --native: the addon and og are built; the wasm is another job's (the
# release workflow) or an earlier build's.
[ "$native_only" != 1 ] || exit 0
# The web host twice: every member (dist/web), then a publish build that keeps
# only the player-facing probe members (dist/web-publish, `og build --publish`).
# Both builds write the same og_host_web.wasm, so each is bound before the next.
gate_modules="$gate_wasm/dev"
if [ "${OG_RELEASE:-}" = 1 ]; then
  bash "$root/scripts/release-wasm.sh"
elif [ "${OG_GATE:-}" != 1 ]; then
  bash "$root/scripts/release-wasm.sh" --development
fi
# The variants the wasm was built with (scripts/gate/commands.sh
# step_wasm_build, OG_WASM_VARIANTS): `dev` has no publish or lanes module,
# so dist/web-publish* stay empty and the tests that load them skip.
variants="${OG_WASM_VARIANTS:-$(cat "$gate_wasm/variants" 2>/dev/null || echo all)}"
if [ -f "$gate_wasm/bindgen/og_host_web/origin-engine_bg.wasm" ] \
    && { [ -f "$gate_modules/og_host_web-publish.wasm" ] || [ "$variants" = dev ]; }; then
  rm -rf "$here/dist/web-publish"/* "$here/dist/web"/* "$here/dist/web-publish-2d"
  cp "$gate_wasm/bindgen/og_host_web/"* "$here/dist/web/"
  [ "$variants" != dev ] || exit 0
  cp "$gate_wasm/bindgen/og_host_web-publish/"* "$here/dist/web-publish/"
  cp "$gate_modules/og_host_web-publish.wasm" "$here/dist/web-publish/origin-engine_bg.wasm"
  # The 2D publish variant, when scripts/wasm-crates.txt declares it (a
  # 2d game's og build --publish ships it); a missing one stops the build.
  variant_2d="$(awk '{ sub(/#.*/, "") } $1 == "variant" && $2 == "publish2d" && $3 == "og-host-web" { print 1 }' "$root/scripts/wasm-crates.txt")"
  if [ -f "$gate_modules/og_host_web-publish2d.wasm" ]; then
    mkdir -p "$here/dist/web-publish-2d"
    cp "$gate_wasm/bindgen/og_host_web-publish2d/"* "$here/dist/web-publish-2d/"
    cp "$gate_modules/og_host_web-publish2d.wasm" "$here/dist/web-publish-2d/origin-engine_bg.wasm"
  elif [ -n "$variant_2d" ] && [ "$variants" != reference ]; then
    echo "build.sh: $gate_modules/og_host_web-publish2d.wasm is missing; prepare all wasm variants again (docs/web-export.md §Variants)" >&2
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
    [ "$variants" != reference ] || [ "$1" != lanes2d ] || continue
    if [ -f "$gate_modules/og_host_web-$1.wasm" ]; then
      cp "$gate_wasm/bindgen/og_host_web-$1/origin-engine-lanes.js" "$here/dist/$2/"
      cp -R "$gate_wasm/bindgen/og_host_web-$1/snippets" "$here/dist/$2/"
      cp "$gate_modules/og_host_web-$1.wasm" "$here/dist/$2/origin-engine-lanes_bg.wasm"
      # The pool Workers start the glue with its single-object init (scripts/rayon-worker.sh).
      sh "$here/scripts/rayon-worker.sh" "$here/dist/$2"
    elif [ -n "$lanes_expected" ]; then
      echo "build.sh: $gate_modules/og_host_web-$1.wasm is missing; prepare all lanes variants again (docs/ci.md §The lanes wasm)" >&2
      [ "${OG_WASM_STALE:-0}" = 1 ] || exit 1
    fi
  done
  # The AV1 fallback decoder (G3-2), its own module beside the engine's.
  [ "${OG_GATE:-}" = 1 ] || sh "$here/scripts/build-av1.sh"
  exit 0
fi
echo "build.sh: incomplete wasm in $gate_modules; run scripts/check.sh --only wasm-bindgen (development) or bash scripts/release-wasm.sh (release)" >&2
exit 1
