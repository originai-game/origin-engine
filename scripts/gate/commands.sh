# shellcheck shell=bash
# Build and validation commands, dispatched by step.sh.
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
  # sccache's shared store covers ordinary wasm and the lanes build-std.
  # Normalize this checkout's prefix; do not share Cargo's mutable target.
  export SCCACHE_BASEDIRS="$OG_ROOT"
  "${cargo[@]}" --profile "${OG_WASM_PROFILE:-$WASM_PROFILE_DEFAULT}" "${args[@]}" --lib \
      --message-format=json-render-diagnostics > "$OG_WASM_OUT/cargo${variant:+-$variant}.json" || return 1
  mkdir -p "$OG_WASM_OUT/raw"
  for f in $(grep '"reason":"compiler-artifact"' "$OG_WASM_OUT/cargo${variant:+-$variant}.json" \
      | grep -o '"[^"]*\.wasm"' | tr -d '"'); do
    name="$(basename "$f" .wasm)"
    dest="$OG_WASM_OUT/raw/$name${variant:+-$variant}.wasm"
    cp "$f" "$dest" || return 1
    printf '%s\n' "$dest" >> "$OG_WASM_OUT/built-${variant:-dev}.txt"
  done
  return 0
}

# outputs_exist LIST: LIST names at least one file and every one it names exists.
outputs_exist() {
  local f
  [ -s "$1" ] || return 1
  while IFS= read -r f; do [ -f "$f" ] || return 1; done < "$1"
}

# wasm_missing: nothing when wasm-opt's outputs are complete (opt-all.txt
# and opt.txt name at least one module and every module they name exists);
# else what is missing, for a step that reads them to fail on. wasm-opt
# writes the two lists last, so an interrupted build leaves them absent.
wasm_missing() {
  local f list
  for list in "$OG_WASM_OUT/opt-all.txt" "$OG_WASM_OUT/opt.txt"; do
    if [ ! -s "$list" ]; then
      printf '%s\n' "$list ($([ -f "$list" ] && echo empty || echo absent))"
      return 0
    fi
    while IFS= read -r f; do
      [ -f "$f" ] || { printf '%s\n' "$f (named in $list)"; return 0; }
    done < "$list"
  done
}

# require_wasm STEP: fails STEP, naming the missing file, unless wasm-opt's
# outputs are complete. Never a smaller run that passes: an interrupted wasm
# build once left bun-functional testing only knowledge and create, and it
# reported PASS (2026-11-02).
require_wasm() {
  local missing
  missing="$(wasm_missing)"
  [ -n "$missing" ] || return 0
  og_err "$1: the web module is incomplete: no $missing; scripts/check.sh --only wasm-opt builds the wasm (wasm-build, wasm-opt)"
  return 1
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
  (
  export CARGO_BUILD_JOBS=1
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
  ) > "$OG_WASM_OUT/normal.log" 2>&1 &
  local normal_pid=$!
  (
  export CARGO_BUILD_JOBS=1
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
  ) > "$OG_WASM_OUT/lanes.log" 2>&1 &
  local lanes_pid=$! failed=0
  wait "$normal_pid" || failed=1
  wait "$lanes_pid" || failed=1
  cat "$OG_WASM_OUT/normal.log" "$OG_WASM_OUT/lanes.log"
  [ "$failed" = 0 ] || return 1
  for c in $pending; do
    og_say "pending: $c"
    if wasm_build "" "" "$c"; then
      og_say "NOTE: $c now builds for $WASM_TRIPLE; mark it 'required' in scripts/wasm-crates.txt"
    else
      og_say "WARN: $c does not build for $WASM_TRIPLE yet (pending in scripts/wasm-crates.txt); tolerated"
    fi
  done
  cat "$OG_WASM_OUT"/built-*.txt | sort -u > "$OG_WASM_OUT/built.txt" || return 1
  if [ ! -s "$OG_WASM_OUT/built.txt" ]; then
    og_say "built as rlib only: no listed crate has crate-type cdylib yet, so no .wasm artifact"
  else
    sed 's/^/artifact: /' "$OG_WASM_OUT/built.txt"
  fi
}

