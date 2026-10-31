#!/usr/bin/env bash
# shellcheck source-path=SCRIPTDIR
# scripts/sound-presets.sh [og]: the procedural sound-preset library end to
# end. `og sfx library` writes every preset's `type sfx` .ogt into a scratch
# project (target/sound-presets/assets/sfx), `og cook` synthesises them into
# cooked sound blobs (target/sound-presets/cooked), and each preset is
# rendered to a WAV twice: tests/evidence/E6/audio/presets.json records
# each preset's length, its cooked blob's size and the SHA-256 of both
# renders (equal: the synthesis is deterministic). `og` defaults to
# target/debug/og.
set -euo pipefail
# shellcheck source=lib.sh
. "$(dirname "$0")/lib.sh"
og="${1:-$OG_TARGET_DIR/debug/og}"
[ -x "$og" ] || { og_err "no og at $og: cargo build -p og-cli first, or pass its path"; exit 1; }
work="$OG_TARGET_DIR/sound-presets"
out="$OG_ROOT/tests/evidence/E6/audio/presets.json"
rm -rf "$work"
mkdir -p "$work/assets/sfx" "$work/wav" "$work/src" "$(dirname "$out")"
printf 'export default {};\n' > "$work/src/main.ts"
printf 'type project\nversion 1\n---\nname "Sound presets"\nid "sound-presets"\nengine "0.7.7"\nentry "src/main.ts"\nprofile "2d"\n' > "$work/project.ogt"
"$og" sfx library "$work/assets/sfx"
"$og" cook "$work" >/dev/null
sha() { shasum -a 256 "$1" | cut -d' ' -f1; }
{
  printf '{\n  "rate": 48000,\n  "presets": {\n'
  first=1
  for ogt in "$work"/assets/sfx/*.ogt; do
    name="$(basename "$ogt" .ogt)"
    "$og" sfx "$name" --out "$work/wav/$name-a.wav" >/dev/null
    "$og" sfx "$name" --out "$work/wav/$name-b.wav" >/dev/null
    a="$(sha "$work/wav/$name-a.wav")"
    b="$(sha "$work/wav/$name-b.wav")"
    [ "$a" = "$b" ] || { og_err "$name rendered differently twice"; exit 1; }
    bytes=$(( $(wc -c < "$work/wav/$name-a.wav") - 44 ))
    cooked="$(find "$work/cooked" -name "sfx_$name*" -o -name "$name.ogt*" | head -1)"
    size=0
    [ -n "$cooked" ] && size=$(wc -c < "$cooked" | tr -d ' ')
    [ $first = 1 ] || printf ',\n'
    first=0
    printf '    "%s": { "seconds": %s, "cookedBytes": %s, "wavSha256": "%s", "rendersEqual": true }' \
      "$name" "$(awk -v b="$bytes" 'BEGIN { printf "%.3f", b / 2 / 48000 }')" "$size" "$a"
  done
  printf '\n  }\n}\n'
} > "$out"
og_say "sound-presets: $(find "$work/assets/sfx" -type f | wc -l | tr -d ' ') presets cooked into $work/cooked; $out"
