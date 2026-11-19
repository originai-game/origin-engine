#!/usr/bin/env bash
# The gate's steps that read the wasm build's outputs fail when those are
# missing or partial, naming the file, and never shrink to a subset that
# passes (2026-11-02: an interrupted wasm build left no output list, and
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
  export OG_EVIDENCE_DIR="$CARGO_TARGET_DIR/evidence"
  . "$root/scripts/lib.sh"
  . "$root/scripts/gate/commands.sh"
  SKIP=3 OG_GATE_TREE=""
  skip_unchanged() { return 1; }
  "step_$(printf '%s' "$1" | tr '-' '_')"
)

# layout NAME: a scratch target whose og-wasm is complete, then broken by NAME.
layout() {
  local t="$scratch/$1" w mode=dev
  w="$t/og-wasm"
  rm -rf "$t"; mkdir -p "$w/$mode"
  for m in og_host_web og_host_web-publish; do printf 'wasm' > "$w/$mode/$m.wasm"; done
  printf '%s\n' "$w/$mode/og_host_web.wasm" "$w/$mode/og_host_web-publish.wasm" > "$w/$mode-all.txt"
  printf '%s\n' "$w/$mode/og_host_web-publish.wasm" > "$w/$mode.txt"
  case "$1" in
    none) rm -rf "$w" ;;
    no-all) rm -f "$w/$mode-all.txt" ;;
    empty-all) : > "$w/$mode-all.txt" ;;
    no-shipping) rm -f "$w/$mode.txt" ;;
    module-gone) rm -f "$w/$mode/og_host_web.wasm" ;;
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
  expect_fail none "$step" "og-wasm/dev-all.txt (absent)"
  expect_fail no-all "$step" "og-wasm/dev-all.txt (absent)"
  expect_fail empty-all "$step" "og-wasm/dev-all.txt (empty)"
  expect_fail no-shipping "$step" "og-wasm/dev.txt (absent)"
  expect_fail module-gone "$step" "dev/og_host_web.wasm (named in"
done

# Preparation preserves bytes, even for publish variants.
t="$scratch/development"; w="$t/og-wasm"
mkdir -p "$w/raw"
for name in og_host_web og_host_web-publish; do
  printf 'unoptimized-%s' "$name" > "$w/raw/$name.wasm"
  printf '%s\n' "$w/raw/$name.wasm" >> "$w/built.txt"
done
(CARGO_TARGET_DIR="$t" PATH="$scratch/bin:$PATH" run_step wasm-bindgen) || exit 1
for name in og_host_web og_host_web-publish; do
  cmp "$w/raw/$name.wasm" "$w/dev/$name.wasm" || exit 1
done
echo 'ok   wasm preparation preserves every variant'

# The release size check measures the selected shipping bindgen bytes, not
# the development host or a stale optimized list from an earlier release.
printf '%s\n' "$scratch/missing-optimized.wasm" > "$w/opt.txt"
(CARGO_TARGET_DIR="$t" run_step size-gate) || exit 1
bun -e '
  const fs = require("node:fs");
  const report = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
  const files = report.files;
  if (report.budget_brotli_bytes !== 16777216 || files.length !== 1 ||
      files[0].file !== "og_host_web-publish.wasm" ||
      files[0].bytes !== fs.statSync(process.argv[2]).size) process.exit(1);
' "$t/evidence/size-gate.json" "$w/dev/og_host_web-publish.wasm" || exit 1
echo 'ok   size gate measures only shipped bindgen bytes with the 16 MiB budget'

steps="$(bash "$root/scripts/check.sh" --list)" || exit 1
case "$steps" in *wasm-opt*|*size-gate*) echo 'FAIL release measurements exposed as gate steps'; exit 1 ;; esac
echo 'ok   no gate step selects release size measurements'

