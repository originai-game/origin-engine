#!/bin/sh
# Packs @origingame/engine and its per-platform packages into dist/pack/
# (docs/web-export.md §Packaging). Nothing is published.
#
#   sh scripts/pack.sh               this machine's packages
#   sh scripts/pack.sh --linux-x64   also @origingame/engine-linux-x64
#
# What ships is release: this runs OG_RELEASE=1 scripts/build.sh (with the
# same arguments) first, and refuses a native/ that build.sh did not record
# as release (a dev build from the gate leaves native/build-profile "dev").
#
# Per platform with an addon in native/: @origingame/engine-<platform>-<arch>
# (origin-engine.node, and the og binary when native/og.<platform>-<arch>
# exists), then @origingame/engine itself (the TypeScript API, the runtime,
# the web hosts in dist/web, dist/web-publish and dist/web-publish-2d (each
# publish directory with its profile's lanes variant), the player shell, the Basis
# transcoder, the knowledge in knowledge/; no native code: npm installs the platform package through
# optionalDependencies). Prints each tarball with its size and sha256.
set -eu
here="$(cd "$(dirname "$0")/.." && pwd)"
root="$(cd "$here/../.." && pwd)"
out="$here/dist/pack"
version="$(bun -e 'console.log(require(process.argv[1]).version)' "$here/package.json")"
OG_RELEASE=1 sh "$here/scripts/build.sh" "$@"
if [ "$(cat "$here/native/build-profile" 2>/dev/null)" != release ]; then
  echo "pack: native/ is not a release build (native/build-profile says '$(cat "$here/native/build-profile" 2>/dev/null)'); run OG_RELEASE=1 sh scripts/build.sh" >&2
  exit 1
fi
rm -rf "$out"
mkdir -p "$out/stage"
for f in dist/web/origin-engine_bg.wasm dist/web-publish/origin-engine_bg.wasm dist/web-publish-2d/origin-engine_bg.wasm dist/web-publish-2d/origin-engine.js \
    dist/web-publish/origin-engine-lanes_bg.wasm dist/web-publish/origin-engine-lanes.js dist/web-publish/snippets \
    dist/web-publish-2d/origin-engine-lanes_bg.wasm dist/web-publish-2d/origin-engine-lanes.js dist/web-publish-2d/snippets; do
  [ -e "$here/$f" ] || { echo "pack: $here/$f is missing; run OG_RELEASE=1 sh scripts/build.sh first" >&2; exit 1; }
done
sum() { if command -v sha256sum >/dev/null; then sha256sum "$1" | awk '{print $1}'; else shasum -a 256 "$1" | awk '{print $1}'; fi; }
packed=""
for addon in "$here"/native/origin-engine.*.node; do
  [ -f "$addon" ] || continue
  platform="$(basename "$addon" .node)"; platform="${platform#origin-engine.}"
  os="${platform%-*}"; cpu="${platform#*-}"
  name="@origingame/engine-$platform"
  stage="$out/stage/engine-$platform"
  mkdir -p "$stage"
  cp "$addon" "$stage/origin-engine.node"
  files='"origin-engine.node"'
  if [ -f "$here/native/og.$platform" ]; then
    cp "$here/native/og.$platform" "$stage/og"
    chmod +x "$stage/og"
    files="$files, \"og\""
  fi
  cp "$root/LICENSE" "$stage/LICENSE"
  libc=""
  [ "$os" = linux ] && libc=', "libc": ["glibc"]'
  cat > "$stage/package.json" <<JSON
{
  "name": "$name",
  "version": "$version",
  "description": "Origin Engine's native host (napi addon, built from crates/og-host-node) and the og tool for $platform. Installed by @origingame/engine through optionalDependencies.",
  "license": "MIT",
  "repository": { "type": "git", "url": "git+https://github.com/originai-game/origin-engine.git", "directory": "packages/engine" },
  "homepage": "https://github.com/originai-game/origin-engine#readme",
  "bugs": { "url": "https://github.com/originai-game/origin-engine/issues" },
  "os": ["$os"],
  "cpu": ["$cpu"]$libc,
  "main": "origin-engine.node",
  "files": [$files, "LICENSE"],
  "publishConfig": { "access": "public" }
}
JSON
  (cd "$stage" && bun pm pack --destination "$out" --quiet --ignore-scripts >/dev/null)
  packed="$packed $name"
done
# The knowledge (packages/knowledge's publishable files, the template's text
# into its scaffold/template first) ships inside the main package as
# knowledge/ (docs/release.md §2); the copy is ignored by Git.
knowledge="$here/../knowledge"
(cd "$knowledge" && bun scripts/pack.ts >/dev/null) || { echo "pack: packages/knowledge/scripts/pack.ts failed" >&2; exit 1; }
rm -rf "$here/knowledge"; mkdir -p "$here/knowledge"
for f in $(bun -e 'console.log(require(process.argv[1]).files.join(" "))' "$knowledge/package.json") package.json; do
  [ -e "$knowledge/$f" ] || { echo "pack: packages/knowledge has no $f (its files list)" >&2; exit 1; }
  cp -R "$knowledge/$f" "$here/knowledge/$f"
done
rm -rf "$here/knowledge/scripts/sync-rules.ts" "$here/knowledge/scaffold/template/node_modules"
# The main package: its files list leaves native/ and the test pages out.
(cd "$here" && bun pm pack --destination "$out" --quiet --ignore-scripts >/dev/null)
rm -rf "$out/stage"
for t in "$out"/*.tgz; do
  printf '%s  %s bytes  sha256 %s\n' "$(basename "$t")" "$(wc -c < "$t" | tr -d ' ')" "$(sum "$t")"
done
[ -n "$packed" ] || echo "pack: no addon in native/: only @origingame/engine was packed (run OG_RELEASE=1 sh scripts/build.sh)" >&2
