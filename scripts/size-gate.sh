#!/usr/bin/env bash
# Size gate: brotli -q 11 size of the optimized engine wasm against the budget
# `wasm_core_brotli_bytes` in scripts/budgets.txt (docs/design.md §7).
#
#   scripts/size-gate.sh [file.wasm ...]
#
# With no arguments it measures every file listed in target/og-wasm/opt.txt,
# which scripts/check.sh's wasm-opt step writes. The budget is per download:
# a page downloads one engine module, never two, so each measured file (the
# 3D publish variant, the 2D one, the lanes variant: docs/web-export.md
# §Variants and §Lanes) is held to it, and the figure that matters is the
# largest of them, not their sum. Writes $OG_EVIDENCE_DIR/size-gate.json
# (each file with its download, and max_brotli_bytes). A module
# measured before (same SHA-256 and brotli) takes its recorded size, unless
# OG_SIZE_GATE_FRESH is set (scripts/check.sh --all sets it).
set -u -o pipefail
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"

budget="$(og_budget wasm_core_brotli_bytes)" || exit 1
artifacts="$(og_budget wasm_artifacts)" || exit 1

files=()
if [ "$#" -gt 0 ]; then
  files=("$@")
elif [ -f "$OG_WASM_OUT/opt.txt" ]; then
  while IFS= read -r f; do [ -n "$f" ] && files+=("$f"); done < "$OG_WASM_OUT/opt.txt"
fi

mkdir -p "$OG_EVIDENCE_DIR"
report="$OG_EVIDENCE_DIR/size-gate.json"

if [ "${#files[@]}" -eq 0 ]; then
  printf '{"budget_brotli_bytes":%s,"total_brotli_bytes":0,"max_brotli_bytes":0,"files":[]}\n' "$budget" > "$report"
  if [ "$artifacts" = required ]; then
    og_err "size-gate: no wasm to measure, and budgets.txt says 'wasm_artifacts required'"
    exit 1
  fi
  og_say "size-gate: no wasm artifacts to measure yet (no crate in scripts/wasm-crates.txt has a cdylib target); budget ${budget} B brotli"
  exit 0
fi

og_need brotli || exit 1

total=0
max=0
over=""
entries=""
# What a player downloads it as (scripts/wasm-crates.txt names the variants).
download_of() {
  case "$1" in
    *-publish.wasm) echo "3D publish" ;;
    *-publish2d.wasm) echo "2D publish" ;;
    *-lanes.wasm) echo "lanes" ;;
    *) echo "-" ;;
  esac
}
printf '%-28s %-11s %12s %12s\n' "wasm" "download" "bytes" "brotli-q11"
for f in "${files[@]}"; do
  if [ ! -f "$f" ]; then og_err "size-gate: $f does not exist"; exit 1; fi
  raw="$(og_bytes "$f")"
  # brotli -q 11 of a module measured before is that size again.
  key="$(og_sha256 "$f")-$(brotli --version 2>/dev/null | tr -c 'A-Za-z0-9.' _)"
  cached="$OG_CONTENT_CACHE/brotli/$key"
  if [ -z "${OG_SIZE_GATE_FRESH:-}" ] && [ -s "$cached" ]; then
    br="$(cat "$cached")"
  else
    br="$(brotli -q 11 -c "$f" | wc -c | tr -d ' ')"
    mkdir -p "$OG_CONTENT_CACHE/brotli" && printf '%s\n' "$br" > "$cached"
  fi
  total=$((total + br))
  name="$(basename "$f")"
  [ "$br" -le "$max" ] || max="$br"
  [ "$br" -le "$budget" ] || over="${over}${over:+, }$name ${br} B (+$((br - budget)) B)"
  download="$(download_of "$name")"
  printf '%-28s %-11s %12s %12s  (%s%% of the budget)\n' "$name" "$download" "$raw" "$br" "$((br * 100 / budget))"
  entries="${entries}${entries:+,}{\"file\":\"$name\",\"download\":\"$download\",\"bytes\":$raw,\"brotli_bytes\":$br}"
done
pct=$((max * 100 / budget))
printf '%-28s %-11s %12s %12s  (the largest download, %s%% of the per-download budget %s)\n' "max" "" "" "$max" "$pct" "$budget"
printf '{"budget_brotli_bytes":%s,"total_brotli_bytes":%s,"max_brotli_bytes":%s,"files":[%s]}\n' "$budget" "$total" "$max" "$entries" > "$report"

if [ -n "$over" ]; then
  og_err "size-gate: over wasm_core_brotli_bytes ${budget} B: $over (docs/ci.md §When the size gate fails)"
  exit 1
fi