# A landing (dev) builds the development host alone; reference groups add
# only the 3D publish pair; every variant is built for the browser tests and
# release packing (all).
for variants in all reference dev; do
  (
    . "$root/scripts/lib.sh"
    . "$root/scripts/gate/commands.sh"
    OG_WASM_OUT="$scratch/selected-$variants"
    OG_WASM_VARIANTS="$variants" WASM_PROFILE_DEFAULT=wasm-test
    WASM_TRIPLE=wasm32-unknown-unknown SKIP=3
    skip_unchanged() { return 1; }
    wasm_build() {
      local dest="$OG_WASM_OUT/${1:-dev}.wasm"
      printf 'module' > "$dest"
      printf '%s\n' "$dest" >> "$OG_WASM_OUT/built-${1:-dev}.txt"
    }
    step_wasm_build >/dev/null || exit 1
    case "$variants" in all) expected="dev publish publish2d publishframes publish2dframes lanes lanes2d lanesframes lanes2dframes" ;; reference) expected="dev lanes publish" ;; dev) expected=dev ;; esac
    [ "$(ls "$OG_WASM_OUT"/*.wasm | xargs -n1 basename | sed 's/\.wasm$//' | sort | xargs)" = "$(printf '%s\n' $expected | sort | xargs)" ] || exit 1
    [ "$(wc -l < "$OG_WASM_OUT/built.txt" | tr -d ' ')" = "$(printf '%s\n' $expected | wc -l | tr -d ' ')" ] || exit 1
  ) || { echo "FAIL OG_WASM_VARIANTS=$variants builds the wrong modules"; exit 1; }
done
echo 'ok   landings build only development wasm; references omit 2D and frames variants; all builds include frames'

# OG_GATE_SKIP drops named steps: an unknown name fails before anything runs,
# and a skipped step is printed and summarised as not run, never as a pass.
printf '%s\n' '#!/usr/bin/env bash' '[ "$1" = metadata ] && { echo "{}"; exit 0; }' 'exit 97' > "$scratch/bin/cargo"
skip_run() (cd "$root" && OG_GATE_SKIP="$1" OG_EVIDENCE_DIR="$scratch/skip" PATH="$scratch/bin:$PATH" python3 scripts/gate/run.py --only "$2" 2>&1)
out="$(skip_run fmt,no-such-step fmt)" && { echo 'FAIL OG_GATE_SKIP accepted an unknown step'; exit 1; }
case "$out" in *no-such-step*) ;; *) echo "FAIL OG_GATE_SKIP's refusal does not name the step: $out"; exit 1 ;; esac
out="$(skip_run ' clippy,fmt' clippy)" || { echo "FAIL OG_GATE_SKIP run: $out"; exit 1; }
case "$out" in
  *'SKIPPED fmt (OG_GATE_SKIP)'*'gate PASSED (0/2 steps'*'NOT RUN (OG_GATE_SKIP): fmt, clippy)'*) ;;
  *) echo "FAIL OG_GATE_SKIP's skips are not printed as not run: $out"; exit 1 ;;
esac
[ "$(cat "$scratch/skip/check-summary.txt")" = "$(printf 'SKIPPED fmt (OG_GATE_SKIP)\nSKIPPED clippy (OG_GATE_SKIP)')" ] ||
  { echo 'FAIL the gate summary does not record the skips'; exit 1; }
echo 'ok   OG_GATE_SKIP refuses unknown steps and reports skips as not run'

# Engine type errors must block both lanes without needing wasm or a GPU.
python3 - "$root/scripts/gate/run.py" <<'PY'
import runpy
import sys
gate = runpy.run_path(sys.argv[1])
assert 'engine-typecheck' in gate['QUICK']
assert 'engine-typecheck' in gate['FULL']
assert gate['NEEDS']['engine-typecheck'] == ('rust-build',)
assert 'engine-typecheck' in gate['NEEDS']['bun-test']
assert 'preflight' in gate['NEEDS']['rust-build']
assert 'api-surface' in gate['NEEDS']['bun-test']
assert 'web-ready' in gate['NEEDS']['browser']
assert 'bun-test' not in gate['NEEDS']['browser']
assert not {'rust-cpu', 'rust-gpu'}.intersection(gate['NEEDS']['bun-test'])
assert {'bun-test', 'browser', 'fixtures'} <= gate['PROJECT_STEPS']
PY
if [ "$?" -ne 0 ]; then echo 'FAIL engine typecheck lane coverage'; exit 1; fi
t="$scratch/typecheck"
mkdir -p "$t/debug"
cat > "$t/debug/og" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$OG_TYPECHECK_CALLS"
[ "${OG_TYPECHECK_FAIL:-}" != "$2" ]
EOF
chmod +x "$t/debug/og"
for bad in none tsconfig.runtime.json; do
  calls="$scratch/typecheck-$bad.calls"
  out="$(CARGO_TARGET_DIR="$t" OG_TYPECHECK_CALLS="$calls" OG_TYPECHECK_FAIL="$bad" run_step engine-typecheck 2>&1)"; rc=$?
  if { [ "$bad" = none ] && [ "$rc" -ne 0 ]; } || { [ "$bad" != none ] && [ "$rc" -eq 0 ]; }; then
    echo "FAIL engine-typecheck with $bad: $out"; exit 1
  fi
  [ "$(cat "$calls")" = "$(printf 'check tsconfig.json\ncheck tsconfig.runtime.json\ncheck tsconfig.templates.json')" ] ||
    { echo 'FAIL engine-typecheck did not check all configurations'; exit 1; }
