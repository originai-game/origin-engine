#!/bin/sh
# The lanes pool's Worker script, as it ships: wasm-bindgen-rayon's
# workerHelpers.no-bundler.js (in <dir>/snippets/, beside
# origin-engine-lanes.js). Two rewrites; the pinned versions stay
# (docs/ci.md §Toolchain):
#
# - Its start call `pkg.default(data.module, data.memory)` is the
#   two-argument form wasm-bindgen 0.2.129's glue warns about on every
#   Worker ("using deprecated parameters for the initialization function;
#   pass a single object instead", eight lines a lanes page: web finding
#   #8). It becomes the single-object form the glue takes,
#   `{ module_or_path, memory }`, which instantiates the same module on the
#   same memory.
# - It starts each pool Worker from a `blob:` copy of itself (fetched, so a
#   helper on another origin still starts), which a Content-Security-Policy
#   without `worker-src blob:` refuses. A helper on the page's origin, as a
#   built page serves it, now starts its Workers from its own URL; the
#   `blob:` copy stays for one on another origin, where no Worker can start
#   from the URL itself (docs/web-export.md §Hosting).
#   File-backed hosts also use the helper URL directly: Bun has no location.
#   WebKit needs distinct request URLs for simultaneous workers served by
#   the offline cache. The service worker strips their query for cache lookup.
#
#   sh scripts/rayon-worker.sh <dir>     <dir>: dist/web-publish
#
# Exits 1 when the directory has a lanes module but no helper, or a helper
# whose start call or Worker start is in neither form (a wasm-bindgen-rayon
# move changed it: read the new helper before shipping it).
# tests/web-csp.test.ts runs it on the pinned helper.
set -eu
dir="$1"
[ -f "$dir/origin-engine-lanes.js" ] || exit 0
found=0
for f in "$dir"/snippets/wasm-bindgen-rayon-*/src/workerHelpers.no-bundler.js; do
  [ -f "$f" ] || continue
  found=1
  old='pkg.default(data.module, data.memory)'
  new='pkg.default({ module_or_path: data.module, memory: data.memory })'
  if grep -qF "$old" "$f"; then
    sed "s/pkg\.default(data\.module, data\.memory)/pkg.default({ module_or_path: data.module, memory: data.memory })/" "$f" > "$f.tmp"
    mv "$f.tmp" "$f"
  fi
  if ! grep -qF "$new" "$f"; then
    echo "rayon-worker.sh: $f starts its Worker neither as '$old' nor as '$new'; read the helper wasm-bindgen-rayon now writes before shipping it" >&2
    exit 1
  fi
  # wasm-bindgen-rayon 1.3's two lines: `let scriptBlob = await fetch(import.meta.url)…;`
  # then `let url = URL.createObjectURL(scriptBlob);`, which become one.
  prior='let url = new URL(import.meta.url).origin === self.location.origin ? import.meta.url : URL.createObjectURL(await fetch(import.meta.url).then(r => r.blob()));'
  own='let url = ((!self.location && new URL(import.meta.url).protocol === "file:") || new URL(import.meta.url).origin === self.location?.origin) ? import.meta.url : URL.createObjectURL(await fetch(import.meta.url).then(r => r.blob()));'
  if grep -qF 'let scriptBlob = await fetch(import.meta.url).then(r => r.blob());' "$f" || grep -qF "$prior" "$f"; then
    awk -v own="$own" '
      index($0, "let url = new URL(import.meta.url).origin === self.location.origin") {
        indent = $0; sub(/let url.*/, "", indent)
        print indent own; next
      }
      index($0, "let scriptBlob = await fetch(import.meta.url).then(r => r.blob());") {
        indent = $0; sub(/let scriptBlob.*/, "", indent)
        print indent "// origin-engine (scripts/rayon-worker.sh): on the page'"'"'s origin, from this file itself, which"
        print indent "// a CSP without `worker-src blob:` allows; elsewhere, from the blob copy."
        print indent own; next
      }
      index($0, "let url = URL.createObjectURL(scriptBlob);") { next }
      { print }
    ' "$f" > "$f.tmp"
    mv "$f.tmp" "$f"
  fi
  if ! grep -qF "$own" "$f" || grep -qF 'scriptBlob' "$f"; then
    echo "rayon-worker.sh: $f starts its pool Workers neither from a blob: copy as wasm-bindgen-rayon 1.3 writes it nor from its own URL; read the helper wasm-bindgen-rayon now writes before shipping it" >&2
    exit 1
  fi
  pool='Array.from({ length: builder.numThreads() }, async () => {'
  lane='Array.from({ length: builder.numThreads() }, async (_, lane) => {'
  if grep -qF "$pool" "$f"; then
    awk -v pool="$pool" -v lane="$lane" -v own="$own" '
      index($0, pool) { sub(/Array.from.*/, lane); print; next }
      index($0, own) {
        print
        indent = $0; sub(/let url.*/, "", indent)
        print indent "const ua = globalThis.navigator?.userAgent ?? \"\";"
        print indent "if (url === import.meta.url && /^https?:/.test(url) && ua.includes(\"AppleWebKit/\") && !ua.includes(\"Chrome/\") && !ua.includes(\"Chromium/\")) {"
        print indent "  const workerUrl = new URL(url);"
        print indent "  workerUrl.searchParams.set(\"og-lane\", String(lane));"
        print indent "  url = workerUrl.href;"
        print indent "}"
        next
      }
      { print }
    ' "$f" > "$f.tmp"
    mv "$f.tmp" "$f"
  fi
  if ! grep -qF "$lane" "$f" || ! grep -qF 'workerUrl.searchParams.set("og-lane", String(lane));' "$f"; then
    echo "rayon-worker.sh: $f has an unrecognized pool loop; read the helper wasm-bindgen-rayon now writes before shipping it" >&2
    exit 1
  fi
  # A split lanes primary (docs/web-export.md §Modules): each pool Worker
  # instantiates it with the placeholder namespaces it imports and loads a
  # module into its own instance when a lane first calls into it
  # (scripts/og-pool-split.js, copied beside the helper). The engine Worker
  # hands the Worker where the modules are (split.ts __ogPoolSplit) and the
  # shim's URL, which a Worker started from a blob: copy cannot resolve
  # relatively.
  cp "$(dirname "$0")/og-pool-split.js" "$(dirname "$f")/og-pool-split.js"
  cp "$(dirname "$0")/../src/web/split-protocol.js" "$(dirname "$f")/split-protocol.js"
  init='  const pkg = await import(data.mainJS);'
  install='  if (data.split) (await import(data.poolSplitJS)).installPoolSplit(data.split);'
  handed='    split: globalThis.__ogPoolSplit ?? null, poolSplitJS: new URL("./og-pool-split.js", import.meta.url).href,'
  if ! grep -qF "$install" "$f"; then
    grep -qF "$init" "$f" && grep -qF "mainJS: builder.mainJS()" "$f" || {
      echo "rayon-worker.sh: $f has no '$init' or no 'mainJS: builder.mainJS()'; read the helper wasm-bindgen-rayon now writes before shipping it" >&2
      exit 1
    }
    awk -v init="$init" -v install="$install" -v handed="$handed" '
      index($0, init) == 1 { print install }
      index($0, "mainJS: builder.mainJS()") { sub(/mainJS: builder\.mainJS\(\)/, "mainJS: builder.mainJS(),"); print; print handed; next }
      { print }
    ' "$f" > "$f.tmp"
    mv "$f.tmp" "$f"
  fi
  if ! grep -qF "$install" "$f" || ! grep -qF "$handed" "$f"; then
    echo "rayon-worker.sh: $f does not hand its pool Workers the split modules; read the helper wasm-bindgen-rayon now writes before shipping it" >&2
    exit 1
  fi
done
if [ "$found" = 0 ]; then
  echo "rayon-worker.sh: $dir has origin-engine-lanes.js but no snippets/wasm-bindgen-rayon-*/src/workerHelpers.no-bundler.js: the pool would have no Worker script" >&2
  exit 1
fi
