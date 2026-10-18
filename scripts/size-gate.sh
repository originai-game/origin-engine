#!/usr/bin/env bash
# Size gate: brotli -q 11 size of the optimized engine wasm against the budget
# `wasm_core_brotli_bytes` in scripts/budgets.txt (docs/design.md §7).
#
#   scripts/size-gate.sh [file.wasm ...]
#
# With no arguments it measures every file listed in target/og-wasm/opt.txt,
# which scripts/check.sh's wasm-opt step writes. The budget applies to the sum
# of all measured files. Writes $OG_EVIDENCE_DIR/size-gate.json.
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
  printf '{"budget_brotli_bytes":%s,"total_brotli_bytes":0,"files":[]}\n' "$budget" > "$report"
  if [ "$artifacts" = required ]; then
    og_err "size-gate: no wasm to measure, and budgets.txt says 'wasm_artifacts required'"
    exit 1
  fi
  og_say "size-gate: no wasm artifacts to measure yet (no crate in scripts/wasm-crates.txt has a cdylib target); budget ${budget} B brotli"
  exit 0
fi

og_need brotli || exit 1

total=0
entries=""
printf '%-28s %12s %12s\n' "wasm" "bytes" "brotli-q11"
for f in "${files[@]}"; do
  if [ ! -f "$f" ]; then og_err "size-gate: $f does not exist"; exit 1; fi
  raw="$(og_bytes "$f")"
  # brotli -q 11 of a module measured before is that size again.
  key="$(og_sha256 "$f")-$(brotli --version 2>/dev/null | tr -c 'A-Za-z0-9.' _)"
  cached="$OG_CONTENT_CACHE/brotli/$key"
  if [ -s "$cached" ]; then
    br="$(cat "$cached")"
  else
    br="$(brotli -q 11 -c "$f" | wc -c | tr -d ' ')"
    mkdir -p "$OG_CONTENT_CACHE/brotli" && printf '%s\n' "$br" > "$cached"
  fi
  total=$((total + br))
  name="$(basename "$f")"
  printf '%-28s %12s %12s\n' "$name" "$raw" "$br"
  entries="${entries}${entries:+,}{\"file\":\"$name\",\"bytes\":$raw,\"brotli_bytes\":$br}"
done
pct=$((total * 100 / budget))
printf '%-28s %12s %12s  (%s%% of budget %s)\n' "total" "" "$total" "$pct" "$budget"
printf '{"budget_brotli_bytes":%s,"total_brotli_bytes":%s,"files":[%s]}\n' "$budget" "$total" "$entries" > "$report"

if [ "$total" -gt "$budget" ]; then
  og_err "size-gate: ${total} B brotli exceeds wasm_core_brotli_bytes ${budget} B by $((total - budget)) B (docs/ci.md §When the size gate fails)"
  exit 1
fi
