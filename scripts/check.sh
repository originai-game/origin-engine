#!/usr/bin/env bash
# Origin Engine gate; CI runs exactly this (docs/ci.md).
#
#   scripts/check.sh            the path-aware full lane: during a round, after a change
#   scripts/check.sh quick      the quick lane: fmt clippy test-quick drift bun-quick
#   scripts/check.sh windows    the Windows lane: what studio-win adds to the Mac's
#                               functional lane at the same commit (wasm-import
#                               test-windows web-reference; OG_WASM_FROM names the
#                               Mac's wasm-export; docs/ci.md §Two machines)
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
# files), check-conformance (packages/engine, templates/, examples/, the
# comparer, its corpus and the pins), web-reference (og-host-web's and og-tests-web's crates,
# packages/engine, templates/fx-showcase), ui-styles (og, templates/ui-kit,
# templates/styles, og-ui's references). A change to scripts/, Cargo.toml,
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
# The functional lane: what a merge must not break (every test, the GPU
# reference frames, the generated API, the packages' tests on the native
# host), without the wasm build, its size, determinism and the browser
# tiers, which are a wave's end measurements (docs/ci.md §Lanes).
STEPS_FUNCTIONAL="fmt clippy test drift bun-functional ui-styles"
# The Windows lane: the Mac's wasm at the same inputs instead of a wasm
# build, the tests whose result a platform can change, and the browser
# tiers in Chrome on D3D12 (docs/ci.md §Two machines).
STEPS_WINDOWS="wasm-import test-windows web-reference"
STEPS="$STEPS_FULL test-quick bun-quick bun-functional ui-styles web-reference check-conformance wasm-export wasm-import test-windows"
FUNCTIONAL=0
WINDOWS=0
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
  echo "$(og_crate_dirs -p og-tests -p og-ui -p og-render -e normal,build,dev | og_words) templates/ui-kit/assets templates/styles $RULE_COMMON"
}
# third_party: the vendored crates the wasm links.
rule_wasm() { echo "crates third_party $RULE_COMMON"; }
# og api check: the IDL, the generator and what it reads, og's api command,
# and every directory it writes (og_idl::OUTPUT_DIRS).
rule_drift() {
  echo "crates/og-api $(og_crate_dirs -p og-idl -e normal,build | og_words) crates/og-cli \
packages/engine/types packages/engine/schema packages/engine/src/generated \
crates/og-host-web/src/generated crates/og-host-node/src/generated packages/knowledge/wiki $RULE_COMMON"
}
# check-conformance: what the two checkers read (the engine package, the
# templates, the examples), the comparer, and where the pins are stated.
rule_conformance() {
  echo "packages/engine templates examples crates/og-cli/src/check.rs scripts/tsrs-diff.sh scripts/tsrs-corpus scripts/install-tools.sh scripts/lib.sh"
}
# ui-styles: og and the addon it loads (og-cli, og-host-node, packages/engine), the kit and the
# built-in styles its sheets draw, and og-ui's references.
rule_ui_styles() {
  echo "$(og_crate_dirs -p og-cli -p og-host-node -e normal,build | og_words) packages/engine templates/ui-kit templates/styles crates/og-ui/references $RULE_COMMON"
}
rule_web() {
  echo "$(og_crate_dirs -p og-host-web -p og-tests-web --target "$WASM_TRIPLE" -e normal,build | og_words) packages/engine templates/fx-showcase templates/looks templates/ui-kit templates/styles templates/archetypes/third-person examples/brasswick examples/meridian-loop $RULE_COMMON"
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
  # og-tests' gfx harness appends a line here for each frame on the
  # vulkan-cpu tier's known-divergence list (crates/og-tests/src/gfx.rs,
  # docs/ci.md §The vulkan-cpu tier's known divergences); nextest shows no
  # output of a passing test, so the list is printed after the run.
  local known="$OG_EVIDENCE_DIR/gfx-known-divergences.txt"
  rm -f "$known"
  if command -v cargo-nextest >/dev/null 2>&1; then
    local args=(--workspace) filters=() expr=""
    for pk in ${exclude[@]+"${exclude[@]}"}; do args+=(--exclude "$pk"); done
    [ "$gpu" = 1 ] || filters+=("not ($GPU_FILTER)")
    # The functional lane builds no wasm: og-tests' check of the gate's
    # wasm32 build of og-render is the full lane's.
    [ "$FUNCTIONAL" = 1 ] && filters+=("not test(og_render_checks_for_wasm32)")
    for f in ${filters[@]+"${filters[@]}"}; do expr="${expr:+$expr & }$f"; done
    [ -z "$expr" ] || args+=(-E "$expr")
    cargo nextest run "${args[@]}"; rc=$?
    if [ "$rc" -eq 0 ] && [ "$doc" = 1 ]; then cargo test --workspace --doc; rc=$?; fi
  else
    # cargo test runs a binary's tests on threads of one process, and two
    # renderers made on two threads trip the Vulkan loader's driver-unload
    # race (docs/ci.md §CI, phase-1 row 81), so the GPU crates run their
    # tests one at a time, after the rest.
    local gpu_crates=(og-tests og-ui og-render)
    local args=(--workspace) gpu_args=()
    for pk in "${gpu_crates[@]}"; do
      args+=(--exclude "$pk")
      case " ${exclude[*]+${exclude[*]}} " in *" $pk "*) ;; *) gpu_args+=(-p "$pk") ;; esac
    done
    for pk in ${exclude[@]+"${exclude[@]}"}; do
      case " ${gpu_crates[*]} " in *" $pk "*) ;; *) args+=(--exclude "$pk") ;; esac
    done
    cargo test "${args[@]}"; rc=$?
    if [ "$rc" -eq 0 ] && [ "$gpu" = 1 ] && [ "${#gpu_args[@]}" -gt 0 ]; then
      cargo test "${gpu_args[@]}" -- --test-threads=1; rc=$?
    fi
  fi
  if [ -s "$known" ]; then
    echo "known vulkan-cpu frame divergences ($known):"
    sort "$known"
  fi
  [ "$rc" -eq 0 ] || return "$rc"
  if [ "$gpu" = 1 ]; then og_green_set "$key.gpu"; fi
  return 0
}