# wasm-bindgen (when the module uses it, at the Cargo.lock version, out-name
# origin-engine as packages/engine ships it) then wasm-opt -Oz for shipping
# and --all builds; development keeps bindgen's module. Writes
# target/og-wasm/opt/<name>.wasm for every variant (opt-all.txt) and lists
# the shipping ones in opt.txt: the -publish* and -lanes* variants (one of
# each per game profile) when any were built, else all. A -lanes* module is
# bindgen'd as origin-engine-lanes and optimised with --threads. size-gate reads opt.txt; build.sh under OG_GATE=1 reads both.
# wasm_opt_one RAW NAME: process into $OG_WASM_OUT/opt/NAME.wasm.
# The flags are scripts/wasm-opt.sh's, the one invocation build.sh ships with.
wasm_opt_one() {
  local raw="$1" name="$2" input="$1" lock have key cache out=origin-engine threads=() optimise=1
  case "$name" in *-lanes*) out=origin-engine-lanes; threads=(--threads) ;; esac
  # Development loads bindgen's module, not the unused optimised copy.
  # Keep --all (wave-end) and the size gate's fallback input optimised.
  if [ "$name" = og_host_web ] \
      && grep -q '/og_host_web-publish.wasm$' "$OG_WASM_OUT/built.txt"; then
    optimise=0
  fi
  # A raw module byte-identical to one already optimised (an edit that does
  # not reach the wasm: a test, a comment, native-only code) takes the
  # earlier result: same input, tools and flags, same output.
  key="$(og_sha256 "$raw") $(wasm-bindgen --version 2>/dev/null) $(wasm-opt --version 2>/dev/null) $("$OG_ROOT/scripts/wasm-opt.sh" --flags ${threads[@]+"${threads[@]}"}) $out $optimise"
  cache="$OG_CONTENT_CACHE/opt/$name"
  if [ "$FORCE_ALL" != 1 ] && [ -f "$cache/key" ] && [ "$(cat "$cache/key")" = "$key" ] && [ -f "$cache/opt.wasm" ]; then
    rm -rf "$OG_WASM_OUT/bindgen/$name"
    if [ -d "$cache/bindgen" ]; then mkdir -p "$OG_WASM_OUT/bindgen" && cp -R "$cache/bindgen" "$OG_WASM_OUT/bindgen/$name"; fi
    cp "$cache/opt.wasm" "$OG_WASM_OUT/opt/$name.wasm" || return 1
    og_say "$name: $(og_bytes "$raw") B raw, byte-identical to the last processed module: reused $(og_bytes "$OG_WASM_OUT/opt/$name.wasm") B ($OG_CONTENT_CACHE/opt)"
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
  if [ "$optimise" = 1 ]; then
    "$OG_ROOT/scripts/wasm-opt.sh" ${threads[@]+"${threads[@]}"} "$input" "$OG_WASM_OUT/opt/$name.wasm" || return 1
    og_say "$name: $(og_bytes "$raw") B raw -> $(og_bytes "$input") B bindgen -> $(og_bytes "$OG_WASM_OUT/opt/$name.wasm") B wasm-opt -Oz"
  else
    cp "$input" "$OG_WASM_OUT/opt/$name.wasm" || return 1
    og_say "$name: $(og_bytes "$input") B bindgen (development: no wasm-opt)"
  fi
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
  # The lists are written last (from .part files), so a run interrupted
  # anywhere leaves none: the steps that read them fail (require_wasm).
  rm -f "$OG_WASM_OUT/stamp" "$OG_WASM_OUT/opt.txt" "$OG_WASM_OUT/opt-all.txt"
  mkdir -p "$OG_WASM_OUT/opt"; : > "$OG_WASM_OUT/opt-all.txt.part"
  if [ ! -s "$built" ]; then
    og_say "no wasm artifact to optimize"
    : > "$OG_WASM_OUT/opt.txt"; mv -f "$OG_WASM_OUT/opt-all.txt.part" "$OG_WASM_OUT/opt-all.txt"
    return 0
  fi
  og_need wasm-opt || return 1
  # Bound Binaryen's workers too; only shipped modules are optimized.
  export BINARYEN_CORES="$OG_GATE_THREADS"
  while IFS= read -r raw; do
    name="$(basename "$raw" .wasm)"
    printf '%s\n' "$OG_WASM_OUT/opt/$name.wasm" >> "$OG_WASM_OUT/opt-all.txt.part"
    wasm_opt_one "$raw" "$name" || return 1
  done < "$built"
  if grep -q -- '-publish\.wasm$' "$OG_WASM_OUT/opt-all.txt.part"; then
    grep -E -- '-(publish|lanes)[a-z0-9]*\.wasm$' "$OG_WASM_OUT/opt-all.txt.part" > "$OG_WASM_OUT/opt.txt.part"
    og_say "size-gate measures the publish variant(s): $(og_words < "$OG_WASM_OUT/opt.txt.part")"
  else
    cp "$OG_WASM_OUT/opt-all.txt.part" "$OG_WASM_OUT/opt.txt.part"
  fi
  mv -f "$OG_WASM_OUT/opt.txt.part" "$OG_WASM_OUT/opt.txt"
  mv -f "$OG_WASM_OUT/opt-all.txt.part" "$OG_WASM_OUT/opt-all.txt"
  wasm_stamp
}

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

