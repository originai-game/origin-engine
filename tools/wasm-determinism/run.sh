#!/usr/bin/env bash
# Cross-target determinism check (docs/design.md §2.4): builds this crate for
# the host and for wasm32-unknown-unknown, runs the wasm under node, and
# compares the og-core math digest, the bouncing-boxes world hash and the
# og-physics and og-anim world hashes (see src/lib.rs) across every build.
# Exits 1 if any build disagrees.
#
#   tools/wasm-determinism/run.sh [--report <file.md>]
#
#   --report  also writes the results as a Markdown evidence file, for
#             scripts/evidence.sh M1 cross-target-determinism <file.md> "..."
#
# Builds: host debug, host release, wasm32 release (scalar), wasm32 release
# with +simd128. The host release runs twice, lanes off (OG_LANES=off) and
# on (OG_LANES=4: the 1,000-body pen's rapier step on four threads,
# crates/og-lanes, while the beyond scenario's few bodies stay below
# og-physics's grain and step serially; the walk's few skeletons stay
# below og-anim's grain, the native lanes test
# packages/engine/tests/lanes.test.ts splits a crowd), and the lanes run
# must have run in parallel, so the comparison is never between two serial
# runs. Needs cargo, the wasm32-unknown-unknown target and node.
# Builds go to $CARGO_TARGET_DIR/wasm-determinism (default target/).
set -eu -o pipefail
here="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source-path=SCRIPTDIR source=../../scripts/lib.sh
. "$here/../../scripts/lib.sh"

report=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --report) [ "$#" -ge 2 ] || { og_err "--report needs a file"; exit 2; }; report="$2"; shift 2 ;;
    *) og_err "usage: tools/wasm-determinism/run.sh [--report <file.md>]"; exit 2 ;;
  esac
done
command -v node >/dev/null || { og_err "node is required to run the wasm build"; exit 1; }

out="$OG_TARGET_DIR/wasm-determinism"
manifest="$here/Cargo.toml"
host="$(rustc -vV | awk '$1 == "host:" { print $2 }')"
results=""

# record NAME OUTPUT: keeps one build's four numbers.
record() {
  local digest hash physics anim
  digest="$(printf '%s\n' "$2" | awk '$1 == "math_digest" { print $2 }')"
  hash="$(printf '%s\n' "$2" | awk '$1 == "bouncing_boxes_hash" { print $2 }')"
  physics="$(printf '%s\n' "$2" | awk '$1 == "physics_hash" { print $2 }')"
  anim="$(printf '%s\n' "$2" | awk '$1 == "anim_hash" { print $2 }')"
  [ -n "$digest" ] && [ -n "$hash" ] && [ -n "$physics" ] && [ -n "$anim" ] \
    || { og_err "$1 printed no numbers:"; og_err "$2"; exit 1; }
  og_say "$(printf '%-28s math_digest %-20s bouncing_boxes_hash %-20s physics_hash %-20s anim_hash %s' "$1" "$digest" "$hash" "$physics" "$anim")"
  results="$results$1|$digest|$hash|$physics|$anim
"
}

for profile in debug release; do
  flag=""; [ "$profile" = release ] && flag="--release"
  # shellcheck disable=SC2086 # $flag is empty or one word
  CARGO_TARGET_DIR="$out/host" cargo build --quiet --locked $flag --manifest-path "$manifest" --bin wasm-determinism
done
record "$host debug" "$("$out/host/debug/wasm-determinism")"
record "$host release, lanes off" "$(OG_LANES=off "$out/host/release/wasm-determinism")"
lanes="$(OG_LANES=4 "$out/host/release/wasm-determinism")"
calls="$(printf '%s\n' "$lanes" | awk '$1 == "lanes_parallel_calls" { print $2 }')"
threads="$(printf '%s\n' "$lanes" | awk '$1 == "lanes_threads" { print $2 }')"
if [ "${calls:-0}" -eq 0 ] || [ "${threads:-1}" -le 1 ]; then
  og_err "the lanes run did not run in parallel (threads ${threads:-?}, parallel calls ${calls:-?}): lanes on against off would compare two serial runs"
  exit 1
fi
record "$host release, lanes on ($threads threads, $calls parallel calls)" "$lanes"

for variant in scalar simd128; do
  rustflags=""; [ "$variant" = simd128 ] && rustflags="-C target-feature=+simd128"
  RUSTFLAGS="$rustflags" CARGO_TARGET_DIR="$out/wasm-$variant" \
    cargo build --quiet --locked --release --manifest-path "$manifest" --lib --target wasm32-unknown-unknown
  wasm="$out/wasm-$variant/wasm32-unknown-unknown/release/wasm_determinism.wasm"
  record "wasm32 $variant (node $(node --version))" "$(node "$here/load.mjs" "$wasm")"
done

distinct="$(printf '%s' "$results" | cut -d '|' -f 2-5 | sort -u | wc -l | tr -d ' ')"
digest="$(printf '%s' "$results" | head -n 1 | cut -d '|' -f 2)"
hash="$(printf '%s' "$results" | head -n 1 | cut -d '|' -f 3)"
physics="$(printf '%s' "$results" | head -n 1 | cut -d '|' -f 4)"
anim="$(printf '%s' "$results" | head -n 1 | cut -d '|' -f 5)"

if [ -n "$report" ]; then
  # shellcheck disable=SC2016 # literal backticks
  {
    printf '# M1 cross-target determinism\n\n'
    printf 'Produced by `tools/wasm-determinism/run.sh --report` at %s on %s.\n' \
      "$(git -C "$OG_ROOT" rev-parse --short HEAD 2>/dev/null || echo no-git)" "$host"
    printf 'glam is built with `scalar-math`; the math digest covers `og_core::math`\n'
    printf 'and glam Vec3A/Vec4/Quat/Mat4 arithmetic (1.4M results); the world is\n'
    printf '`crates/og-ecs/tests/support/bouncing_boxes.rs`, seed 42, 3600 ticks.\n'
    printf 'The host release runs with lanes off and on (crates/og-lanes: the\n'
    printf "1,000-body pen's rapier step on a four-thread pool, the beyond scenario\n"
    printf "below og-physics's grain; the walk's few skeletons stay below og-anim's\n"
    printf 'grain, so animation runs serially either way); the wasm builds are\n'
    printf 'single-threaded, with rapier built without its parallel feature.\n\n'
    printf '| build | math digest | bouncing boxes world_hash | physics world hashes | anim world hash and retarget |\n|---|---|---|---|---|\n'
    printf '%s' "$results" | awk -F '|' '{ printf "| %s | %s | %s | %s | %s |\n", $1, $2, $3, $4, $5 }'
    printf '\n'
    if [ "$distinct" = 1 ]; then
      printf 'All builds agree: math digest %s, world_hash %s, physics %s, anim %s.\n' "$digest" "$hash" "$physics" "$anim"
    else
      printf 'The builds DISAGREE.\n'
    fi
    if [ "$(uname -m)" != x86_64 ]; then
      printf '\nx86_64 (SSE2) not run here: run the script on an x86_64 host (CI) to cover it.\n'
    fi
  } > "$report"
  og_say "wrote $report"
fi

if [ "$distinct" != 1 ]; then
  og_err "FAIL: the builds disagree (see above)"
  exit 1
fi
og_say "ok: every build agrees (math_digest $digest, bouncing_boxes_hash $hash, physics_hash $physics, anim_hash $anim)"
