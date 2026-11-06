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
  local t="$scratch/$1" w
  w="$t/og-wasm"
  rm -rf "$t"; mkdir -p "$w/opt"
  for m in og_host_web og_host_web-publish; do printf 'wasm' > "$w/opt/$m.wasm"; done
  printf '%s\n' "$w/opt/og_host_web.wasm" "$w/opt/og_host_web-publish.wasm" > "$w/opt-all.txt"
  printf '%s\n' "$w/opt/og_host_web-publish.wasm" > "$w/opt.txt"
  case "$1" in
    none) rm -rf "$w" ;;
    no-opt-all) rm -f "$w/opt-all.txt" ;;
    empty-opt-all) : > "$w/opt-all.txt" ;;
    no-opt) rm -f "$w/opt.txt" ;;
    module-gone) rm -f "$w/opt/og_host_web.wasm" ;;
  esac
  printf '%s\n' "$t"
}

# expect_fail CASE STEP NAMED: STEP fails on layout CASE, its log names NAMED,
# and it ran no package's tests.
expect_fail() {
  local t out rc
  t="$(layout "$1")"
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
  expect_fail none "$step" "og-wasm/opt-all.txt (absent)"
  expect_fail no-opt-all "$step" "og-wasm/opt-all.txt (absent)"
  expect_fail empty-opt-all "$step" "og-wasm/opt-all.txt (empty)"
  expect_fail no-opt "$step" "og-wasm/opt.txt (absent)"
  expect_fail module-gone "$step" "opt/og_host_web.wasm (named in"
done

# Use real wasm-opt output: development may bypass it only when the size
# list selects publish variants. The no-publish fallback must not.
if command -v wasm-as >/dev/null && command -v wasm-opt >/dev/null; then
  t="$scratch/optimisation"; w="$t/og-wasm"
  mkdir -p "$w/raw"
  printf '%s\n' '(module (func (export "answer") (result i32) (i32.add (i32.const 1) (i32.const 2))))' > "$scratch/input.wat"
  wasm-as "$scratch/input.wat" -o "$w/raw/og_host_web.wasm" || exit 1
  cp "$w/raw/og_host_web.wasm" "$w/raw/og_host_web-publish.wasm"
  "$root/scripts/wasm-opt.sh" "$w/raw/og_host_web.wasm" "$scratch/expected.wasm" || exit 1
  if cmp -s "$scratch/expected.wasm" "$w/raw/og_host_web.wasm"; then
    echo 'FAIL optimisation fixture does not distinguish optimised output'; exit 1
  fi
  printf '%s\n' "$w/raw/og_host_web.wasm" "$w/raw/og_host_web-publish.wasm" > "$w/built.txt"
  for mode in dev full dev-again fallback; do
    args=(wasm-opt)
    [ "$mode" != full ] || args=(--all wasm-opt)
    [ "$mode" != fallback ] || printf '%s\n' "$w/raw/og_host_web.wasm" > "$w/built.txt"
    # Invalidate the step's path cache, not the content cache: full -> dev
    # and dev -> fallback must never reuse the wrong optimisation mode.
    rm -f "$t/og-gate/green/wasm-opt"
    if ! (cd "$root" && PATH="$scratch/bin:$PATH" CARGO_TARGET_DIR="$t" OG_GATE_MACHINE_DIR="$t/machine" \
      run_step "${args[@]}") > "$scratch/$mode.log" 2>&1; then
      cat "$scratch/$mode.log"; exit 1
    fi
    expected="$scratch/expected.wasm"
    case "$mode" in dev|full|dev-again) expected="$w/raw/og_host_web.wasm" ;; esac
    cmp "$expected" "$w/opt/og_host_web.wasm" || exit 1
    while IFS= read -r module; do
      cmp "$scratch/expected.wasm" "$module" || exit 1
    done < "$w/opt.txt"
    echo "ok   $mode: development output and size-gate input retain their contracts"
  done
else
  echo 'SKIP optimisation cases: wasm-as and wasm-opt required'
fi

[ "$failures" -eq 0 ] || { echo "$failures case(s) failed"; exit 1; }
echo "every case failed as it should"
