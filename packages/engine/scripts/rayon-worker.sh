#!/bin/sh
# The lanes pool's Worker script, as it ships: wasm-bindgen-rayon's
# workerHelpers.no-bundler.js (in <dir>/snippets/, beside
# origin-engine-lanes.js) starts each pool Worker with
# `pkg.default(data.module, data.memory)`, the two-argument form wasm-bindgen
# 0.2.129's glue warns about on every Worker ("using deprecated parameters
# for the initialization function; pass a single object instead", eight
# lines a lanes page: examples/WEB-FINDINGS.md #8). The pinned versions stay
# (docs/ci.md §Toolchain); this rewrites that one call to the single-object
# form the glue takes, `{ module_or_path, memory }`, which instantiates the
# same module on the same memory.
#
#   sh scripts/rayon-worker.sh <dir>     <dir>: dist/web-publish or dist/web-publish-2d
#
# Exits 1 when the directory has a lanes module but no helper, or a helper
# whose start call is neither form (a wasm-bindgen-rayon move changed it:
# look at the new helper before shipping it). tests/web-export.test.ts checks
# the shipped glue.
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
done
if [ "$found" = 0 ]; then
  echo "rayon-worker.sh: $dir has origin-engine-lanes.js but no snippets/wasm-bindgen-rayon-*/src/workerHelpers.no-bundler.js: the pool would have no Worker script" >&2
  exit 1
fi
