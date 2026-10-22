#!/usr/bin/env bash
# Wave 0.2 of goals/tsrs/plan.md: tsgo 7.0.2 against tsrs at equal checker
# threads. Each cell: 5 runs of `-p <config> --noEmit --incremental false
# --pretty false --checkers N`, median and range of the wall clock (ms) and
# the peak RSS (MB) of the checker process. Deleted with the package (Wave 3.5).
#
#   TSGO=<tsgo 7.0.2 native tsc> TSRS=<tsrs> goals/tsrs/evidence/bench.sh
#
# TSGO is the native binary of typescript@7.0.2 (node_modules/@typescript/
# typescript-<platform>/lib/tsc), not `bun x`, so neither side pays a JS
# launcher. Prints a Markdown table.
set -eu -o pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
TSGO="${TSGO:?set TSGO to the typescript@7.0.2 native tsc}"
TSRS="${TSRS:-tsrs}"
RUNS="${RUNS:-5}"
CHECKERS="${CHECKERS:-1 2 4 8}"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT

# templates/game has no `paths` (a created project resolves the engine from
# node_modules): the wrapper og check synthesises, by hand.
mkdir -p "$tmp/template"
cat > "$tmp/template/tsconfig.json" <<EOF
{ "extends": "$ROOT/templates/game/tsconfig.json",
  "compilerOptions": { "paths": {
    "@origingame/engine": ["$ROOT/packages/engine/types/index.d.ts"],
    "@origingame/engine/playtest": ["$ROOT/packages/engine/src/playtest.ts"] } } }
EOF
# Startup: one file, no imports.
mkdir -p "$tmp/empty"
echo 'export const x: number = 1;' > "$tmp/empty/main.ts"
echo '{ "compilerOptions": { "strict": true, "noEmit": true, "types": [], "lib": ["ES2022"], "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler" }, "include": ["main.ts"] }' > "$tmp/empty/tsconfig.json"

targets=(
  "engine:types=$ROOT/packages/engine/tsconfig.json"
  "engine:runtime=$ROOT/packages/engine/tsconfig.runtime.json"
  "engine:templates=$ROOT/packages/engine/tsconfig.templates.json"
  "templates/game=$tmp/template/tsconfig.json"
)
for e in brasswick hollowbell kestrel-stair lantern-vale meridian-loop; do
  targets+=("examples/$e=$ROOT/examples/$e/tsconfig.json")
done
targets+=("startup (1 file)=$tmp/empty/tsconfig.json")

# cell TOOL CONFIG N: "median_ms (min–max) | median_MB (min–max)"; wall
# clock by perf_counter around the child, peak RSS by its own rusage (wait4).
cell() {
  python3 -I - "$RUNS" "$@" <<'PY'
import os, statistics, subprocess, sys, time
runs, tool, config, n = int(sys.argv[1]), sys.argv[2], sys.argv[3], sys.argv[4]
ms, mb = [], []
for _ in range(runs):
    t0 = time.perf_counter()
    p = subprocess.Popen([tool, "-p", config, "--noEmit", "--incremental", "false",
                          "--pretty", "false", "--checkers", n],
                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    _, _, ru = os.wait4(p.pid, 0)
    ms.append((time.perf_counter() - t0) * 1000)
    mb.append(ru.ru_maxrss / 1024)
print(f"{statistics.median(ms):.0f} ({min(ms):.0f}–{max(ms):.0f}) | "
      f"{statistics.median(mb):.0f} ({min(mb):.0f}–{max(mb):.0f})")
PY
}

echo "machine: $(uname -sm), nproc $(nproc), load $(cut -d' ' -f1-3 /proc/loadavg 2>/dev/null || uptime)"
echo "tsgo: $("$TSGO" --version); tsrs: $("$TSRS" --version)"
echo "runs per cell: $RUNS"
echo
printf '| target | checkers | tsgo ms (range) | tsgo MB (range) | tsrs ms (range) | tsrs MB (range) |\n'
printf '|---|---|---|---|---|---|\n'
for t in "${targets[@]}"; do
  name="${t%%=*}"; config="${t#*=}"
  for n in $CHECKERS; do
    printf '| %s | %s | %s | %s |\n' "$name" "$n" "$(cell "$TSGO" "$config" "$n")" "$(cell "$TSRS" "$config" "$n")"
  done
done
