#!/usr/bin/env bash
# Origin Engine gate; CI runs exactly this (docs/ci.md).
#
#   scripts/check.sh            the path-aware full lane: during a round, after a change
#   scripts/check.sh quick      the quick lane: fmt clippy test-quick drift bun-quick
#   scripts/check.sh STEP...    only the named steps
#   scripts/check.sh --only STEP...
#                               the named steps and the steps whose outputs they read
#                               (test: wasm-build; bun-test: wasm-build wasm-opt), e.g.
#                               after re-recording a reference frame
#   scripts/check.sh --all ...  any of the above with every step and test run and
#                               nothing reused from target/og-wasm-cache (CI always is):
#                               once at the end of a round, never after every edit
#   scripts/check.sh --list     the step names
#
# Steps run concurrently as their inputs allow (step_needs below), each
# with its log in $OG_EVIDENCE_DIR/gate/<step>.log, printed whole when the
# step ends; the first FAIL stops the others and the gate.
# OG_GATE_JOBS=N runs at most N steps at once (1: one at a time, in order).
#
# Path-aware (docs/ci.md §Lanes): a step that passes records the working
# tree it passed on (<target>/og-gate/green/), and a later run skips what
# none of its inputs changed since: the GPU reference tests (og-tests,
# og-ui, og-render and the crates they build on, templates/ui-kit/assets),
# the wasm steps (crates/), drift (the IDL, og-idl, og and the generated
# files), web-reference (og-host-web's and og-tests-web's crates,
# packages/engine, templates/fx-showcase). A change to scripts/, Cargo.toml,
# Cargo.lock, rust-toolchain.toml, .cargo/ or .config/ reruns all of them. Each skip is
# printed with its reason. --all, or CI set in the environment, runs
# everything.
#
# Full lane: fmt clippy wasm-build test wasm-opt size-gate drift bun-test determinism.
# test starts after wasm-build: og-tests' wasm test takes its og-render
# build instead of compiling og-render for wasm32 a second time.
# Quick lane: fmt clippy test-quick drift bun-quick (no og-tests, no wasm,
# no engine package build: the loop while working and the PR signal).
# wasm-opt reads what wasm-build produced, size-gate reads what wasm-opt
# produced and bun-test's build.sh takes the gate's wasm (OG_GATE=1), so
# each starts after the step it reads (--only adds that step). Every native
# step is the dev profile; the wasm steps are the wasm profile
# (release-based), the wasm the size gate measures (AGENTS.md §Build and
# gate discipline).
# Holds the gate lock (scripts/lock.sh, <target>/og-gate.lock) for its whole
# run; a second gate on the same target directory waits for it.
# Honours CARGO_TARGET_DIR. Each step prints PASS, FAIL or SKIPPED (a step
# a FAIL stopped, CANCELLED); the summary, in lane order, lands in
# $OG_EVIDENCE_DIR/check-summary.txt.
set -u -o pipefail
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"
cd "$OG_ROOT" || exit 1

STEPS_FULL="fmt clippy wasm-build test wasm-opt size-gate drift bun-test determinism"
STEPS_QUICK="fmt clippy test-quick drift bun-quick"
STEPS="$STEPS_FULL test-quick bun-quick web-reference"
BUN_QUICK_PACKAGES="packages/knowledge packages/create"   # no native build needed
WASM_TRIPLE=wasm32-unknown-unknown
SKIP=3   # a step returns this to report SKIPPED instead of PASS
FORCE_ALL=0  # --all or CI: no step is skipped for unchanged inputs
RAN=" "      # the steps run (not skipped) so far in this invocation
SKIP_NOTE="" # why a step was skipped, for the summary
PASS_NOTE="" # what a step that passed left out, for the summary

# The inputs of each path-aware rule (og_unchanged in scripts/lib.sh).
# Everything reruns on a change to the build's own configuration.
RULE_COMMON="Cargo.toml Cargo.lock rust-toolchain.toml .cargo .config scripts"
rule_gpu() {
  echo "$(og_crate_dirs -p og-tests -p og-ui -p og-render -e normal,build,dev | og_words) templates/ui-kit/assets $RULE_COMMON"
}
rule_wasm() { echo "crates $RULE_COMMON"; }
# og api check: the IDL, the generator and what it reads, og's api command,
# and every directory it writes (og_idl::OUTPUT_DIRS).
rule_drift() {
  echo "crates/og-api $(og_crate_dirs -p og-idl -e normal,build | og_words) crates/og-cli \
packages/engine/types packages/engine/schema packages/engine/src/generated \
crates/og-host-web/src/generated crates/og-host-node/src/generated packages/knowledge/wiki $RULE_COMMON"
}
rule_web() {
  echo "$(og_crate_dirs -p og-host-web -p og-tests-web --target "$WASM_TRIPLE" -e normal,build | og_words) packages/engine templates/fx-showcase $RULE_COMMON"
}

