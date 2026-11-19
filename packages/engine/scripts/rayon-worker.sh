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
#
#   sh scripts/rayon-worker.sh <dir>     <dir>: dist/web-publish or dist/web-publish-2d
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
  own='let url = new URL(import.meta.url).origin === self.location.origin ? import.meta.url : URL.createObjectURL(await fetch(import.meta.url).then(r => r.blob()));'
  if grep -qF 'let scriptBlob = await fetch(import.meta.url).then(r => r.blob());' "$f"; then
    awk -v own="$own" '
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
done
if [ "$found" = 0 ]; then
  echo "rayon-worker.sh: $dir has origin-engine-lanes.js but no snippets/wasm-bindgen-rayon-*/src/workerHelpers.no-bundler.js: the pool would have no Worker script" >&2
  exit 1
fi