done
echo 'ok   engine typecheck covers quick/full and propagates runtime errors without wasm'

# OG_GATE selects dev outputs, not permission to copy a stale addon. Only
# a successful rust-build in this gate lets consumers skip the cargo build.
t="$scratch/native-build"
mkdir -p "$t/packages/engine/scripts" "$t/scripts" "$t/bin" "$t/target/debug"
cp "$root/packages/engine/scripts/build.sh" "$t/packages/engine/scripts/build.sh"
: > "$t/scripts/lock.sh"
printf '%s\n' '#!/bin/sh' 'exit 0' > "$t/bin/bun"
printf '%s\n' '#!/bin/sh' 'echo linux-x64' > "$t/bin/node"
cat > "$t/bin/cargo" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >> "$CARGO_TARGET_DIR/cargo.calls"
[ "${FAIL_NATIVE_BUILD:-0}" != 1 ] || exit 97
echo fresh-addon > "$CARGO_TARGET_DIR/debug/libog_host_node.so"
echo fresh-og > "$CARGO_TARGET_DIR/debug/og"
EOF
chmod +x "$t/bin/"*
for ran in '' ' rust-build '; do
  rm -f "$t/target/cargo.calls"
  echo old-addon > "$t/target/debug/libog_host_node.so"
  echo old-og > "$t/target/debug/og"
  PATH="$t/bin:$PATH" CARGO_TARGET_DIR="$t/target" OG_GATE=1 OG_GATE_RAN="$ran" OG_RELEASE= \
    sh "$t/packages/engine/scripts/build.sh" --native || exit 1
  if [ -z "$ran" ]; then
    [ "$(cat "$t/target/cargo.calls")" = 'build -q -p og-host-node -p og-cli' ] &&
      [ "$(cat "$t/packages/engine/native/origin-engine.linux-x64.node")" = fresh-addon ] &&
      [ "$(cat "$t/packages/engine/native/og.linux-x64")" = fresh-og ] ||
      { echo 'FAIL standalone dev build reused stale native outputs'; exit 1; }
  else
    [ ! -f "$t/target/cargo.calls" ] || { echo 'FAIL gate consumer rebuilt native outputs'; exit 1; }
  fi
done
PATH="$t/bin:$PATH" CARGO_TARGET_DIR="$t/target" OG_GATE=1 OG_GATE_RAN= OG_RELEASE= FAIL_NATIVE_BUILD=1 \
  sh "$t/packages/engine/scripts/build.sh" --native && { echo 'FAIL native build error was ignored'; exit 1; }
echo 'ok   standalone dev builds refresh native outputs; gate consumers reuse their completed rust-build'