# skip_unchanged KEY WHAT RULE...: true (and prints why) when KEY may be
# skipped: not forced, and no input in RULE changed since KEY last passed.
skip_unchanged() {
  local key="$1" what="$2"
  shift 2
  [ "$FORCE_ALL" = 1 ] && return 1
  # shellcheck disable=SC2068 # the rule is a word list
  og_unchanged "$key" $@ || return 1
  SKIP_NOTE="no input changed since it passed on tree ${OG_GREEN:0:12}"
  og_say "SKIPPED $what: $SKIP_NOTE (inputs: $*; --all runs it)"
  return 0
}

# The GPU-bound packages; a test step leaves them out when skip_unchanged
# says their inputs did not change.
GPU_FILTER='package(og-tests) | package(og-ui) | package(og-render)'

# fmt and clippy cover every workspace crate except scripts/gate-exclude.txt.
gate_packages() {  # prints "-p a -p b ..." for the included crates
  local all ex c out=""
  all="$(og_workspace_crates)" || return 1
  ex=" $(og_gate_excluded | og_words) "
  for c in $all; do
    case "$ex" in *" $c "*) ;; *) out="$out -p $c" ;; esac
  done
  printf '%s\n' "$out"
}

gate_excluded_note() {
  local ex
  ex="$(og_gate_excluded | og_words)"
  [ -z "$ex" ] || og_say "excluded (scripts/gate-exclude.txt): $ex"
}

step_fmt() {
  local pk
  pk="$(gate_packages)" || return 1
  gate_excluded_note
  # shellcheck disable=SC2086
  cargo fmt --check $pk
}

step_clippy() {
  local ex="" c
  for c in $(og_gate_excluded); do ex="$ex --exclude $c"; done
  gate_excluded_note
  # shellcheck disable=SC2086
  cargo clippy --workspace $ex --all-targets -- -D warnings
}

# run_tests KEY DOC EXCLUDE...: the workspace's tests but the EXCLUDEd
# packages, through cargo nextest (.config/nextest.toml) or else cargo test,
# plus the doctests (nextest runs none) when DOC is 1. The GPU-bound
# packages are left out when their inputs did not change since KEY.gpu
# passed; KEY.gpu is recorded only by a run that included them.
run_tests() {
  local key="$1" doc="$2" gpu=1 rc pk
  shift 2
  local exclude=("$@")
  if skip_unchanged "$key.gpu" "the GPU reference tests ($GPU_FILTER)" "$(rule_gpu)"; then
    gpu=0
    PASS_NOTE="the GPU reference tests skipped, $SKIP_NOTE"
    SKIP_NOTE=""
  else
    og_green_clear "$key.gpu"
  fi
  if command -v cargo-nextest >/dev/null 2>&1; then
    local args=(--workspace)
    for pk in ${exclude[@]+"${exclude[@]}"}; do args+=(--exclude "$pk"); done
    [ "$gpu" = 1 ] || args+=(-E "not ($GPU_FILTER)")
    cargo nextest run "${args[@]}"; rc=$?
    if [ "$rc" -eq 0 ] && [ "$doc" = 1 ]; then cargo test --workspace --doc; rc=$?; fi
  else
    local args=(--workspace)
    for pk in ${exclude[@]+"${exclude[@]}"}; do args+=(--exclude "$pk"); done
    [ "$gpu" = 1 ] || args+=(--exclude og-tests --exclude og-ui --exclude og-render)
    cargo test "${args[@]}"; rc=$?
  fi
  [ "$rc" -eq 0 ] || return "$rc"
  if [ "$gpu" = 1 ]; then og_green_set "$key.gpu"; fi
  return 0
}

# Every crate's tests (gfx in og-tests) and the doctests.
step_test() { run_tests test 1; }