step_size_gate() {
  # --all measures afresh; otherwise a module measured before is not again.
  [ "$FORCE_ALL" = 1 ] && export OG_SIZE_GATE_FRESH=1
  case "$RAN" in *" wasm-opt "*) ;; *)
    if [ -f "$OG_EVIDENCE_DIR/size-gate.json" ] \
        && skip_unchanged size-gate "size-gate (the wasm it measured is unchanged)" "$(rule_wasm)"; then
      return $SKIP
    fi ;;
  esac
  require_wasm size-gate || return 1
  "$OG_ROOT/scripts/size-gate.sh"
}

# `og api check` regenerates every IDL output (crates/og-api/api/<module>.ogt -> .d.ts,
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
# test fixtures and scripts/tsrs-corpus (scripts/tsrs-diff.sh): a diagnostic tsgo
# reports and tsrs does not fails; every other difference is logged, not
# failed. In no lane: a wave's end runs it (scripts/wave-end.sh). Reads
# sources only, never a build output, so it waits for no step.
step_check_conformance() {
  if skip_unchanged check-conformance "check-conformance" "$(rule_conformance)"; then return $SKIP; fi
  scripts/tsrs-diff.sh
}

step_bun_test() {
  require_wasm bun-test || return 1
  OG_GATE=1 sh packages/engine/scripts/build.sh || return 1
  (cd packages/engine && bun run typecheck) || return 1
  bun scripts/gate/bun.ts full
}

