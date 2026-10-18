#!/usr/bin/env bash
# Origin Engine gate; CI runs exactly this (docs/ci.md).
#
#   scripts/check.sh            the full lane, every step in order, stopping at the first FAIL
#   scripts/check.sh quick      the quick lane: fmt clippy test-quick drift bun-quick
#   scripts/check.sh STEP...    only the named steps, in the order given
#   scripts/check.sh --list     the step names
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
STEPS="$STEPS_FULL test-quick bun-quick"
BUN_QUICK_PACKAGES="packages/knowledge packages/create"   # no native build needed
WASM_TRIPLE=wasm32-unknown-unknown
SKIP=3   # a step returns this to report SKIPPED instead of PASS

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

# cargo nextest runs the test binaries in parallel (.config/nextest.toml);
# without it, cargo test. Doctests are cargo test's (nextest runs none), so
# the full lane adds them.
step_test() {
  if command -v cargo-nextest >/dev/null 2>&1; then
    cargo nextest run --workspace && cargo test --workspace --doc
  else
    cargo test --workspace
  fi
}

# Every crate's tests but og-tests (gfx, walks, perf-shaped suites).
step_test_quick() {
  if command -v cargo-nextest >/dev/null 2>&1; then
    cargo nextest run --workspace --exclude og-tests
  else
    cargo test --workspace --exclude og-tests
  fi
}

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

step_wasm_build() {
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
wasm_opt_one() {
  local raw="$1" name="$2" input="$1" lock have
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
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --low-memory-unused \
    "$input" -o "$OG_WASM_OUT/opt/$name.wasm" || return 1
  og_say "$name: $(og_bytes "$raw") B raw -> $(og_bytes "$input") B bindgen -> $(og_bytes "$OG_WASM_OUT/opt/$name.wasm") B wasm-opt -Oz"
}

step_wasm_opt() {
  local built="$OG_WASM_OUT/built.txt" raw name input lock have
  if [ ! -f "$built" ]; then og_err "no $built: run the wasm-build step first"; return 1; fi
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

step_size_gate() { "$OG_ROOT/scripts/size-gate.sh"; }

# `og api check` regenerates every IDL output (crates/og-api/api.ogt -> .d.ts,
# wiki, JSON schema, Rust glue) and fails when a committed copy differs. The
# command counts as available when `og --help` lists an `api` subcommand.
step_drift() {
  local required help
  required="$(og_budget drift_check)" || return 1
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

case "${1:-}" in
  --list) for s in $STEPS; do echo "$s"; done; exit 0 ;;
  -h|--help) sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
esac

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
for s in $selected; do
  n=$((n + 1))
  og_say ""
  og_say "==> [$n/$total] $s"
  t0="$(date +%s)"
  fn="step_$(echo "$s" | tr '-' '_')"
  "$fn"; rc=$?
  dt=$(( $(date +%s) - t0 ))
  if [ "$rc" -eq 0 ]; then
    line="PASS $s (${dt}s)"
  elif [ "$rc" -eq "$SKIP" ]; then
    line="SKIPPED $s (${dt}s)"
  else
    line="FAIL $s (exit $rc, ${dt}s)"
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
