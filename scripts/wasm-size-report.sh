#!/usr/bin/env bash
# Where a wasm module's bytes are, per crate and per function, measured on
# the module that ships (wasm-bindgen, then scripts/wasm-opt.sh's flags)
# rather than on the raw cargo output; tests/evidence/P4/wasm-size/README.md
# has a report and what it led to.
#
#   scripts/wasm-size-report.sh [-n N] [--keep DIR] MODULE.wasm
#
# MODULE is a cargo-built wasm (the lanes variant is
# target/wasm-mt/wasm32-unknown-unknown/<profile>/og_host_web.wasm, the
# gate's copies target/og-wasm/raw/*.wasm) or one wasm-bindgen already
# wrote. A module with wasm atomics (the lanes variant: it exports
# __wasm_init_tls) is bound as origin-engine-lanes and optimised with
# --threads, as scripts/check.sh's wasm-opt step does.
#
# It runs wasm-opt twice, side by side: with the shipping flags (the
# brotli -q 11 printed first is the size gate's number for this module)
# and with -g as well, which keeps the name section the tables read
# (scripts/wasm-size-report.ts: the per-crate table with each crate's
# brotli alone, then the top N functions, 30 by default). -g changes the
# optimised code by about 0.1 %. --keep DIR keeps the bound, optimised and
# named modules there; `bun scripts/wasm-size-report.ts DIR/named.wasm`
# prints the tables again without rebuilding anything.
#
# Needs wasm-bindgen and wasm-opt (scripts/install-tools.sh), brotli and bun.
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
for t in wasm-bindgen wasm-opt brotli bun; do og_need "$t" || exit 1; done

if [ -n "$keep" ]; then
  work="$keep"; mkdir -p "$work"
else
  work="$(mktemp -d)"; trap 'rm -rf "$work"' EXIT
fi

threads=()
name=origin-engine
if LC_ALL=C grep -q __wasm_init_tls "$module"; then
  threads=(--threads); name=origin-engine-lanes
fi
bound="$module"
if LC_ALL=C grep -q __wbindgen_placeholder__ "$module"; then
  wasm-bindgen --target web --out-dir "$work/bindgen" --out-name "$name" "$module"
  bound="$work/bindgen/${name}_bg.wasm"
fi
# The flags are words.
# shellcheck disable=SC2207
flags=($("$here/wasm-opt.sh" --flags ${threads[@]+"${threads[@]}"}))
wasm-opt "${flags[@]}" "$bound" -o "$work/opt.wasm" &
ship=$!
wasm-opt "${flags[@]}" -g "$bound" -o "$work/named.wasm"
wait "$ship"
budget="$(og_budget wasm_core_brotli_bytes)"
br="$(brotli -q 11 -c "$work/opt.wasm" | wc -c | tr -d ' ')"
printf '%s: %s B raw, %s B after wasm-opt, %s B brotli -q 11 (%s of wasm_core_brotli_bytes, %s B)\n\n' \
  "$(basename "$module")" "$(og_bytes "$module")" "$(og_bytes "$work/opt.wasm")" "$br" \
  "$(awk -v b="$br" -v t="$budget" 'BEGIN { printf "%.1f %%", 100 * b / t }')" "$budget"
bun "$here/wasm-size-report.ts" -n "$top" "$work/named.wasm"
