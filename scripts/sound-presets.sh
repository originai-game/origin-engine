#!/usr/bin/env bash
# shellcheck source-path=SCRIPTDIR
# scripts/sound-presets.sh [og]: the procedural sound-preset library end to
# end. `og sfx library` writes every preset's TS declaration into a scratch
# project (target/sound-presets/content/sfx), `og cook` synthesises them into
# cooked sound blobs (target/sound-presets/cooked), and each preset is
# rendered to a WAV twice: the former test sample records
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
mkdir -p "$work/content/sfx" "$work/wav" "$work/src" "$(dirname "$out")"
printf 'export default {};\n' > "$work/src/main.ts"
printf '{"version":1,"name":"Sound presets","engine":"0.9.0"}\n' > "$work/og-project.json"
printf 'import {defineProject} from "@origingame/engine/author";\nexport default defineProject({id:"sound-presets",entry:"src/main.ts",create:{form:"2d"}});\n' > "$work/project.ts"
"$og" sfx library "$work/content/sfx"
"$og" cook "$work" >/dev/null
sha() { shasum -a 256 "$1" | cut -d' ' -f1; }
{
  printf '{\n  "rate": 48000,\n  "presets": {\n'
  first=1
  for source in "$work"/content/sfx/*.ts; do
    name="$(basename "$source" .ts)"
    "$og" sfx "$name" --out "$work/wav/$name-a.wav" >/dev/null
    "$og" sfx "$name" --out "$work/wav/$name-b.wav" >/dev/null
    a="$(sha "$work/wav/$name-a.wav")"
    b="$(sha "$work/wav/$name-b.wav")"
    [ "$a" = "$b" ] || { og_err "$name rendered differently twice"; exit 1; }
    bytes=$(( $(wc -c < "$work/wav/$name-a.wav") - 44 ))
    size="$("$og" assets --project "$work" --json "base/sfx/$name" | jq -er --arg id "base/sfx/$name" '.assets[] | select(.id == $id) | .bytes')"
    [ $first = 1 ] || printf ',\n'
    first=0
    printf '    "%s": { "seconds": %s, "cookedBytes": %s, "wavSha256": "%s", "rendersEqual": true }' \
      "$name" "$(awk -v b="$bytes" 'BEGIN { printf "%.3f", b / 2 / 48000 }')" "$size" "$a"
  done
  printf '\n  }\n}\n'
} > "$out"
og_say "sound-presets: $(find "$work/content/sfx" -type f | wc -l | tr -d ' ') presets cooked into $work/cooked; $out"