# Every crate's tests (gfx in og-tests) and the doctests.
step_test() { run_tests test 1; }

# Every crate's tests but og-tests (gfx, walks, perf-shaped suites).
step_test_quick() { run_tests test-quick 0 og-tests; }

# The tests whose result Windows can change, beside the Mac's functional
# lane at the same commit (docs/ci.md §Two machines): the GPU reference
# tests (DX12's frames; path-aware as in test), the cfg(windows) tests, and
# the tests of every crate changed since this step last passed here, with
# the crates that depend on it (nextest's rdeps). A change to the build's
# configuration (RULE_COMMON), to third_party, a first run, or --all runs
# every test. The wave's end and a release run the whole suite (test).
WINDOWS_TESTS='(package(og-cli) & test(/windows/))'
step_test_windows() {
  og_need cargo-nextest || return 1
  local green changed="" f c p all="" gpu=1 expr="$WINDOWS_TESTS" crates=""
  green="$(og_green_get test-windows)"
  if [ "$FORCE_ALL" = 1 ]; then all="--all"
  elif [ -z "$green" ] || [ -z "${OG_GATE_TREE:-}" ]; then all="no earlier pass here to compare with"
  elif ! changed="$(git -C "$OG_ROOT" diff --no-renames --name-only "$green" "$OG_GATE_TREE" 2>/dev/null)"; then
    all="its last pass's tree is unknown here"
  fi
  while IFS= read -r f; do
    [ -n "$f" ] && [ -z "$all" ] || continue
    for p in $RULE_COMMON third_party; do
      case "$f" in "$p" | "$p"/*) all="$f changed" ;; esac
    done
    case "$f" in crates/*/*)
      c="${f#crates/}"; c="${c%%/*}"
      # A removed crate has no tests left to select.
      if [ -f "crates/$c/Cargo.toml" ]; then
        case " $crates " in *" $c "*) ;; *) crates="$crates $c" ;; esac
      fi ;;
    esac
  done <<< "$changed"
  if [ -n "$all" ]; then
    og_say "test-windows: every test ($all)"
    og_green_clear test-windows.gpu
    cargo nextest run --workspace || return 1
    og_green_set test-windows.gpu
    return 0
  fi
  if skip_unchanged test-windows.gpu "the GPU reference tests ($GPU_FILTER)" "$(rule_gpu)"; then
    gpu=0
    SKIP_NOTE=""
  else
    og_green_clear test-windows.gpu
    expr="$expr | $GPU_FILTER"
  fi
  for c in $crates; do expr="$expr | rdeps(=$c)"; done
  og_say "test-windows: changed since ${green:0:12}:${crates:- no crate}; nextest -E '$expr'"
  PASS_NOTE="the cfg(windows) tests${crates:+, the crates changed and their dependents:$crates}$([ "$gpu" = 1 ] && echo ", the GPU reference tests")"
  cargo nextest run --workspace --no-tests=warn -E "$expr" || return 1
  [ "$gpu" = 0 ] || og_green_set test-windows.gpu
}

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
# A VARIANT lanes* (lanes, lanes2d) builds through scripts/wasm-lanes.sh
# (wasm atomics, std rebuilt), which keeps its own target dir
# (target/wasm-mt) for the same reason; the lanes variants share it as the
# others share the wasm32 one.
# A FEATURES starting with `-` builds without the default features
# (`-og-host-web/publish,og-host-web/physics3d`).
wasm_build() {
  local variant="$1" features="$2"; shift 2
  local args=() c f name dest cargo=(cargo build --target "$WASM_TRIPLE")
  for c in "$@"; do args+=(-p "$c"); done
  case "$features" in -*) args+=(--no-default-features); features="${features#-}" ;; esac
  [ -z "$features" ] || args+=(--features "$features")
  # Every variant but lanes builds into the one wasm32 target directory:
  # cargo keys each unit by its feature set, so the 3D and 2D variants
  # coexist there and share every dependency (one wgpu, one naga, one
  # og-render) instead of compiling them once per variant in a directory
  # of their own; only the final .wasm is copied out under the variant's
  # name right after each build. The lanes variants need their own flags
  # and a rebuilt std, so they keep scripts/wasm-lanes.sh's directory, where
  # the 3D and 2D lanes modules share std and every dependency the same way.
  case "$variant" in
    lanes*) cargo=("$OG_ROOT/scripts/wasm-lanes.sh") ;;
    *) ;;
  esac
  # The gate's profile (Cargo.toml [profile.wasm-gate]: the shipping wasm
  # profile with codegen units, minutes faster, modules slightly larger, so
  # the size gate holds an upper bound); OG_WASM_PROFILE=wasm measures the
  # shipping profile itself.
  "${cargo[@]}" --profile "${OG_WASM_PROFILE:-$WASM_PROFILE_DEFAULT}" "${args[@]}" --lib \
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
  printf '%s\n' "${OG_WASM_PROFILE:-$WASM_PROFILE_DEFAULT}" > "$OG_WASM_OUT/profile"
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
  local v vname vfeat
  for c in $(og_wasm_publish_crates); do
    v="$(og_wasm_variants "$c")"
    if [ -z "$v" ]; then
      og_say "publish variant: $c (--features $c/publish)"
      wasm_build publish "$c/publish" "$c" || return 1
      continue
    fi
    while read -r vname vfeat; do
      og_say "publish variant $vname: $c (--no-default-features --features $vfeat)"
      wasm_build "$vname" "-$(printf '%s' "$vfeat" | tr ',' '\n' | sed "s|^|$c/|" | paste -sd, -)" "$c" || return 1
    done <<EOF_VARIANTS