# The milestone may suppress its own checkout hooks, never the gate's:
# nested landing fixtures use hooks to simulate outside writers and rejection.
t="$scratch/milestone"
mkdir -p "$t/repo/scripts" "$t/repo/.agents" "$t/bin"
cp "$root/scripts/gpu-milestone.sh" "$t/repo/scripts/"
printf 'export VK_ICD_FILENAMES=/dev/null CHROME=%q\n' "$(type -P true)" > "$t/repo/scripts/gpu-env.sh"
: > "$t/repo/.agents/environment"
printf 'og_gpu_acquire() { :; }\nog_gpu_acquire_all() { :; }\nog_lock_release() { :; }\n' > "$t/repo/scripts/lock.sh"
printf '#!/bin/sh\nexit 0\n' > "$t/repo/scripts/clean-targets.sh"
mkdir -p "$t/repo/packages/engine/scripts"
printf '/target/\n/packages/engine/native/\n' > "$t/repo/.gitignore"
cat > "$t/repo/packages/engine/scripts/build.sh" <<'EOF'
#!/bin/sh
cd "$(dirname "$0")/../../.." || exit 1
[ "${MILESTONE_NATIVE_FAIL:-0}" = 0 ] || exit 97
[ "$OG_RELEASE" = 1 ] && [ "${OG_GATE:-0}" != 1 ] || exit 98
cargo build || exit 1
mkdir -p packages/engine/native
touch packages/engine/native/addon-ready
EOF
cat > "$t/repo/scripts/check.sh" <<'EOF'
#!/usr/bin/env bash
set -eu
# A separate repository owns its own hook, just as test-landing.py does.
git init -q "$MILESTONE_FIXTURE/nested"
git -C "$MILESTONE_FIXTURE/nested" config user.name Test
git -C "$MILESTONE_FIXTURE/nested" config user.email test@example.invalid
git -C "$MILESTONE_FIXTURE/nested" config commit.gpgsign false
git -C "$MILESTONE_FIXTURE/nested" config core.hooksPath "$MILESTONE_FIXTURE/nested-hooks"
mkdir -p "$MILESTONE_FIXTURE/nested-hooks"
printf '#!/bin/sh\ntouch "$MILESTONE_FIXTURE/hook-ran"\n' > "$MILESTONE_FIXTURE/nested-hooks/pre-commit"
chmod +x "$MILESTONE_FIXTURE/nested-hooks/pre-commit"
git -C "$MILESTONE_FIXTURE/nested" commit -q --allow-empty -m fixture
EOF
chmod +x "$t/repo/scripts/check.sh"
git -C "$t/repo" init -q
git -C "$t/repo" config user.name Test
git -C "$t/repo" config user.email test@example.invalid
git -C "$t/repo" -c core.hooksPath=/dev/null -c commit.gpgsign=false add .
git -C "$t/repo" -c core.hooksPath=/dev/null -c commit.gpgsign=false commit -qm fixture
head=$(git -C "$t/repo" rev-parse HEAD)
git -C "$t/repo" update-ref refs/remotes/origin/main "$head"
mkdir -p "$t/runner-hooks"
printf '#!/bin/sh\necho unexpected-checkout-hook >&2\nexit 99\n' > "$t/runner-hooks/post-checkout"
chmod +x "$t/runner-hooks/post-checkout"
git -C "$t/repo" config core.hooksPath "$t/runner-hooks"
# No network, historical builds, GPU work or infinite service sleep.
MILESTONE_REAL_GIT="$(command -v git)"
export MILESTONE_REAL_GIT MILESTONE_FIXTURE="$t"
cat > "$t/bin/git" <<'EOF'
#!/usr/bin/env bash
for arg in "$@"; do
  case "$arg" in fetch) exit 0 ;; 2fb1c4819|6cc20b167) exit 0 ;; esac
done
exec "$MILESTONE_REAL_GIT" "$@"
EOF
cat > "$t/bin/cargo" <<'EOF'
#!/bin/sh
[ "$1" = build ] || exit 1
mkdir -p target/release
cat > target/release/og <<'OG'
#!/bin/sh
[ -f packages/engine/native/addon-ready ] || exit 97
echo screen >> "$MILESTONE_FIXTURE/screens"
OG
chmod +x target/release/og
EOF
printf '#!/bin/sh\nexit 0\n' > "$t/bin/sleep"
chmod +x "$t/bin/"*
for run in new existing; do
  rm -f "$t/hook-ran" "$t/screens"
  PATH="$t/bin:$PATH" bash "$t/repo/scripts/gpu-milestone.sh" > "$t/run.log" 2>&1 ||
    { cat "$t/run.log"; echo "FAIL milestone with $run worktrees"; exit 1; }
  [ -f "$t/hook-ran" ] || { echo 'FAIL milestone disabled nested gate hooks'; exit 1; }
  [ -f "$t/screens" ] && [ "$(wc -l < "$t/screens")" -eq 4 ] ||
    { echo 'FAIL milestone screens did not build their native addon first'; exit 1; }
done
echo 'ok   milestone suppresses checkout hooks only; nested gate hooks run'
rm -f "$t/screens" "$t/l4-gate/packages/engine/native/addon-ready"
PATH="$t/bin:$PATH" MILESTONE_NATIVE_FAIL=1 bash "$t/repo/scripts/gpu-milestone.sh" > "$t/run.log" 2>&1 ||
  { cat "$t/run.log"; echo 'FAIL milestone with failed native build'; exit 1; }
[ ! -f "$t/screens" ] || { echo 'FAIL milestone ran screens after native build failed'; exit 1; }
echo 'ok   milestone builds native before screens and skips them on build failure'

[ "$failures" -eq 0 ] || { echo "$failures case(s) failed"; exit 1; }
echo "every case failed as it should"
