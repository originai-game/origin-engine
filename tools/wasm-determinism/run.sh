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
# with +simd128, and the lanes wasm once per physics profile (the 3D and the
# 2D lanes module's configuration: scripts/wasm-lanes.sh's atomics, shared
# memory and rebuilt std, og-physics with `physics3d` or `physics2d` alone,
# so rapier's parallel build; run under node on one lane, since no Worker
# pool is started: a 2D world on the 2D lanes build, a 3D one on the 3D).
# Each one-profile build has only its profile's physics hash, compared
# with every other build's. The host release runs twice, lanes off (OG_LANES=off) and
# on (OG_LANES=4: the 1,000-body pen's rapier step on four threads,
# crates/og-lanes, while the beyond scenario's few bodies stay below
# og-physics's grain and step serially; the walk's few skeletons stay
# below og-anim's grain, the native lanes test
# packages/engine/tests/lanes.test.ts splits a crowd), and the lanes run
# must have run in parallel, so the comparison is never between two serial
# runs. Needs cargo, the wasm32-unknown-unknown target, rust-src (the lanes
# builds rebuild std), wasm-bindgen and node.
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

# The numbers every build prints (src/lib.rs); a one-profile build leaves
# out the other profile's physics hash.
KEYS="math_digest bouncing_boxes_hash physics3d_hash physics2d_hash anim_hash"
# record NAME OUTPUT [SKIP_KEY]: keeps one build's numbers, "-" for SKIP_KEY
# (the physics hash of the profile the build does not have).
record() {
  local key value row="$1" line="$1"
  for key in $KEYS; do
    value="$(printf '%s\n' "$2" | awk -v k="$key" '$1 == k { print $2 }')"
    if [ "$key" = "${3:-}" ]; then
      [ -z "$value" ] || { og_err "$1 printed $key, which its profile does not have:"; og_err "$2"; exit 1; }
      value=-
    fi
    [ -n "$value" ] || { og_err "$1 printed no $key:"; og_err "$2"; exit 1; }
    row="$row|$value"
    line="$line $key $value"
  done
  og_say "$line"
  results="$results$row
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

# The lanes wasm per profile: scripts/wasm-lanes.sh, the one place its flags
# are, into $out/lanes/wasm-mt (both profiles share std and every
# dependency but og-physics and this crate), bound by wasm-bindgen as the
# lanes modules are (og-lanes' rayon on wasm reads the JS global through
# wasm-bindgen), then run under node through that glue (load.mjs).
og_need wasm-bindgen || exit 1
for p in 3d 2d; do
  CARGO_TARGET_DIR="$out/lanes" "$OG_ROOT/scripts/wasm-lanes.sh" --quiet --locked --release \
    --manifest-path "$manifest" --lib --no-default-features --features "physics$p"
  raw="$(CARGO_TARGET_DIR="$out/lanes" "$OG_ROOT/scripts/wasm-lanes.sh" --target-dir)/wasm32-unknown-unknown/release/wasm_determinism.wasm"
  rm -rf "$out/lanes-$p"
  wasm-bindgen --target web --out-dir "$out/lanes-$p" --out-name wasm_determinism "$raw"
  other=physics2d_hash; [ "$p" = 2d ] && other=physics3d_hash
  record "wasm32 lanes $p (atomics, one lane; node $(node --version))" \
    "$(node "$here/load.mjs" "$out/lanes-$p/wasm_determinism_bg.wasm" "$out/lanes-$p/wasm_determinism.js")" "$other"
done

# Each number agrees across every build that has it.
distinct=1
n=0
first="$(printf '%s' "$results" | head -n 1)"
for key in $KEYS; do
  n=$((n + 1))
  values="$(printf '%s' "$results" | cut -d '|' -f $((n + 1)) | grep -vx -- - | sort -u | wc -l | tr -d ' ')"
  [ "$values" = 1 ] || { distinct="$values"; og_err "$key: the builds disagree"; }
done
digest="$(printf '%s' "$first" | cut -d '|' -f 2)"
hash="$(printf '%s' "$first" | cut -d '|' -f 3)"
physics3d="$(printf '%s' "$first" | cut -d '|' -f 4)"
physics2d="$(printf '%s' "$first" | cut -d '|' -f 5)"
anim="$(printf '%s' "$first" | cut -d '|' -f 6)"

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
    printf 'grain, so animation runs serially either way); the scalar and simd128\n'
    printf 'wasm builds are single-threaded, with rapier built without its parallel\n'
    printf 'feature; the lanes builds (one per physics profile, as the 3D and 2D\n'
    printf 'lanes modules are built) have it and run on one lane under node, each\n'
    printf "with its own profile's physics hash only (- for the other).\n\n"
    printf '| build | math digest | bouncing boxes world_hash | physics 3D world hashes | physics 2D world hash | anim world hash and retarget |\n|---|---|---|---|---|---|\n'
    printf '%s' "$results" | awk -F '|' '{ printf "| %s | %s | %s | %s | %s | %s |\n", $1, $2, $3, $4, $5, $6 }'
    printf '\n'
    if [ "$distinct" = 1 ]; then
      printf 'All builds agree: math digest %s, world_hash %s, physics 3D %s, physics 2D %s, anim %s.\n' "$digest" "$hash" "$physics3d" "$physics2d" "$anim"
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
og_say "ok: every build agrees (math_digest $digest, bouncing_boxes_hash $hash, physics3d_hash $physics3d, physics2d_hash $physics2d, anim_hash $anim)"
