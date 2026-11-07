#!/usr/bin/env bash
# The gate's steps that read the wasm build's outputs fail when those are
# missing or partial, naming the file, and never shrink to a subset that
# passes (2026-11-02: an interrupted wasm build left no opt-all.txt, and
# bun-functional ran only knowledge and create and reported PASS).
#
#   scripts/test-gate-guards.sh
#
# Each case runs the gate command against scratch outputs, so it builds nothing,
# touches no real target/ and waits on no running gate. A few seconds.
set -u -o pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
failures=0

mkdir -p "$scratch/bin"
cat > "$scratch/bin/cargo" <<'EOF'
#!/usr/bin/env bash
echo "unexpected cargo invocation: $*" >&2
exit 97
EOF
chmod +x "$scratch/bin/cargo"

# The scheduler now always adds prerequisites. Exercise the consuming command
# directly to simulate interrupted output without rebuilding that output first.
run_step() (
  . "$root/scripts/lib.sh"
  . "$root/scripts/gate/commands.sh"
  FORCE_ALL=0 SKIP=3 RAN=" " OG_GATE_TREE="" OG_GATE_THREADS=2
  [ "${1:-}" != --all ] || { FORCE_ALL=1; shift; }
  skip_unchanged() { return 1; }
  "step_$(printf '%s' "$1" | tr '-' '_')"
)

# layout NAME: a scratch target whose og-wasm is complete, then broken by NAME.
layout() {
  local t="$scratch/$1" w mode="${2:-opt}"
  w="$t/og-wasm"
  rm -rf "$t"; mkdir -p "$w/$mode"
  for m in og_host_web og_host_web-publish; do printf 'wasm' > "$w/$mode/$m.wasm"; done
  printf '%s\n' "$w/$mode/og_host_web.wasm" "$w/$mode/og_host_web-publish.wasm" > "$w/$mode-all.txt"
  printf '%s\n' "$w/$mode/og_host_web-publish.wasm" > "$w/$mode.txt"
  case "$1" in
    none) rm -rf "$w" ;;
    no-opt-all) rm -f "$w/$mode-all.txt" ;;
    empty-opt-all) : > "$w/$mode-all.txt" ;;
    no-opt) rm -f "$w/$mode.txt" ;;
    module-gone) rm -f "$w/$mode/og_host_web.wasm" ;;
  esac
  printf '%s\n' "$t"
}

# expect_fail CASE STEP NAMED: STEP fails on layout CASE, its log names NAMED,
# and it ran no package's tests.
expect_fail() {
  local t out rc mode=opt
  [ "$2" != bun-test ] || mode=dev
  t="$(layout "$1" "$mode")"
  out="$(cd "$root" && PATH="$scratch/bin:$PATH" CARGO_TARGET_DIR="$t" run_step "$2" 2>&1)"; rc=$?
  if [ "$rc" -eq 0 ]; then
    echo "FAIL $2 with $1: passed"; failures=$((failures + 1)); return
  fi
  if ! printf '%s' "$out" | grep -qF -- "$3"; then
    echo "FAIL $2 with $1: exit $rc without naming $3:"; printf '%s\n' "$out" | tail -5
    failures=$((failures + 1)); return
  fi
  if printf '%s' "$out" | grep -q 'bun run test'; then
    echo "FAIL $2 with $1: ran package tests before failing"; failures=$((failures + 1)); return
  fi
  echo "ok   $2 with $1: fails naming $3"
}

for step in bun-test size-gate; do
  mode=opt
  [ "$step" != bun-test ] || mode=dev
  expect_fail none "$step" "og-wasm/$mode-all.txt (absent)"
  expect_fail no-opt-all "$step" "og-wasm/$mode-all.txt (absent)"
  expect_fail empty-opt-all "$step" "og-wasm/$mode-all.txt (empty)"
  expect_fail no-opt "$step" "og-wasm/$mode.txt (absent)"
  expect_fail module-gone "$step" "$mode/og_host_web.wasm (named in"
done

# Functional preparation preserves bytes, even for publish variants, without
# needing Binaryen installed. Release size checks still require opt's lists.
cp "$scratch/bin/cargo" "$scratch/bin/wasm-opt"
t="$scratch/development"; w="$t/og-wasm"
mkdir -p "$w/raw"
for name in og_host_web og_host_web-publish; do
  printf 'unoptimized-%s' "$name" > "$w/raw/$name.wasm"
  printf '%s\n' "$w/raw/$name.wasm" >> "$w/built.txt"
done
(CARGO_TARGET_DIR="$t" OG_WASM_CACHE="$scratch/cache" PATH="$scratch/bin:$PATH" run_step --all wasm-bindgen) || exit 1
for name in og_host_web og_host_web-publish; do
  cmp "$w/raw/$name.wasm" "$w/dev/$name.wasm" || exit 1
done
[ ! -e "$w/opt-all.txt" ] || exit 1
echo 'ok   functional wasm preparation preserves every variant without wasm-opt'

steps="$(bash "$root/scripts/check.sh" --list)" || exit 1
case "$steps" in *wasm-opt*|*size-gate*) echo 'FAIL release measurements exposed as gate steps'; exit 1 ;; esac
echo 'ok   no gate step selects release optimization or size measurements'

[ "$failures" -eq 0 ] || { echo "$failures case(s) failed"; exit 1; }
echo "every case failed as it should"