# The test fixtures (tests/fixtures/*/, one per feature, each with its
# README and a budget of 10 s warm): `og test` in each one whose inputs
# changed since it last passed (rule_fixture; --all runs every one), each
# holding one GPU slot (scripts/lock.sh) for its run. A fixture that left a
# frame unchecked (no renderer) is reported SKIPPED and keeps no green
# mark, so the next gate with a GPU runs it.
step_fixtures() {
  local dir name log rc project ran="" unchanged="" unchecked="" bad="" t0 times=""
  OG_GATE=1 sh packages/engine/scripts/build.sh --native || return 1
  for dir in packages/create/template tests/fixtures/*/; do
    dir="${dir%/}"
    [ -f "$dir/project.ogt" ] && [ -d "$dir/tests" ] || continue
    name="${dir##*/}"
    if skip_unchanged "fixture.$name" "fixture $name" "$(rule_fixture "$dir")"; then
      unchanged="$unchanged $name"
      continue
    fi
    og_green_clear "fixture.$name"
    log="$GATE_LOGS/fixtures-$name.txt"
    t0="$(date +%s)"
    (
      project="$dir"
      if [ "$dir" = packages/create/template ]; then
        project="$GATE_LOGS/creation-skeleton"
        bun packages/create/bin/create.ts "$project" --no-prompt --force || exit 1
      fi
      "$OG_TARGET_DIR/debug/og" test "$project"; rc=$?
      exit "$rc"
    ) 2>&1 | tee "$log"
    rc="${PIPESTATUS[0]}"
    times="$times $name=$(( $(date +%s) - t0 ))s"
    if [ "$rc" -ne 0 ]; then
      bad="$bad $name"
    elif grep -Eq '[1-9][0-9]* GPU frame/expectation checks unchecked' "$log"; then
      unchecked="$unchecked $name"
    else
      ran="$ran $name"
      og_green_set "fixture.$name"
    fi
  done
  SKIP_NOTE=""
  og_say "fixtures:${times:- none run}"
  if [ -n "$bad" ]; then
    og_err "fixtures: failed:$bad"
    return 1
  fi
  PASS_NOTE="ran${ran:- none}${unchanged:+; unchanged$unchanged}${unchecked:+; unchecked (no renderer)$unchecked}${times:+;$times}"
  if [ -n "$unchecked" ]; then
    SKIP_NOTE="GPU checks unchecked:$unchecked (no passing step snapshot)"
    return $SKIP
  fi
  if [ -z "$ran" ]; then
    if [ -n "$unchecked" ]; then
      SKIP_NOTE="no renderer:$unchecked left their frames unchecked, so nothing was compared (no green mark)"
    else
      SKIP_NOTE="no fixture's inputs changed (${unchanged# }; --all runs them)"
    fi
    return $SKIP
  fi
}

# The browser tiers in headless Chrome on WebGPU (docs/ci.md §Browser
# reference tiers; `make web-reference`): the former test sample, og-tests'
# and og-ui's gfx scenes (crates/og-tests-web), Ember Vale, the
# vehicle-terrain fixture, the lanes' panic and the accessibility mirror. Run at a wave's end on any host with Chrome.
#
# Six groups, each with the projects only it cooks and exports (two
# runs of one project in parallel would cook and export into the same
# directories), run OG_WEB_JOBS at a time (default 3, or 1 for serial), each
# group's output in $GATE_LOGS/web-reference/<group>.log, printed in order
# at the end; the step fails when any group does. Every run serves on its
# own port and starts Chrome with its own profile directory (port 0,
# mkdtemp: tests/web/serve.ts, cdp.ts).
WEB_GROUPS="ui gfx vehicle-terrain lane-panic"
WEB_JOBS_DEFAULT=3

# Each web_<group> runs in packages/engine; non-zero when any of its runs failed.
# The same two fx-showcase representatives as its native smoke playtest.
# The accessibility mirror over its settings screen, a
# screen reader's click, the live region and the page's preferences.
web_ui() {
  local rc=0
  bun tests/web/a11y.ts || rc=1
  return "$rc"
}
# og-tests' scenes and og-ui's against their tier's reference, else metal's.
web_gfx() { bun tests/web/reference-gfx.ts; }
# Ember Vale uses its native smoke's two frames and command timeline.
# The vehicle-terrain fixture's slide frame: its tests/drive.og.ts itself
# (--playtest: its commands without its screenshots and quit, each shot at
# its screenshot's tick, 60 t). Its assets are all generated by the cook.
web_vehicle_terrain() {
  bun tests/web/reference.ts --project tests/fixtures/vehicle-terrain --playtest tests/drive.og.ts --out dist/test-reference/vehicle-terrain
}
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