# Every crate's tests but og-tests (gfx, walks, perf-shaped suites).
step_test_quick() { run_tests test-quick 0 og-tests; }

# wasm_build VARIANT FEATURES CRATE...: builds scripts/wasm-crates.txt crates
# for wasm32 and copies each produced .wasm to $OG_WASM_OUT/raw/<name>[-VARIANT].wasm,
# listed in built.txt (cargo keeps one uplifted file per crate, so a second
# feature variant would overwrite the first). Artifacts come from cargo's
# JSON messages, so a fresh (not relinked) cdylib still counts and a stale
# file never does. A VARIANT builds in its own target dir (target/wasm-VARIANT):
# cargo 1.99 keeps one fingerprint per unit whatever its features, so the two
# variants sharing target/wasm32-unknown-unknown found each other's features
# on every run and recompiled and relinked og-host-web, og-runtime and
# og-probe both times (about 29 s on an unrelated change). The variant's
# dependencies come from sccache (or CI's cached target/) the first time.
wasm_build() {
  local variant="$1" features="$2"; shift 2
  local args=() c f name dest
  for c in "$@"; do args+=(-p "$c"); done
  [ -z "$features" ] || args+=(--features "$features")
  [ -z "$variant" ] || args+=(--target-dir "$OG_TARGET_DIR/wasm-$variant")
  cargo build --profile wasm --target "$WASM_TRIPLE" "${args[@]}" --lib \
      --message-format=json-render-diagnostics > "$OG_WASM_OUT/cargo${variant:+-$variant}.json" || return 1
  mkdir -p "$OG_WASM_OUT/raw"
  for f in $(grep '"reason":"compiler-artifact"' "$OG_WASM_OUT/cargo${variant:+-$variant}.json" \
      | grep -o '"[^"]*\.wasm"' | tr -d '"'); do
    name="$(basename "$f" .wasm)"
    dest="$OG_WASM_OUT/raw/$name${variant:+-$variant}.wasm"
    cp "$f" "$dest" || return 1
    printf '%s\n' "$dest" >> "$OG_WASM_OUT/built.txt"
  done
  return 0
}

# outputs_exist LIST: LIST names at least one file and every one it names exists.
outputs_exist() {
  local f
  [ -s "$1" ] || return 1
  while IFS= read -r f; do [ -f "$f" ] || return 1; done < "$1"
}

step_wasm_build() {
  if outputs_exist "$OG_WASM_OUT/built.txt" \
      && skip_unchanged wasm-build "wasm-build (reusing $OG_WASM_OUT/raw)" "$(rule_wasm)"; then
    return $SKIP
  fi
  rm -rf "$OG_WASM_OUT"; mkdir -p "$OG_WASM_OUT"; : > "$OG_WASM_OUT/built.txt"
  if ! rustup target list --installed 2>/dev/null | grep -qx "$WASM_TRIPLE"; then
    og_err "rust target $WASM_TRIPLE is not installed (rust-toolchain.toml lists it; run 'rustup target add $WASM_TRIPLE')"
    return 1
  fi
  local required pending c
  required="$(og_wasm_crates required)"
  pending="$(og_wasm_crates pending)"
  if [ -z "$required$pending" ]; then
    og_say "scripts/wasm-crates.txt lists no crate; nothing to build"
    return 0
  fi
  if [ -n "$required" ]; then
    og_say "required: $(printf '%s\n' "$required" | og_words)"
    # shellcheck disable=SC2086
    wasm_build "" "" $required || return 1
  fi
  for c in $(og_wasm_publish_crates); do
    og_say "publish variant: $c (--features $c/publish)"
    wasm_build publish "$c/publish" "$c" || return 1
  done
  for c in $pending; do
    og_say "pending: $c"
    if wasm_build "" "" "$c"; then
      og_say "NOTE: $c now builds for $WASM_TRIPLE; mark it 'required' in scripts/wasm-crates.txt"
    else
      og_say "WARN: $c does not build for $WASM_TRIPLE yet (pending in scripts/wasm-crates.txt); tolerated"
    fi
  done
  sort -u -o "$OG_WASM_OUT/built.txt" "$OG_WASM_OUT/built.txt"
  if [ ! -s "$OG_WASM_OUT/built.txt" ]; then
    og_say "built as rlib only: no listed crate has crate-type cdylib yet, so no .wasm artifact"
  else
    sed 's/^/artifact: /' "$OG_WASM_OUT/built.txt"
  fi
}

