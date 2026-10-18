#!/usr/bin/env bash
# Origin Engine gate; CI runs exactly this (docs/ci.md).
#
#   scripts/check.sh            the full lane, every step in order, stopping at the first FAIL
#   scripts/check.sh quick      the quick lane: fmt clippy test-quick drift bun-quick
#   scripts/check.sh STEP...    only the named steps, in the order given
#   scripts/check.sh --all ...  any of the above with every step and test run and
#                               nothing reused from target/og-wasm-cache (CI always is)
#   scripts/check.sh --list     the step names
#
# Path-aware (docs/ci.md §Lanes): a step that passes records the working
# tree it passed on (<target>/og-gate/green/), and a later run skips what
# none of its inputs changed since: the GPU reference tests (og-tests,
# og-ui, og-render and the crates they build on, templates/ui-kit/assets),
# the wasm steps (crates/), drift (the IDL, og-idl, og and the generated
# files), web-reference (og-host-web's crates, packages/engine,
# templates/fx-showcase). A change to scripts/, Cargo.toml, Cargo.lock,
# rust-toolchain.toml, .cargo/ or .config/ reruns all of them. Each skip is
# printed with its reason. --all, or CI set in the environment, runs
# everything.
#
# Full lane: fmt clippy wasm-build test wasm-opt size-gate drift bun-test determinism.
# wasm-build comes before test: og-tests' wasm test takes its og-render
# build instead of compiling og-render for wasm32 a second time.
# Quick lane: fmt clippy test-quick drift bun-quick (no og-tests, no wasm,
# no engine package build: the loop while working and the PR signal).
# wasm-opt reads what wasm-build produced, size-gate reads what wasm-opt
# produced and bun-test's build.sh takes the gate's wasm (OG_GATE=1), so run
# them together. Every native step is the dev profile; release is only the
# wasm the size gate measures (AGENTS.md §Build and gate discipline).
# Holds the gate lock (scripts/lock.sh, <target>/og-gate.lock) for its whole
# run; a second gate on the same target directory waits for it.
# Honours CARGO_TARGET_DIR. Each step prints PASS, FAIL or SKIPPED; the
# summary lands in $OG_EVIDENCE_DIR/check-summary.txt.
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
  echo "$(og_crate_dirs -p og-host-web --target "$WASM_TRIPLE" -e normal,build | og_words) packages/engine templates/fx-showcase $RULE_COMMON"
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
# file never does.
wasm_build() {
  local variant="$1" features="$2"; shift 2
  local args=() c f name dest
  for c in "$@"; do args+=(-p "$c"); done
  [ -z "$features" ] || args+=(--features "$features")
  cargo build --release --target "$WASM_TRIPLE" "${args[@]}" --lib \
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
WASM_OPT_FLAGS="-Oz --enable-bulk-memory --enable-nontrapping-float-to-int --low-memory-unused"
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
  # --low-memory-unused: rustc links wasm32 stack-first (the stack is
  # [0, 1 MiB), data from 1 MiB), so the low 1 KiB is only ever the bottom of
  # a stack about to overflow; it lets wasm-opt fold constant offsets into
  # loads and stores (binaryen 133: -100 KB, -23.7 KB brotli on the publish
  # variant). --converge, a second -Oz and the strip flags gained nothing.
  # shellcheck disable=SC2086 # the flags are words
  wasm-opt $WASM_OPT_FLAGS "$input" -o "$OG_WASM_OUT/opt/$name.wasm" || return 1
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

# The webgpu and webgl2 tiers of templates/fx-showcase in headless Chrome
# (docs/ci.md §Browser reference tiers; `make web-reference`). In no lane:
# CI has no Chrome for Testing.
step_web_reference() {
  local a b
  if skip_unchanged web-reference "web-reference" "$(rule_web)"; then return $SKIP; fi
  og_need bun || return 1
  (cd packages/engine && bun tests/web/reference.ts); a=$?
  (cd packages/engine && bun tests/web/reference.ts --webgl); b=$?
  # The E5 stations, each a fresh run like its playtest.
  (cd packages/engine && bun tests/web/reference-e5.ts); c=$?
  (cd packages/engine && bun tests/web/reference-e5.ts --webgl); d=$?
  return $((a | b | c | d))
}

case "${1:-}" in
  --list) for s in $STEPS; do echo "$s"; done; exit 0 ;;
  -h|--help) sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
esac
args=()
for a in "$@"; do
  if [ "$a" = --all ]; then FORCE_ALL=1; else args+=("$a"); fi
done
set -- ${args[@]+"${args[@]}"}
[ -z "${CI:-}" ] || FORCE_ALL=1

case "${1:-}" in
  "") selected="$STEPS_FULL" ;;
  quick) shift; selected="$STEPS_QUICK${*:+ $*}" ;;
  *) selected="$*" ;;
esac
for s in $selected; do
  case " $STEPS " in *" $s "*) ;; *) og_err "unknown step '$s' (scripts/check.sh --list)"; exit 2 ;; esac
done

# One gate (or release build.sh) per target directory: a second one waits
# here instead of thrashing beside the first (scripts/lock.sh).
og_lock_acquire "$OG_TARGET_DIR" "scripts/check.sh ${selected}" || exit 1
mkdir -p "$OG_EVIDENCE_DIR"
summary="$OG_EVIDENCE_DIR/check-summary.txt"
: > "$summary"
total="$(printf '%s\n' "$selected" | wc -w | tr -d ' ')"
n=0
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
for s in $selected; do
  n=$((n + 1))
  og_say ""
  og_say "==> [$n/$total] $s"
  t0="$(date +%s)"
  fn="step_$(echo "$s" | tr '-' '_')"
  SKIP_NOTE=""
  PASS_NOTE=""
  OG_GREEN=""
  "$fn"; rc=$?
  dt=$(( $(date +%s) - t0 ))
  if [ "$rc" -eq 0 ]; then
    line="PASS $s (${dt}s)${PASS_NOTE:+: $PASS_NOTE}"
    RAN="$RAN$s "
    og_green_set "$s"
  elif [ "$rc" -eq "$SKIP" ]; then
    line="SKIPPED $s (${dt}s)${SKIP_NOTE:+: $SKIP_NOTE}"
  else
    line="FAIL $s (exit $rc, ${dt}s)"
    RAN="$RAN$s "
    # A failed step leaves no earlier pass to be skipped on.
    og_green_clear "$s"
  fi
  og_say "$line"; echo "$line" >> "$summary"
  if [ "$rc" -ne 0 ] && [ "$rc" -ne "$SKIP" ]; then
    og_say ""; og_say "gate FAILED at '$s'; summary in $summary"
    exit 1
  fi
done
og_say ""
og_say "gate PASSED ($n/$total steps, $(( $(date +%s) - started ))s)"
cat "$summary"
