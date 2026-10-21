#!/usr/bin/env bash
# The wave's end, serially, by the one session that runs gates (docs/ci.md
# §When to run what): the whole gate and the browser tiers. The vulkan-cpu
# tier on lavapipe (the CPU-only sandbox requirement) was set aside on
# 2026-10-22 (docs/phase-1.md, "The CPU sandbox requirement set aside"):
# its stages stay here for when it returns and run only when --stages
# names them.
#
#   scripts/wave-end.sh [--dry-run] [--stages LIST] [--templates LIST] [--record NAMES]
#
# Stages, in order (--stages takes a comma list of them; default gate,web):
#   gate       scripts/check.sh --all
#   web        scripts/check.sh web-reference
#   cross      cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.31:
#              og-tests' og-gfx, og-ui's GPU test binaries and og-host-node,
#              the addon copied to packages/engine/native/origin-engine.linux-x64.node
#   prepare    each template cooked (`og cook`, the gate's dev og) and bundled
#              for the native host (<template>/.og/game.native.js), on this machine
#   gfx        og-gfx --all --capture target/lava-gfx in the lavapipe image
#   ui         og-ui's GPU tests (gfx, kit_gfx, materials_gfx, text_gfx) in the
#              image, against crates/og-ui/references/vulkan-cpu (else metal)
#   playtests  each template's playtests through playtest-run.ts in the image
#   table      every vulkan-cpu frame's outliers and RMSE against metal
#              (scripts/frames-vs-metal.ts), in target/evidence/wave-end/
#
# --templates  the projects whose playtests run (directory names under
#              templates/, templates/archetypes/ or examples/; default every
#              one with references/metal/ (a project whose lavapipe frames
#              all match metal has no vulkan-cpu/ directory and still runs),
#              and ui-kit, whose frames are og-ui's)
# --record     what to record on vulkan-cpu instead of compare, a comma list:
#              <template> (every playtest of it), <template>/<playtest>,
#              gfx (every og-tests scene), gfx/<scene>, or og-ui (its GPU
#              tests' frames). A playtest named runs with OG_FX_CAPTURE=1 and
#              OG_UI_CAPTURE=1; a gfx scene's capture is copied into
#              crates/og-tests/references/vulkan-cpu/; og-ui runs its tests
#              with OG_UI_CAPTURE=1.
#              Nothing is recorded without it.
# --dry-run    print every command and run none.
#
# The image is og-lavapipe-bun:local (amd64, Mesa lavapipe and bun; runs
# under emulation on Apple silicon, so no --platform flag). The checkout is
# mounted at its own path: the bundles hold absolute paths. Every stage runs
# even when one before it failed (a wave's end is a measurement); the exit
# status is non-zero when any failed. Holds the machine's and the target's
# gate locks for the whole run (scripts/lock.sh), so no other gate starts
# between its stages. Logs: target/evidence/wave-end/.
# shellcheck disable=SC2329 # the stage_* functions are called by name, "stage_$s"
set -u -o pipefail
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"
cd "$OG_ROOT" || exit 1