$v
EOF_VARIANTS
  done
  for c in $(og_wasm_lanes_crates); do
    v="$(og_wasm_lanes_variants "$c")"
    if [ -z "$v" ]; then
      og_say "lanes variant: $c (--features $c/publish, wasm atomics: scripts/wasm-lanes.sh)"
      wasm_build lanes "$c/publish" "$c" || return 1
      continue
    fi
    # One lanes module per game profile (scripts/wasm-crates.txt `lanes`).
    while read -r vname vfeat; do
      og_say "lanes variant $vname: $c (--no-default-features --features $vfeat, wasm atomics: scripts/wasm-lanes.sh)"
      wasm_build "$vname" "-$(printf '%s' "$vfeat" | tr ',' '\n' | sed "s|^|$c/|" | paste -sd, -)" "$c" || return 1
    done <<EOF_LANES
$v
EOF_LANES
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
# the shipping ones in opt.txt: the -publish* and -lanes* variants (one of
# each per game profile) when any were built, else all. A -lanes* module is
# bindgen'd as origin-engine-lanes and optimised with --threads. size-gate reads opt.txt; build.sh under OG_GATE=1 reads both.
# wasm_opt_one RAW NAME: wasm-bindgen (when the module uses it) then wasm-opt
# into $OG_WASM_OUT/opt/NAME.wasm.
# The flags are scripts/wasm-opt.sh's, the one invocation build.sh ships with.
wasm_opt_one() {
  local raw="$1" name="$2" input="$1" lock have key cache out=origin-engine threads=()
  case "$name" in *-lanes*) out=origin-engine-lanes; threads=(--threads) ;; esac
  # A raw module byte-identical to one already optimised (an edit that does
  # not reach the wasm: a test, a comment, native-only code) takes the
  # earlier result: same input, tools and flags, same output.
  key="$(og_sha256 "$raw") $(wasm-bindgen --version 2>/dev/null) $(wasm-opt --version 2>/dev/null) $("$OG_ROOT/scripts/wasm-opt.sh" --flags ${threads[@]+"${threads[@]}"}) $out"
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
    wasm-bindgen --target web --out-dir "$OG_WASM_OUT/bindgen/$name" --out-name "$out" "$raw" || return 1
    input="$OG_WASM_OUT/bindgen/$name/${out}_bg.wasm"
  fi
  # The flags and why: scripts/wasm-opt.sh.
  "$OG_ROOT/scripts/wasm-opt.sh" ${threads[@]+"${threads[@]}"} "$input" "$OG_WASM_OUT/opt/$name.wasm" || return 1
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
      wasm_stamp
      return $SKIP
    fi ;;
  esac
  rm -f "$OG_WASM_OUT/stamp"
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
    grep -E -- '-(publish|lanes)[a-z0-9]*\.wasm$' "$OG_WASM_OUT/opt-all.txt" > "$OG_WASM_OUT/opt.txt"
    og_say "size-gate measures the publish variant(s): $(og_words < "$OG_WASM_OUT/opt.txt")"
  else
    cp "$OG_WASM_OUT/opt-all.txt" "$OG_WASM_OUT/opt.txt"
  fi
  wasm_stamp
}