# wasm-bindgen (when the module uses it, at the Cargo.lock version, out-name
# origin-engine as packages/engine ships it) then wasm-opt -Oz; writes
# target/og-wasm/opt/<name>.wasm for every variant (opt-all.txt) and lists
# the shipping ones in opt.txt: the -publish variants when any were built,
# else all. size-gate reads opt.txt; build.sh under OG_GATE=1 reads both.
# wasm_opt_one RAW NAME: wasm-bindgen (when the module uses it) then wasm-opt
# into $OG_WASM_OUT/opt/NAME.wasm.
# The flags are scripts/wasm-opt.sh's, the one invocation build.sh ships with.
WASM_OPT_FLAGS="$("$OG_ROOT/scripts/wasm-opt.sh" --flags)"
wasm_opt_one() {
  local raw="$1" name="$2" input="$1" lock have key cache
  # A raw module byte-identical to one already optimised (an edit that does
  # not reach the wasm: a test, a comment, native-only code) takes the
  # earlier result: same input, tools and flags, same output.
  key="$(og_sha256 "$raw") $(wasm-bindgen --version 2>/dev/null) $(wasm-opt --version 2>/dev/null) $WASM_OPT_FLAGS"
  cache="$OG_CONTENT_CACHE/opt/$name"
  if [ "$FORCE_ALL" != 1 ] && [ -f "$cache/key" ] && [ "$(cat "$cache/key")" = "$key" ] && [ -f "$cache/opt.wasm" ]; then
    rm -rf "$OG_WASM_OUT/bindgen/$name"
    if [ -d "$cache/bindgen" ]; then mkdir -p "$OG_WASM_OUT/bindgen" && cp -R "$cache/bindgen" "$OG_WASM_OUT/bindgen/$name"; fi
    cp "$cache/opt.wasm" "$OG_WASM_OUT/opt/$name.wasm" || return 1
    og_say "$name: $(og_bytes "$raw") B raw, byte-identical to the last optimised module: reused $(og_bytes "$OG_WASM_OUT/opt/$name.wasm") B ($OG_CONTENT_CACHE/opt)"
    return 0
  fi
  if grep -q '__wbindgen_placeholder__' "$raw"; then
    og_need wasm-bindgen || return 1
    lock="$(og_lock_version wasm-bindgen)"
    have="$(wasm-bindgen --version | awk '{print $2}')"
    if [ "$lock" != "$have" ]; then
      og_err "wasm-bindgen $have does not match Cargo.lock $lock (run scripts/install-tools.sh)"
      return 1
    fi
    rm -rf "$OG_WASM_OUT/bindgen/$name"
    wasm-bindgen --target web --out-dir "$OG_WASM_OUT/bindgen/$name" --out-name origin-engine "$raw" || return 1
    input="$OG_WASM_OUT/bindgen/$name/origin-engine_bg.wasm"
  fi
  # The flags and why: scripts/wasm-opt.sh.
  "$OG_ROOT/scripts/wasm-opt.sh" "$input" "$OG_WASM_OUT/opt/$name.wasm" || return 1
  og_say "$name: $(og_bytes "$raw") B raw -> $(og_bytes "$input") B bindgen -> $(og_bytes "$OG_WASM_OUT/opt/$name.wasm") B wasm-opt -Oz"
  rm -rf "$cache"; mkdir -p "$cache"
  [ ! -d "$OG_WASM_OUT/bindgen/$name" ] || cp -R "$OG_WASM_OUT/bindgen/$name" "$cache/bindgen"
  cp "$OG_WASM_OUT/opt/$name.wasm" "$cache/opt.wasm" && printf '%s\n' "$key" > "$cache/key"
}