IMAGE="${OG_LAVAPIPE_IMAGE:-og-lavapipe-bun:local}"
TRIPLE=x86_64-unknown-linux-gnu
ALL_STAGES="gate web cross prepare gfx ui playtests table"
UI_TESTS="gfx kit_gfx materials_gfx text_gfx"
stages="gate web"
templates="ui-kit"
for d in templates/*/references/metal templates/archetypes/*/references/metal examples/*/references/metal; do
  [ -d "$d" ] && templates="$templates $(basename "$(dirname "$(dirname "$d")")")"
done
record=""
dry=0
usage() { sed -n '2,44p' "$0" | sed 's/^# \{0,1\}//'; }
while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run) dry=1 ;;
    --stages) stages="$(printf '%s' "${2:?--stages LIST}" | tr ',' ' ')"; shift ;;
    --templates) templates="$(printf '%s' "${2:?--templates LIST}" | tr ',' ' ')"; shift ;;
    --record) record="$(printf '%s' "${2:?--record NAMES}" | tr ',' ' ')"; shift ;;
    -h|--help) usage; exit 0 ;;
    *) og_err "wave-end: unknown argument '$1'"; usage >&2; exit 2 ;;
  esac
  shift
done
for s in $stages; do
  case " $ALL_STAGES " in *" $s "*) ;; *) og_err "wave-end: unknown stage '$s' (stages: $ALL_STAGES)"; exit 2 ;; esac
done

# template_dir NAME: the project directory of a template name.
template_dir() {
  if [ -f "templates/$1/project.ogt" ]; then echo "templates/$1"
  elif [ -f "templates/archetypes/$1/project.ogt" ]; then echo "templates/archetypes/$1"
  elif [ -f "examples/$1/project.ogt" ]; then echo "examples/$1"
  else return 1; fi
}
for t in $templates; do
  template_dir "$t" >/dev/null || { og_err "wave-end: no project '$t' (templates/<name>, templates/archetypes/<name> or examples/<name> with a project.ogt)"; exit 2; }
done

OUT="$OG_EVIDENCE_DIR/wave-end"
LAVA="$OG_TARGET_DIR/lava-gfx"
RELEASE="$OG_TARGET_DIR/$TRIPLE/release"
mkdir -p "$OUT"
LOG="$OUT/wave-end.log"
[ "$dry" = 1 ] || : > "$LOG"
say() { og_say "$*"; [ "$dry" = 1 ] || printf '%s\n' "$*" >> "$LOG"; }

# run CMD...: prints the command (quoted as a shell would read it), runs it
# unless --dry-run, its output into the log as well.
run() {
  say "+ $(printf '%q ' "$@")"
  [ "$dry" = 1 ] && return 0
  "$@" 2>&1 | tee -a "$LOG"
  return "${PIPESTATUS[0]}"
}

# The lavapipe container: the checkout at its own path, in its own directory.
docker_run() {
  run docker run --rm -v "$OG_ROOT:$OG_ROOT" -w "$OG_ROOT" "$@"
}

# recorded KIND NAME: true when --record names it (KIND gfx: a scene; else
# a template's playtest, NAME template/playtest).
recorded() {
  local r
  for r in $record; do
    if [ "$1" = gfx ]; then
      case "$r" in gfx|"gfx/$2") return 0 ;; esac
    else
      case "$r" in "${2%%/*}"|"$2") return 0 ;; esac
    fi
  done
  return 1
}

stage_gate() { run scripts/check.sh --all; }

stage_web() { run scripts/check.sh web-reference; }

stage_cross() {
  local t tests=()
  run cargo zigbuild --release --target "$TRIPLE.2.31" -p og-tests --bin og-gfx || return 1
  # og-ui's GPU test binaries; cargo's JSON names each executable.
  for t in $UI_TESTS; do tests+=(--test "$t"); done
  local ui=(cargo zigbuild --release --target "$TRIPLE.2.31" -p og-ui "${tests[@]}" --message-format=json-render-diagnostics)
  say "+ $(printf '%q ' "${ui[@]}")> $OUT/og-ui-tests.json"
  if [ "$dry" != 1 ]; then
    "${ui[@]}" > "$OUT/og-ui-tests.json" || return 1
    grep '"reason":"compiler-artifact"' "$OUT/og-ui-tests.json" | grep -o '"executable":"[^"]*"' | cut -d'"' -f4 > "$OUT/og-ui-tests.txt"
  fi
  run cargo zigbuild --release --target "$TRIPLE.2.31" -p og-host-node || return 1
  run cp "$RELEASE/libog_host_node.so" packages/engine/native/origin-engine.linux-x64.node
}

# The image runs bun on the bundles this machine makes: og's own prepare
# (`og run`/`og test`): cook when the project has assets, then bundle.
stage_prepare() {
  local t dir entry og="$OG_TARGET_DIR/debug/og" rc=0
  if [ "$dry" != 1 ] && [ ! -x "$og" ]; then og_err "no $og: the gate stage builds it (or cargo build -p og-cli)"; return 1; fi
  for t in $templates; do
    dir="$(template_dir "$t")"
    entry="$(awk '$1 == "entry" { gsub(/"/, "", $2); print $2; exit }' "$dir/project.ogt")"
    # A project that generates its assets (the examples: tools/make-assets.ts
    # writes what is gitignored, e.g. a music track) makes them first.
    if [ -f "$dir/tools/make-assets.ts" ]; then (cd "$dir" && run bun tools/make-assets.ts) || rc=1; fi
    if [ -d "$dir/assets" ]; then run "$og" cook "$dir" || rc=1; fi
    run bun packages/engine/src/node/build.ts --entry "$dir/${entry:-src/main.ts}" \
      --out "$OG_ROOT/$dir/.og/game.native.js" --target native || rc=1
  done
  return "$rc"
}

stage_gfx() {
  local f scene rc
  [ "$dry" = 1 ] || rm -rf "$LAVA"
  docker_run -e "OG_TEST_ASSETS=$OG_ROOT/crates/og-tests/assets" \
    --entrypoint "$RELEASE/og-gfx" "$IMAGE" --all --capture "$LAVA"; rc=$?
  [ "$rc" -eq 0 ] || return "$rc"
  # --record gfx[/<scene>]: the capture becomes the committed reference.
  for f in "$LAVA"/vulkan-cpu/*.png; do
    [ -f "$f" ] || continue
    scene="$(basename "$f" .png)"
    if recorded gfx "$scene"; then run cp "$f" "crates/og-tests/references/vulkan-cpu/$scene.png" || return 1; fi
  done
  case " $record " in *" gfx"*) [ "$dry" != 1 ] || say "(--record: each captured scene named is copied to crates/og-tests/references/vulkan-cpu/)" ;; esac
  return 0
}

# og-ui's GPU tests on lavapipe, every binary in one container; with
# --record og-ui, OG_UI_CAPTURE=1 records each frame on vulkan-cpu.
stage_ui() {
  local b script="$OUT/og-ui.sh" capture=""
  case " $record " in *" og-ui "*) capture="OG_UI_CAPTURE=1 " ;; esac
  {
    echo "#!/bin/sh"
    echo "# Written by scripts/wave-end.sh; runs in $IMAGE."
    echo "rc=0"
    if [ "$dry" = 1 ]; then
      echo "# one line per executable the cross stage lists in $OUT/og-ui-tests.txt, as:"
      echo "${capture}<og-ui test executable> --test-threads=2 || rc=1"
    else
      [ -s "$OUT/og-ui-tests.txt" ] || { og_err "no og-ui test binaries in $OUT/og-ui-tests.txt: run the cross stage"; return 1; }
      while IFS= read -r b; do
        [ -x "$b" ] || { og_err "$b is not there: run the cross stage"; return 1; }
        printf 'echo "== %s"\n%s%s --test-threads=2 || rc=1\n' "$(basename "$b")" "$capture" "$b"
      done < "$OUT/og-ui-tests.txt"
    fi
    # shellcheck disable=SC2016 # expands in the generated script
    echo 'exit "$rc"'
  } > "$script" || return 1
  [ "$dry" != 1 ] || sed 's/^/  | /' "$script"
  docker_run --entrypoint sh "$IMAGE" "$script"
}

# One container for every playtest: a POSIX sh script the image runs, one
# `PASS name`/`FAIL name` line per playtest in its output.
stage_playtests() {
  local t dir p name script="$OUT/playtests.sh" n=0
  {
    echo "#!/bin/sh"
    echo "# Written by scripts/wave-end.sh; runs in $IMAGE."
    echo "fail=0"
    for t in $templates; do
      dir="$(template_dir "$t")"
      for p in $(find "$dir/tests" -name '*.og.ts' -not -path '*/og-out/*' | sort); do
        name="$t/$(basename "$p" .og.ts)"
        n=$((n + 1))
        if recorded playtest "$name"; then
          printf 'echo "== %s (recording)"\n' "$name"
          printf 'if OG_FX_CAPTURE=1 OG_UI_CAPTURE=1 '
        else
          printf 'echo "== %s"\n' "$name"
          printf 'if '
        fi
        printf 'bun packages/engine/src/node/playtest-run.ts --file %s --entry %s --root %s; then echo "PASS %s"; else echo "FAIL %s"; fail=1; fi\n' \
          "$OG_ROOT/$p" "$OG_ROOT/$dir/.og/game.native.js" "$OG_ROOT/$dir" "$name" "$name"
      done
    done
    # shellcheck disable=SC2016 # expands in the generated script
    echo 'exit "$fail"'
  } > "$script"
  say "playtests: $n in $(printf '%s' "$templates" | og_words) ($script)"
  [ "$dry" != 1 ] || sed 's/^/  | /' "$script"
  docker_run --entrypoint sh "$IMAGE" "$script"
}

stage_table() {
  local t dir args=()
  args+=(--set og-tests-captured "$LAVA/vulkan-cpu" crates/og-tests/references/metal crates/og-tests/references/vulkan-cpu)
  args+=(--set og-ui crates/og-ui/references/vulkan-cpu crates/og-ui/references/metal)
  for dir in templates/*/references templates/archetypes/*/references examples/*/references; do
    [ -d "$dir/vulkan-cpu" ] || continue
    t="$(basename "$(dirname "$dir")")"
    args+=(--set "$t" "$dir/vulkan-cpu" "$dir/metal")
  done
  run bun scripts/frames-vs-metal.ts --out "$OUT/vulkan-cpu-vs-metal.tsv" "${args[@]}"
}