# --- The wasm between machines (docs/ci.md §Two machines) --------------------
# The wasm is the same bytes on every platform, so studio-win takes the
# Mac's instead of building it: wasm-export packs it, wasm-import takes it
# only when it was built from inputs equal to this checkout's.

# wasm_inputs: "input PATH TREE" for each of the wasm's inputs (rule_wasm),
# TREE its git object in this working tree (untracked files included), or
# "none". Equal lines on two machines: the same wasm.
wasm_inputs() {
  local p
  for p in $(rule_wasm); do
    printf 'input %s %s\n' "$p" "$(git -C "$OG_ROOT" rev-parse -q --verify "$OG_GATE_TREE:$p" 2>/dev/null || echo none)"
  done
}

# wasm_stamp: $OG_WASM_OUT/stamp, what the wasm there was built from: the
# commit, the profile and wasm_inputs. Without a git tree, none.
wasm_stamp() {
  [ -n "${OG_GATE_TREE:-}" ] || return 0
  {
    printf 'commit %s\n' "$(git -C "$OG_ROOT" rev-parse HEAD)"
    printf 'profile %s\n' "$(cat "$OG_WASM_OUT/profile" 2>/dev/null || echo unknown)"
    wasm_inputs
  } > "$OG_WASM_OUT/stamp"
}

# The web modules (wasm-opt's outputs and the bindgen glue build.sh copies)
# and their stamp, as $OG_TARGET_DIR/og-wasm.tar.gz for another machine.
step_wasm_export() {
  local tar="$OG_TARGET_DIR/og-wasm.tar.gz"
  if [ ! -f "$OG_WASM_OUT/stamp" ]; then
    og_err "no $OG_WASM_OUT/stamp: wasm-opt writes it (scripts/check.sh --only wasm-export runs wasm-build and wasm-opt first)"
    return 1
  fi
  # tar reads and writes stdio: GNU tar on Windows reads "E:/..." as a remote host.
  (cd "$OG_WASM_OUT" && tar -czf - stamp opt.txt opt-all.txt opt bindgen) > "$tar" || return 1
  og_say "$tar: $(og_bytes "$tar") B, built at $(awk '$1 == "commit" { print $2 }' "$OG_WASM_OUT/stamp") ($(awk '$1 == "profile" { print $2 }' "$OG_WASM_OUT/stamp") profile)"
  og_say "on the other machine, at the same commit: OG_WASM_FROM=<its copy> scripts/check.sh windows"
}

