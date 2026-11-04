#!/usr/bin/env bash
# The gate's steps that read the wasm build's outputs fail when those are
# missing or partial, naming the file, and never shrink to a subset that
# passes (2026-11-02: an interrupted wasm build left no opt-all.txt, and
# bun-functional ran only knowledge and create and reported PASS).
#
#   scripts/test-gate-guards.sh
#
# Each case runs scripts/check.sh STEP against a scratch target directory
# (CARGO_TARGET_DIR, with its own machine lock), so it builds nothing,
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
  local args=("$2")
  [ "${4:-}" != --only ] || args=(--only "$2")
  t="$(layout "$1")"
  out="$(cd "$root" && PATH="$scratch/bin:$PATH" CARGO_TARGET_DIR="$t" OG_GATE_MACHINE_DIR="$t/machine" OG_GATE_JOBS=1 \
    scripts/check.sh "${args[@]}" 2>&1)"; rc=$?
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

for step in bun-functional bun-test size-gate; do
  expect_fail none "$step" "og-wasm/opt-all.txt (absent)"
  expect_fail no-opt-all "$step" "og-wasm/opt-all.txt (absent)"
  expect_fail empty-opt-all "$step" "og-wasm/opt-all.txt (empty)"
  expect_fail no-opt "$step" "og-wasm/opt.txt (absent)"
  expect_fail module-gone "$step" "opt/og_host_web.wasm (named in"
done

expect_fail none bun-functional "og-wasm/opt-all.txt (absent)" --only
expect_fail module-gone bun-functional "opt/og_host_web.wasm (named in" --only

t="$(layout none)"
out="$(cd "$root" && PATH="$scratch/bin:$PATH" CARGO_TARGET_DIR="$t" OG_GATE_MACHINE_DIR="$t/machine" OG_GATE_JOBS=2 \
  scripts/check.sh --only bun-functional wasm-opt 2>&1)"; rc=$?
if [ "$rc" -eq 0 ] || ! printf '%s' "$out" | grep -qF 'unexpected cargo invocation: build' \
  || printf '%s' "$out" | grep -qF 'bun-functional started'; then
  echo "FAIL explicitly selected wasm: bun-functional must wait for the refreshed module"
  printf '%s\n' "$out" | tail -10
  failures=$((failures + 1))
else
  echo "ok   explicitly selected wasm: bun-functional waits for the refreshed module"
fi

[ "$failures" -eq 0 ] || { echo "$failures case(s) failed"; exit 1; }
echo "every case failed as it should"