step_wasm_opt() {
  local built="$OG_WASM_OUT/built.txt" raw name input lock have
  if [ ! -f "$built" ]; then og_err "no $built: run the wasm-build step first"; return 1; fi
  case "$RAN" in *" wasm-build "*) ;; *)
    if outputs_exist "$OG_WASM_OUT/opt-all.txt" \
        && skip_unchanged wasm-opt "wasm-opt (reusing $OG_WASM_OUT/opt)" "$(rule_wasm)"; then
      return $SKIP
    fi ;;
  esac
  mkdir -p "$OG_WASM_OUT/opt"; : > "$OG_WASM_OUT/opt.txt"; : > "$OG_WASM_OUT/opt-all.txt"
  if [ ! -s "$built" ]; then
    og_say "no wasm artifact to optimize"
    return 0
  fi
  og_need wasm-opt || return 1
  # Each variant is bindgen'd and optimised in its own background job (the
  # two take the same ~14 s; serial they would add up).
  local pids=() rc=0
  while IFS= read -r raw; do
    name="$(basename "$raw" .wasm)"
    printf '%s\n' "$OG_WASM_OUT/opt/$name.wasm" >> "$OG_WASM_OUT/opt-all.txt"
    wasm_opt_one "$raw" "$name" > "$OG_WASM_OUT/opt/$name.log" 2>&1 &
    pids+=($!)
  done < "$built"
  for pid in "${pids[@]}"; do wait "$pid" || rc=1; done
  cat "$OG_WASM_OUT"/opt/*.log
  [ "$rc" -eq 0 ] || return 1
  if grep -q -- '-publish\.wasm$' "$OG_WASM_OUT/opt-all.txt"; then
    grep -- '-publish\.wasm$' "$OG_WASM_OUT/opt-all.txt" > "$OG_WASM_OUT/opt.txt"
    og_say "size-gate measures the publish variant(s): $(og_words < "$OG_WASM_OUT/opt.txt")"
  else
    cp "$OG_WASM_OUT/opt-all.txt" "$OG_WASM_OUT/opt.txt"
  fi
}

step_size_gate() {
  # --all measures afresh; otherwise a module measured before is not again.
  [ "$FORCE_ALL" = 1 ] && export OG_SIZE_GATE_FRESH=1
  case "$RAN" in *" wasm-opt "*) ;; *)
    if [ -f "$OG_EVIDENCE_DIR/size-gate.json" ] \
        && skip_unchanged size-gate "size-gate (the wasm it measured is unchanged)" "$(rule_wasm)"; then
      return $SKIP
    fi ;;
  esac
  "$OG_ROOT/scripts/size-gate.sh"
}

# `og api check` regenerates every IDL output (crates/og-api/api.ogt -> .d.ts,
# wiki, JSON schema, Rust glue) and fails when a committed copy differs. The
# command counts as available when `og --help` lists an `api` subcommand.
step_drift() {
  local required help
  required="$(og_budget drift_check)" || return 1
  if skip_unchanged drift "drift" "$(rule_drift)"; then return $SKIP; fi
  help="$(cargo run -q -p og-cli -- --help 2>&1)" || help=""
  if printf '%s\n' "$help" | grep -Eq '^[[:space:]]*api([[:space:]]|$)'; then
    cargo run -q -p og-cli -- api check
    return $?
  fi
  if [ "$required" = required ]; then
    og_err "og api check is not available, and scripts/budgets.txt says 'drift_check required'"
    return 1
  fi
  og_say "SKIPPED: og api check not available yet (\`og --help\` lists no 'api' command); becomes a failure once scripts/budgets.txt says 'drift_check required'"
  return $SKIP
}

step_determinism() {
  # Cross-target determinism: the same math digest and world hash on the host
  # and on wasm32 (scalar and simd128), tools/wasm-determinism/run.sh.
  og_need node || return 1
  "$OG_ROOT/tools/wasm-determinism/run.sh"
}

# bun_test_packages DIR...: `bun run test` in each package that has a test
# script, installing its dependencies first when node_modules is missing.
bun_test_packages() {
  local pkg ran=0
  for pkg in "$@"; do
    [ -f "$pkg/package.json" ] || continue
    og_need bun || return 1
    if ! (cd "$pkg" && bun -e 'process.exit(require("./package.json").scripts?.test ? 0 : 1)'); then
      og_say "$pkg: no test script"
      continue
    fi
    og_say "$pkg: bun run test"
    if [ ! -d "$pkg/node_modules" ] && (cd "$pkg" && bun -e 'const p=require("./package.json");process.exit(Object.keys({...p.dependencies,...p.devDependencies}).length?0:1)'); then
      if [ -f "$pkg/bun.lock" ] || [ -f "$pkg/bun.lockb" ]; then
        (cd "$pkg" && bun install --frozen-lockfile) || return 1
      else
        (cd "$pkg" && bun install) || return 1
      fi
    fi
    (cd "$pkg" && bun run test) || return 1
    ran=$((ran + 1))
  done
  [ "$ran" -gt 0 ] || og_say "no package under packages/ has a test script yet"
}

# Every package. OG_GATE=1 makes packages/engine's build.sh take the dev
# addon and `og` the test step built and the wasm wasm-opt produced
# instead of compiling release and wasm again.
step_bun_test() {
  if [ ! -s "$OG_WASM_OUT/opt-all.txt" ]; then
    og_err "no $OG_WASM_OUT/opt-all.txt: run wasm-build and wasm-opt before bun-test (or 'bun run test' in packages/engine standalone)"
    return 1
  fi
  OG_GATE=1 bun_test_packages packages/*/
}