# $OG_WASM_FROM (wasm-export's archive, or a directory with its contents)
# into $OG_WASM_OUT, where build.sh takes the web modules from under
# OG_GATE=1. Refused unless its stamp's inputs equal this checkout's.
step_wasm_import() {
  local from="${OG_WASM_FROM:-}" tmp="$OG_TARGET_DIR/og-wasm.import" theirs mine differ f
  if [ -z "$from" ]; then
    og_err "wasm-import: OG_WASM_FROM names no archive: copy the Mac's target/og-wasm.tar.gz here (scripts/check.sh --only wasm-export at this commit) and name it"
    return 1
  fi
  if [ -z "${OG_GATE_TREE:-}" ]; then
    og_err "wasm-import: no git tree here, so what $from was built from cannot be checked"
    return 1
  fi
  rm -rf "$tmp"; mkdir -p "$tmp"
  if [ -d "$from" ]; then cp -R "$from"/. "$tmp"/ || return 1
  else tar -xzf - -C "$tmp" < "$from" || return 1
  fi
  if [ ! -f "$tmp/stamp" ]; then
    og_err "wasm-import: $from has no stamp: it is not wasm-export's"
    return 1
  fi
  theirs="$(grep '^input ' "$tmp/stamp")"
  mine="$(wasm_inputs)"
  if [ "$theirs" != "$mine" ]; then
    differ="$(printf '%s\n' "$mine" | grep -vxF "$theirs" | awk '{ print $2 }' | og_words)"
    og_err "wasm-import: $from was built at $(awk '$1 == "commit" { print $2 }' "$tmp/stamp"), whose wasm inputs differ from this checkout's (HEAD $(git -C "$OG_ROOT" rev-parse HEAD)) in: ${differ:-the list of inputs}"
    og_err "export the wasm again at this commit, or check out the commit it was built at"
    return 1
  fi
  rm -rf "$OG_WASM_OUT"
  mv "$tmp" "$OG_WASM_OUT" || return 1
  # The lists name the exporting machine's paths.
  for f in opt.txt opt-all.txt; do
    sed "s|.*/opt/|$OG_WASM_OUT/opt/|" "$OG_WASM_OUT/$f" > "$OG_WASM_OUT/$f.tmp" && mv "$OG_WASM_OUT/$f.tmp" "$OG_WASM_OUT/$f" || return 1
  done
  og_say "wasm-import: the wasm built at $(awk '$1 == "commit" { print $2 }' "$OG_WASM_OUT/stamp") ($(awk '$1 == "profile" { print $2 }' "$OG_WASM_OUT/stamp") profile), its inputs equal to this checkout's: $(og_words < "$OG_WASM_OUT/opt-all.txt" | sed "s|$OG_WASM_OUT/opt/||g")"
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

# tsrs (og check) against tsgo on the engine's configs, every template, the
# examples and scripts/tsrs-corpus (scripts/tsrs-diff.sh): a diagnostic tsgo
# reports and tsrs does not fails; every other difference is logged, not
# failed. In no lane: a wave's end runs it (scripts/wave-end.sh). Reads
# sources only, never a build output, so it waits for no step.
step_check_conformance() {
  if skip_unchanged check-conformance "check-conformance" "$(rule_conformance)"; then return $SKIP; fi
  scripts/tsrs-diff.sh
}

# The determinism tool and the workspace crates it links (og-core, og-ecs,
# og-physics, og-anim, og-lanes and what they build on).
rule_determinism() {
  echo "tools/wasm-determinism $(og_crate_dirs --manifest-path tools/wasm-determinism/Cargo.toml -e normal | og_words) $RULE_COMMON"
}
step_determinism() {
  # Cross-target determinism: the same math digest and world hash on the host
  # and on wasm32 (scalar and simd128), tools/wasm-determinism/run.sh. Four
  # builds and five runs (about a minute warm), so it runs only when one of
  # its inputs changed.
  if skip_unchanged determinism "determinism" "$(rule_determinism)"; then return $SKIP; fi
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
# The packages' tests on what is here: the dev addon and og (built by
# build.sh under OG_GATE=1) and the last full lane's wasm when one exists,
# else only the packages that need no native build. A merge's functional
# signal; the web module's own tests are the full lane's.
step_bun_functional() {
  if [ -s "$OG_WASM_OUT/opt-all.txt" ]; then
    # The web module is the last full lane's. When a crate changed since,
    # it no longer matches the native addon, and the tests that compare the
    # two hosts' hashes skip (OG_WASM_STALE; the full lane runs them).
    # shellcheck disable=SC2046 # the rule is a word list
    if og_unchanged wasm-build $(rule_wasm); then
      og_say "bun-functional: the web module ($OG_WASM_OUT/opt-all.txt) is current"
      OG_GATE=1 bun_test_packages packages/*/
    else
      og_say "bun-functional: the web module ($OG_WASM_OUT/opt-all.txt) predates a crate change; the cross-host tests skip (OG_WASM_STALE=1), the full lane runs them"
      OG_GATE=1 OG_WASM_STALE=1 bun_test_packages packages/*/
    fi
  else
    og_say "bun-functional: no wasm built here yet; the packages that need none"
    # shellcheck disable=SC2086
    bun_test_packages $BUN_QUICK_PACKAGES
  fi
}

step_bun_test() {
  if [ ! -s "$OG_WASM_OUT/opt-all.txt" ]; then
    og_err "no $OG_WASM_OUT/opt-all.txt: run wasm-build and wasm-opt before bun-test (or 'bun run test' in packages/engine standalone)"
    return 1
  fi
  OG_GATE=1 bun_test_packages packages/*/
}

# Every built-in style pack's reference sheets, desktop and phone
# (templates/ui-kit/tests/style-*.og.ts against
# crates/og-ui/references/<tier>/kit-style-*.png, else metal's): the native
# og-ui tests draw no style pack's screens, so without this step a style
# changed without its sheets re-recorded was first caught by a wave end's
# web-reference (2026-11-01, neon after bc65bed4). Without a renderer the
# playtests report their frames unchecked. Path-aware (rule_ui_styles).
step_ui_styles() {
  if skip_unchanged ui-styles "ui-styles" "$(rule_ui_styles)"; then return $SKIP; fi
  # og test loads packages/engine/native's addon (else the npm one, an
  # older engine): build the dev addon and og there first.
  OG_GATE=1 sh packages/engine/scripts/build.sh --native || return 1
  "$OG_TARGET_DIR/debug/og" test templates/ui-kit --filter tests/style-
}

# The packages that need no native build.
step_bun_quick() {
  # shellcheck disable=SC2086
  bun_test_packages $BUN_QUICK_PACKAGES
}

# The browser tiers in headless Chrome on WebGPU (docs/ci.md §Browser
# reference tiers; `make web-reference`): templates/fx-showcase, og-tests'
# and og-ui's gfx scenes (crates/og-tests-web), og-ui's kit frames
# (templates/ui-kit), Ember Vale, two example games, the lanes' panic and
# the accessibility mirror. In the windows lane and a wave's end, no other:
# CI has no Chrome for Testing.
#
# Seven groups, each with the projects only it cooks and exports (two
# runs of one project in parallel would cook and export into the same
# directories), run OG_WEB_JOBS at a time (WEB_JOBS_DEFAULT, measured per
# machine in docs/ci.md §Two machines; 1 is the old serial order), each
# group's output in $GATE_LOGS/web-reference/<group>.log, printed in order
# at the end; the step fails when any group does. Every run serves on its
# own port and starts Chrome with its own profile directory (port 0,
# mkdtemp: tests/web/serve.ts, cdp.ts).
WEB_GROUPS="fx ui gfx meridian-loop brasswick ember-vale lane-panic"
case "$(uname -s)" in
  MINGW*|MSYS*) WEB_JOBS_DEFAULT=3 ;;
  *) WEB_JOBS_DEFAULT=3 ;;
esac

# Each web_<group> runs in packages/engine; non-zero when any of its runs failed.
# templates/fx-showcase: its five stations; the E5 stations, each a fresh
# run like its playtest; the looks station, every preset of the catalogue
# in one run, each frame against its tier's reference, else metal's
# (tests/looks-script.ts).
web_fx() {
  local rc=0
  bun tests/web/reference.ts || rc=1
  bun tests/web/reference-e5.ts || rc=1
  bun tests/web/reference.ts --looks || rc=1
  return "$rc"
}
# templates/ui-kit: og-ui's kit frames, each reference playtest a fresh run
# like its own; then the accessibility mirror over its settings screen, a
# screen reader's click, the live region and the page's preferences.
web_ui() {
  local rc=0
  bun tests/web/reference-ui.ts || rc=1
  bun tests/web/a11y.ts || rc=1
  return "$rc"
}
# og-tests' scenes and og-ui's against their tier's reference, else metal's.
web_gfx() { bun tests/web/reference-gfx.ts; }
# Ember Vale's three frames: tests/frames.og.ts's commands without its
# screenshots and quit, each shot at its screenshot's tick (60 t).
web_ember_vale() {
  # shellcheck disable=SC2054 # the commas are inside one --shots word
  local tp=(--project templates/archetypes/third-person --out dist/test-reference/third-person
    --script "1.0:run;1.1:warp:0;1.3:key_down:KeyF;3.5:key_up:KeyF;3.6:light:1;3.7:light:2;3.8:light:3"
    --shots title@54,beacon@204,dawn@432)
  bun tests/web/reference.ts "${tp[@]}"
}
# An example game's frames: its tests/frames.og.ts itself (--playtest: its
# commands without its screenshots and quit, each shot at its screenshot's
# tick, 60 t). Its style packs' art and fonts and its music are not
# committed: a checkout without them makes them first (tools/make-assets.ts,
# as wave-end.sh's stage_prepare and each README say).
web_game() {
  if ! ls "$OG_ROOT/examples/$1"/assets/styles/*/atlas.png "$OG_ROOT/examples/$1"/assets/music/*.wav > /dev/null 2>&1; then
    og_say "web-reference: examples/$1 has no generated assets yet: bun tools/make-assets.ts"
    (cd "$OG_ROOT/examples/$1" && bun tools/make-assets.ts) || return 1
  fi
  bun tests/web/reference.ts --project "examples/$1" --playtest tests/frames.og.ts --out "dist/test-reference/$1"
}
web_brasswick() { web_game brasswick; }
web_meridian_loop() { web_game meridian-loop; }
# A panic inside a lane reaches the page as a crash within 5 s, not a hang
# (docs/web-export.md §Errors), and the same fault single-threaded.
web_lane_panic() { bun tests/web/lane-panic.ts; }

# web_launch GROUP DIR: runs web_<GROUP> in the background, its output in
# DIR/GROUP.log; DIR/GROUP.rc appears last, when it has ended.
web_launch() {
  local g="$1" dir="$2"
  (
    t0="$(date +%s)"
    (cd "$OG_ROOT/packages/engine" && "web_$(gate_key "$g")") < /dev/null > "$dir/$g.log" 2>&1; rc=$?
    printf '%s\n' "$(( $(date +%s) - t0 ))" > "$dir/$g.time"
    printf '%s\n' "$rc" > "$dir/$g.rc"
  ) &
}

step_web_reference() {
  local jobs="${OG_WEB_JOBS:-$WEB_JOBS_DEFAULT}" dir="$GATE_LOGS/web-reference" pending="$WEB_GROUPS"
  local g running=0 failed="" times="" t0
  case "$jobs" in ''|0|*[!0-9]*) og_err "OG_WEB_JOBS must be a number from 1"; return 1 ;; esac
  if skip_unchanged web-reference "web-reference" "$(rule_web)"; then return $SKIP; fi
  og_need bun || return 1
  # The tests cook each project with packages/engine/native's og and serve
  # dist/web: this checkout's dev og and addon and the gate's wasm, never
  # what a release build or an older gate left there (the measure lane's
  # 0.3.0 og refused fx-showcase's newer asset types, 2026-10-24).
  OG_GATE=1 sh packages/engine/scripts/build.sh || return 1
  rm -rf "$dir"; mkdir -p "$dir"
  og_say "web-reference: $(printf '%s' "$WEB_GROUPS" | og_words), $jobs at a time (OG_WEB_JOBS); logs in ${dir#"$OG_ROOT"/}/"
  t0="$(date +%s)"
  while [ -n "$pending" ] || [ "$running" -gt 0 ]; do
    while [ -n "$pending" ] && [ "$running" -lt "$jobs" ]; do
      g="${pending%% *}"
      pending="${pending#"$g"}"; pending="${pending# }"
      web_launch "$g" "$dir"
      running=$((running + 1))
      og_say "web-reference: $g started (+$(( $(date +%s) - t0 ))s)"
    done
    for g in $WEB_GROUPS; do
      [ -f "$dir/$g.rc" ] && [ ! -f "$dir/$g.done" ] || continue
      : > "$dir/$g.done"
      running=$((running - 1))
      if [ "$(cat "$dir/$g.rc")" = 0 ]; then
        og_say "web-reference: PASS $g ($(cat "$dir/$g.time")s)"
      else
        og_say "web-reference: FAIL $g (exit $(cat "$dir/$g.rc"), $(cat "$dir/$g.time")s)"
        failed="$failed $g"
      fi
      times="$times${times:+, }$g $(cat "$dir/$g.time")s"
    done
    sleep 1
  done
  wait
  for g in $WEB_GROUPS; do
    og_say ""
    og_say "---- web-reference $g (exit $(cat "$dir/$g.rc"), $(cat "$dir/$g.time")s)"
    cat "$dir/$g.log"
  done
  og_say ""
  og_say "web-reference: $(( $(date +%s) - t0 ))s at $jobs at a time: $times"
  PASS_NOTE="$jobs groups at a time: $times"
  if [ -n "$failed" ]; then
    og_err "web-reference: failed:$failed"
    return 1
  fi
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
    test) [ "$FUNCTIONAL" = 1 ] || echo wasm-build ;;
    wasm-opt) echo wasm-build ;;
    size-gate) echo wasm-opt ;;
    bun-test) echo wasm-opt ;;
    wasm-export) echo wasm-opt ;;
    # The Windows lane's browser tiers take the Mac's wasm (build.sh, OG_GATE=1).
    web-reference) [ "$WINDOWS" = 0 ] || echo wasm-import ;;
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
  -h|--help) sed -n '2,23p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
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
# The slow integration tests (packages/engine's og dev reloads and the web
# export, two minutes of og processes) run under --all and in CI; the
# path-aware lane skips them (OG_GATE_ALL, read by the tests).
[ "$FORCE_ALL" != 1 ] || export OG_GATE_ALL=1
# The wasm profile: the shipping `wasm` under --all and in CI (the size the
# gate records is the one that ships), `wasm-gate` (codegen units, a third
# faster, modules 5-6 % larger: an upper bound) in the path-aware lane.
# Each profile keeps its own target subdirectory, so alternating costs no
# rebuild. OG_WASM_PROFILE overrides.
if [ "$FORCE_ALL" = 1 ]; then WASM_PROFILE_DEFAULT=wasm; else WASM_PROFILE_DEFAULT=wasm-gate; fi

if [ "$ONLY" = 1 ]; then
  [ "$#" -gt 0 ] || { og_err "--only needs at least one step (scripts/check.sh --list)"; exit 2; }
  case "$1" in quick|functional|windows) og_err "--only takes steps, not a lane"; exit 2 ;; esac
fi
case "${1:-}" in
  "") selected="$STEPS_FULL" ;;
  quick) shift; selected="$STEPS_QUICK${*:+ $*}" ;;
  functional) shift; FUNCTIONAL=1; selected="$STEPS_FUNCTIONAL${*:+ $*}" ;;
  windows) shift; WINDOWS=1; selected="$STEPS_WINDOWS${*:+ $*}" ;;
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
# Two steps at once by default: the native chain (clippy, test, drift) beside
# the wasm chain (wasm-build, wasm-opt, size-gate, bun-test); 0 lets every
# step start as soon as its inputs allow, which on a shared machine only
# adds load (docs/ci.md §Running the gate locally).
GATE_JOBS="${OG_GATE_JOBS:-2}"   # steps at once; 0: as many as their inputs allow
case "$GATE_JOBS" in ''|*[!0-9]*) og_err "OG_GATE_JOBS must be a number (0: no cap)"; exit 2 ;; esac

# One gate per machine, then one per target directory: a second one waits
# here instead of thrashing beside the first (scripts/lock.sh). Four
# worktrees' gates at once took a 12-core machine to load 228 and every one
# of them past its timeouts. OG_GATE_MACHINE_LOCK=0 skips the machine lock
# (CI runs one gate per runner and needs no shared directory).
if [ "${OG_GATE_MACHINE_LOCK:-1}" != "0" ]; then
  og_lock_acquire "${OG_GATE_MACHINE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/og-gate}" "scripts/check.sh ${selected} ($PWD)" || exit 1
fi
og_lock_acquire "$OG_TARGET_DIR" "scripts/check.sh ${selected}" || exit 1
# Running steps stop with the gate (Ctrl-C, a FAIL, a kill), then the lock goes.
trap 'gate_cancel; og_lock_release' EXIT
GATE_LOGS="$OG_EVIDENCE_DIR/gate"
mkdir -p "$GATE_LOGS"
# The gate's builds and tests yield to the editor, the agents and the
# browser beside them (not in CI, where the runner is the gate's alone).
[ -n "${CI:-}" ] || renice -n 10 -p $$ >/dev/null 2>&1 || true
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
    if [ "$(gate_get state "$s")" != running ] || [ ! -f "$GATE_LOGS/$s.rc" ]; then continue; fi
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