# The machine's and the target's gate locks for the whole run; the
# check.sh runs below find them held by their parent and go on.
if [ "$dry" != 1 ]; then
  og_lock_acquire "${OG_GATE_MACHINE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/og-gate}" "scripts/wave-end.sh ($PWD)" || exit 1
  og_lock_acquire "$OG_TARGET_DIR" "scripts/wave-end.sh" || exit 1
fi

note=""
[ -z "$record" ] || note=", recording: $(printf '%s' "$record" | og_words)"
[ "$dry" != 1 ] || note="$note (dry run: nothing runs)"
say "wave-end: $(git rev-parse --short HEAD 2>/dev/null || echo no-git), stages: $(printf '%s' "$stages" | og_words)$note"
summary=""
failed=0
t_all="$(date +%s)"
for s in $ALL_STAGES; do
  case " $(printf '%s' "$stages" | og_words) " in *" $s "*) ;; *) continue ;; esac
  say ""
  say "==> $s"
  t0="$(date +%s)"
  "stage_$s"; rc=$?
  dt=$(( $(date +%s) - t0 ))
  if [ "$dry" = 1 ]; then line="DRY $s"
  elif [ "$rc" -eq 0 ]; then line="PASS $s (${dt}s)"
  else line="FAIL $s (exit $rc, ${dt}s)"; failed=1; fi
  say "$line"
  summary="$summary$line"$'\n'
done
say ""
if [ "$dry" = 1 ]; then verdict="dry run"; elif [ "$failed" = 0 ]; then verdict=PASSED; else verdict=FAILED; fi
say "wave-end $verdict ($(( $(date +%s) - t_all ))s); logs in ${OUT#"$OG_ROOT"/}/"
printf '%s' "$summary" | while IFS= read -r l; do say "$l"; done
[ "$dry" = 1 ] || printf '%s' "$summary" > "$OUT/summary.txt"
exit "$failed"