# The packages that need no native build.
step_bun_quick() {
  # shellcheck disable=SC2086
  bun_test_packages $BUN_QUICK_PACKAGES
}

# The webgpu and webgl2 tiers of templates/fx-showcase and of og-tests' own
# gfx scenes (crates/og-tests-web) in headless Chrome (docs/ci.md §Browser
# reference tiers; `make web-reference`). In no lane: CI has no Chrome for
# Testing.
step_web_reference() {
  local a b c d e f
  if skip_unchanged web-reference "web-reference" "$(rule_web)"; then return $SKIP; fi
  og_need bun || return 1
  (cd packages/engine && bun tests/web/reference.ts); a=$?
  (cd packages/engine && bun tests/web/reference.ts --webgl); b=$?
  # The E5 stations, each a fresh run like its playtest.
  (cd packages/engine && bun tests/web/reference-e5.ts); c=$?
  (cd packages/engine && bun tests/web/reference-e5.ts --webgl); d=$?
  # og-tests' scenes against their tier's reference, else metal's.
  (cd packages/engine && bun tests/web/reference-gfx.ts); e=$?
  (cd packages/engine && bun tests/web/reference-gfx.ts --webgl); f=$?
  return $((a | b | c | d | e | f))
}

# --- Scheduling ---------------------------------------------------------------
# The steps run concurrently, each as soon as what it reads is there
# (docs/ci.md §Running the gate locally): native cargo builds (target/debug)
# and the wasm32 ones (target/wasm, target/wasm32-unknown-unknown) take
# different cargo locks, so the Rust and the wasm chains overlap, and cargo
# serialises what shares a directory. Each step writes its own log
# ($OG_EVIDENCE_DIR/gate/<step>.log), printed whole when it ends.

# step_needs STEP: the steps whose outputs STEP reads. A selected STEP starts
# after those of them that are selected; --only STEP adds them.
# test needs wasm-build for og-tests::wasm's proof (its og-render artifact).
# bun-test does not wait for test: build.sh builds the dev addon and og
# itself (cargo's lock on target/debug serialises it with the test build),
# and beside test it measured stable (tests/evidence/gate-speed/).
step_needs() {
  case "$1" in
    test) echo wasm-build ;;
    wasm-opt) echo wasm-build ;;
    size-gate) echo wasm-opt ;;
    bun-test) echo wasm-opt ;;
  esac
}

# Per-step state in G_<field>_<step> variables (bash 3.2 has no associative arrays).
gate_key() { printf '%s' "$1" | tr '-' '_'; }
gate_set() { printf -v "G_$1_$(gate_key "$2")" '%s' "$3"; }
gate_get() { local n; n="G_$1_$(gate_key "$2")"; printf '%s' "${!n:-}"; }

# kill_tree PID: PID and everything under it (stopped first, so it forks no more).
kill_tree() {
  local c
  kill -STOP "$1" 2>/dev/null
  for c in $(pgrep -P "$1" 2>/dev/null); do kill_tree "$c"; done
  kill -TERM "$1" 2>/dev/null
  kill -CONT "$1" 2>/dev/null
}

