#!/usr/bin/env bash
# Where a wasm module's bytes are, per crate and per function, measured on
# the module that ships (wasm-bindgen's output)
# rather than on the raw cargo output; the former test sample
# has a report and what it led to.
#
#   scripts/wasm-size-report.sh [-n N] [--keep DIR] MODULE.wasm
#
# MODULE is a cargo-built wasm (the lanes variant is
# target/wasm-mt/wasm32-unknown-unknown/<profile>/og_host_web.wasm, the
# gate's copies target/og-wasm/raw/*.wasm) or one wasm-bindgen already
# wrote. A module with wasm atomics (the lanes variant: it exports
# __wasm_init_tls) is bound as origin-engine-lanes.
#
# Brotli -q 11 printed first is the size gate's number for this module.
# The tables read bindgen's retained name section (per crate with each
# crate's brotli alone, then the top N functions, 30 by default).
# --keep DIR keeps the bound module there;
# `bun scripts/wasm-size-report.ts DIR/bindgen/*_bg.wasm`
# prints the tables again without rebuilding anything.
#
# Needs wasm-bindgen (scripts/install-tools.sh), brotli and bun.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source-path=SCRIPTDIR
. "$here/lib.sh"

top=30
keep=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    -n) top="$2"; shift 2 ;;
    --keep) keep="$2"; shift 2 ;;
    -h|--help) sed -n '2,/^set -euo/p' "$0" | sed '$d; s/^# \{0,1\}//'; exit 0 ;;
    *) break ;;
  esac
done
if [ "$#" -ne 1 ]; then
  og_err "usage: scripts/wasm-size-report.sh [-n N] [--keep DIR] MODULE.wasm"
  exit 2
fi
module="$1"
[ -f "$module" ] || { og_err "wasm-size-report: no $module"; exit 1; }
for t in wasm-bindgen brotli bun; do og_need "$t" || exit 1; done

if [ -n "$keep" ]; then
  work="$keep"; mkdir -p "$work"
else
  work="$(mktemp -d)"; trap 'rm -rf "$work"' EXIT
fi

name=origin-engine
if LC_ALL=C grep -q __wasm_init_tls "$module"; then
  name=origin-engine-lanes
fi
if LC_ALL=C grep -q __wbindgen_placeholder__ "$module"; then
  wasm-bindgen --target web --out-dir "$work/bindgen" --out-name "$name" "$module"
else
  mkdir -p "$work/bindgen"
  cp "$module" "$work/bindgen/${name}_bg.wasm"
fi
bound="$work/bindgen/${name}_bg.wasm"
budget="$(og_budget wasm_core_brotli_bytes)"
br="$(brotli -q 11 -c "$bound" | wc -c | tr -d ' ')"
printf '%s: %s B raw, %s B after bindgen, %s B brotli -q 11 (%s of wasm_core_brotli_bytes, %s B)\n\n' \
  "$(basename "$module")" "$(og_bytes "$module")" "$(og_bytes "$bound")" "$br" \
  "$(awk -v b="$br" -v t="$budget" 'BEGIN { printf "%.1f %%", 100 * b / t }')" "$budget"
bun "$here/wasm-size-report.ts" -n "$top" "$bound"
