#!/usr/bin/env bash
# Origin Engine local gate; CI runs exactly this (docs/ci.md).
#
#   scripts/check.sh            every step, in order, stopping at the first FAIL
#   scripts/check.sh STEP...    only the named steps, in the order given
#   scripts/check.sh --list     the step names
#
# Steps: fmt clippy test wasm-build wasm-opt size-gate drift bun-test.
# wasm-opt reads what wasm-build produced and size-gate reads what wasm-opt
# produced, so run them together. Honours CARGO_TARGET_DIR. Each step prints
# PASS, FAIL or SKIPPED; the summary lands in $OG_EVIDENCE_DIR/check-summary.txt.
set -u -o pipefail
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"
cd "$OG_ROOT" || exit 1

STEPS="fmt clippy test wasm-build wasm-opt size-gate drift bun-test determinism"
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

step_test() { cargo test --workspace; }

# Builds scripts/wasm-crates.txt and records each produced .wasm in
# target/og-wasm/built.txt. Artifacts are taken from cargo's JSON messages, so
# a fresh (not relinked) cdylib still counts and a stale file never does.
wasm_build() {  # wasm_build CRATE... -> appends .wasm paths to built.txt
  local args=() c
  for c in "$@"; do args+=(-p "$c"); done
  cargo build --release --target "$WASM_TRIPLE" "${args[@]}" --lib \
      --message-format=json-render-diagnostics > "$OG_WASM_OUT/cargo.json" || return 1
  grep '"reason":"compiler-artifact"' "$OG_WASM_OUT/cargo.json" \
    | grep -o '"[^"]*\.wasm"' | tr -d '"' >> "$OG_WASM_OUT/built.txt"
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
    wasm_build $required || return 1
  fi
  for c in $pending; do
    og_say "pending: $c"
    if wasm_build "$c"; then
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

# wasm-bindgen (when the module uses it, at the Cargo.lock version) then
# wasm-opt -Oz; writes target/og-wasm/opt/<name>.wasm and lists them in opt.txt.
step_wasm_opt() {
  local built="$OG_WASM_OUT/built.txt" raw name input lock have
  if [ ! -f "$built" ]; then og_err "no $built: run the wasm-build step first"; return 1; fi
  mkdir -p "$OG_WASM_OUT/opt"; : > "$OG_WASM_OUT/opt.txt"
  if [ ! -s "$built" ]; then
    og_say "no wasm artifact to optimize"
    return 0
  fi
  og_need wasm-opt || return 1
  while IFS= read -r raw; do
    name="$(basename "$raw" .wasm)"
    input="$raw"
    if grep -q '__wbindgen_placeholder__' "$raw"; then
      og_need wasm-bindgen || return 1
      lock="$(og_lock_version wasm-bindgen)"
      have="$(wasm-bindgen --version | awk '{print $2}')"
      if [ "$lock" != "$have" ]; then
        og_err "wasm-bindgen $have does not match Cargo.lock $lock (run scripts/install-tools.sh)"
        return 1
      fi
      wasm-bindgen --target web --out-dir "$OG_WASM_OUT/bindgen/$name" "$raw" || return 1
      input="$OG_WASM_OUT/bindgen/$name/${name}_bg.wasm"
    fi
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int \
      "$input" -o "$OG_WASM_OUT/opt/$name.wasm" || return 1
    og_say "$name: $(og_bytes "$raw") B raw -> $(og_bytes "$input") B bindgen -> $(og_bytes "$OG_WASM_OUT/opt/$name.wasm") B wasm-opt -Oz"
    printf '%s\n' "$OG_WASM_OUT/opt/$name.wasm" >> "$OG_WASM_OUT/opt.txt"
  done < "$built"
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

step_bun_test() {
  local pkg ran=0
  for pkg in packages/*/; do
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

case "${1:-}" in
  --list) for s in $STEPS; do echo "$s"; done; exit 0 ;;
  -h|--help) sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
esac

selected="${*:-$STEPS}"
for s in $selected; do
  case " $STEPS " in *" $s "*) ;; *) og_err "unknown step '$s' (scripts/check.sh --list)"; exit 2 ;; esac
done

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