# gate_launch STEP: runs STEP in the background with its output in its log;
# the log's .rc file appears last, when the step has ended.
gate_launch() {
  local s="$1" fn
  fn="step_$(gate_key "$s")"
  rm -f "$GATE_LOGS/$s".*
  (
    trap - EXIT INT TERM
    SKIP_NOTE="" PASS_NOTE="" OG_GREEN=""
    t0="$(date +%s)"
    "$fn" < /dev/null > "$GATE_LOGS/$s.log" 2>&1; rc=$?
    printf '%s\n' "$(( $(date +%s) - t0 ))" > "$GATE_LOGS/$s.time"
    printf '%s' "$SKIP_NOTE" > "$GATE_LOGS/$s.skip"
    printf '%s' "$PASS_NOTE" > "$GATE_LOGS/$s.pass"
    printf '%s\n' "$rc" > "$GATE_LOGS/$s.rc"
    exit "$rc"
  ) &
  gate_set pid "$s" "$!"
  gate_set start "$s" "$(date +%s)"
  gate_set state "$s" running
  og_say "--> [$(gate_get idx "$s")/$total] $s started (+$(( $(date +%s) - started ))s; log ${GATE_LOGS#"$OG_ROOT"/}/$s.log)"
}

# gate_finish STEP RC: prints STEP's log and its PASS, SKIPPED or FAIL line.
gate_finish() {
  local s="$1" rc="$2" dt line note
  dt="$(cat "$GATE_LOGS/$s.time" 2>/dev/null || echo 0)"
  og_say ""
  og_say "==> [$(gate_get idx "$s")/$total] $s"
  cat "$GATE_LOGS/$s.log"
  if [ "$rc" -eq 0 ]; then
    note="$(cat "$GATE_LOGS/$s.pass" 2>/dev/null)"
    line="PASS $s (${dt}s)${note:+: $note}"
    RAN="$RAN$s "
    og_green_set "$s"
    gate_set state "$s" ok
  elif [ "$rc" -eq "$SKIP" ]; then
    note="$(cat "$GATE_LOGS/$s.skip" 2>/dev/null)"
    line="SKIPPED $s (${dt}s)${note:+: $note}"
    gate_set state "$s" ok
  else
    line="FAIL $s (exit $rc, ${dt}s)"
    RAN="$RAN$s "
    # A failed step leaves no earlier pass to be skipped on.
    og_green_clear "$s"
    gate_set state "$s" failed
    [ -n "$FAILED" ] || FAILED="$s"
  fi
  gate_set line "$s" "$line"
  og_say "$line"
}

# gate_ready STEP: every step STEP waits for that is selected has passed or been skipped.
gate_ready() {
  local d
  for d in $(step_needs "$1"); do
    case " $selected " in *" $d "*) [ "$(gate_get state "$d")" = ok ] || return 1 ;; esac
  done
  return 0
}

# gate_cancel [REASON]: stops every running step; with REASON, records it.
gate_cancel() {
  local s pid
  for s in $selected; do
    [ "$(gate_get state "$s")" = running ] || continue
    pid="$(gate_get pid "$s")"
    kill_tree "$pid"
    wait "$pid" 2>/dev/null
    gate_set state "$s" cancelled
    if [ -n "${1:-}" ]; then
      gate_set line "$s" "CANCELLED $s ($(( $(date +%s) - $(gate_get start "$s") ))s): $1; partial log ${GATE_LOGS#"$OG_ROOT"/}/$s.log"
    fi
  done
}

case "${1:-}" in
  --list) for s in $STEPS; do echo "$s"; done; exit 0 ;;
  -h|--help) sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
esac
args=()
ONLY=0
for a in "$@"; do
  case "$a" in
    --all) FORCE_ALL=1 ;;
    --only) ONLY=1 ;;
    *) args+=("$a") ;;
  esac
done
set -- ${args[@]+"${args[@]}"}
[ -z "${CI:-}" ] || FORCE_ALL=1

if [ "$ONLY" = 1 ]; then
  [ "$#" -gt 0 ] || { og_err "--only needs at least one step (scripts/check.sh --list)"; exit 2; }
  case "$1" in quick) og_err "--only takes steps, not a lane"; exit 2 ;; esac
fi
case "${1:-}" in
  "") selected="$STEPS_FULL" ;;
  quick) shift; selected="$STEPS_QUICK${*:+ $*}" ;;
  *) selected="$*" ;;
esac
for s in $selected; do
  case " $STEPS " in *" $s "*) ;; *) og_err "unknown step '$s' (scripts/check.sh --list)"; exit 2 ;; esac
done
if [ "$ONLY" = 1 ]; then
  # The named steps and, transitively, the steps whose outputs they read,
  # in lane order; each prerequisite stays path-aware.
  want=" $selected "
  grew=1
  while [ "$grew" = 1 ]; do
    grew=0
    for s in $want; do
      for d in $(step_needs "$s"); do
        case "$want" in *" $d "*) ;; *) want="$want$d "; grew=1 ;; esac
      done
    done
  done
  selected=""
  for s in $STEPS; do case "$want" in *" $s "*) selected="$selected${selected:+ }$s" ;; esac; done
fi
GATE_JOBS="${OG_GATE_JOBS:-0}"   # steps at once; 0: as many as their inputs allow
case "$GATE_JOBS" in ''|*[!0-9]*) og_err "OG_GATE_JOBS must be a number (0: no cap)"; exit 2 ;; esac

# One gate (or release build.sh) per target directory: a second one waits
# here instead of thrashing beside the first (scripts/lock.sh).
og_lock_acquire "$OG_TARGET_DIR" "scripts/check.sh ${selected}" || exit 1
# Running steps stop with the gate (Ctrl-C, a FAIL, a kill), then the lock goes.
trap 'gate_cancel; og_lock_release' EXIT
GATE_LOGS="$OG_EVIDENCE_DIR/gate"
mkdir -p "$GATE_LOGS"
summary="$OG_EVIDENCE_DIR/check-summary.txt"
: > "$summary"
total="$(printf '%s\n' "$selected" | wc -w | tr -d ' ')"
n=0
for s in $selected; do
  n=$((n + 1))
  gate_set idx "$s" "$n"
  gate_set state "$s" pending
done
started="$(date +%s)"
og_say "Origin Engine gate: $(git rev-parse --short HEAD 2>/dev/null || echo no-git) $(rustc --version 2>/dev/null) target=$OG_TARGET_DIR"
OG_GATE_TREE="$(og_worktree_tree)" || OG_GATE_TREE=""
if [ "$FORCE_ALL" = 1 ]; then
  if [ -n "${CI:-}" ]; then og_say "every step and test runs (CI)"; else og_say "every step and test runs (--all)"; fi
elif [ -z "$OG_GATE_TREE" ]; then
  og_say "no git tree for this working copy: every step and test runs"
else
  og_say "path-aware: working tree ${OG_GATE_TREE:0:12}; a step none of whose inputs changed since it last passed is skipped (--all runs everything)"
fi
if [ "$GATE_JOBS" = 0 ]; then
  og_say "steps run concurrently as their inputs allow; logs in ${GATE_LOGS#"$OG_ROOT"/}/ (OG_GATE_JOBS=1 runs one at a time)"
else
  og_say "at most $GATE_JOBS step(s) at once (OG_GATE_JOBS); logs in ${GATE_LOGS#"$OG_ROOT"/}/"
fi

FAILED=""
running=0
done_n=0
while :; do
  for s in $selected; do
    [ "$(gate_get state "$s")" = running ] && [ -f "$GATE_LOGS/$s.rc" ] || continue
    wait "$(gate_get pid "$s")"; rc=$?
    running=$((running - 1))
    done_n=$((done_n + 1))
    gate_finish "$s" "$rc"
  done
  [ -z "$FAILED" ] || break
  [ "$done_n" -lt "$total" ] || break
  for s in $selected; do
    [ "$GATE_JOBS" = 0 ] || [ "$running" -lt "$GATE_JOBS" ] || break
    if [ "$(gate_get state "$s")" = pending ] && gate_ready "$s"; then
      gate_launch "$s"
      running=$((running + 1))
    fi
  done
  if [ "$running" -eq 0 ]; then
    og_err "gate: no step can start (scripts/check.sh step_needs)"; FAILED="${FAILED:-scheduler}"; break
  fi
  sleep 0.2
done

write_summary() {
  local s line
  for s in $selected; do
    line="$(gate_get line "$s")"
    [ -z "$line" ] || printf '%s\n' "$line"
  done > "$summary"
}
og_say ""
if [ -n "$FAILED" ]; then
  gate_cancel "stopped when '$FAILED' failed"
  write_summary
  og_say "gate FAILED at '$FAILED'; summary in $summary"
  cat "$summary"
  exit 1
fi
write_summary
og_say "gate PASSED ($done_n/$total steps, $(( $(date +%s) - started ))s)"
cat "$summary"
