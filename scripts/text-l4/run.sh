#!/usr/bin/env bash
# The text section of the final L4 pass: run once over one main SHA on the GPU host.
#   scripts/text-l4/run.sh <og> <chrome> <out-dir> [packs]
# <og> is a built og, <chrome> a Chrome for Testing binary, packs default
# default,cartoon,fantasy,scifi. Needs only bun beyond them: every PNG is read and
# written with packages/engine/src/node/png.ts.
# OG_ENGINE_PACKAGE must point at this checkout's packages/engine with dist/web built
# (scripts/release-wasm.sh --development, then packages/engine/scripts/build.sh's copy).
# RT_TESTS, if set, is og-runtime's `tests` binary: the mixed-script and emoji
# reference frames are recorded with it into <out-dir>/mixed/.
#
# Writes under <out-dir>:
#   sheets/og-<pack>.png, sheets/chrome-<pack>.png, cmp/cmp-<pack>.png, cmp/rows.json
#   gallery/  (subtitles captions and meta/shop-confirm in every pack; defects.txt)
#   cold-native/cold-native.json, cold-web/cold-web.json (+ frames)
#   failures.txt: one line per failed step; the exit code is 1 when it is not empty.
set -u
here="$(cd "$(dirname "$0")" && pwd)"; root="$(cd "$here/../.." && pwd)"
og="$(realpath "$1")"; chrome="$(realpath "$2")"; out="$(realpath -m "$3")"; packs="${4:-default,cartoon,fantasy,scifi}"
mkdir -p "$out/sheets" "$out/cmp"
: > "$out/failures.txt"
fail() { echo "$1" | tee -a "$out/failures.txt" >&2; }
work="$(mktemp -d)"; trap 'rm -rf "$work"' EXIT
link() { mkdir -p "$1/node_modules/@origingame"; ln -sfn "$root/packages/engine" "$1/node_modules/@origingame/engine"; }

# 1. The sheets: og in each pack, Chrome in the same families, then row by row.
cp -R "$here/sheet" "$work/sheet"; link "$work/sheet"
if "$og" cook "$work/sheet" > "$out/sheets/cook.log" 2>&1; then
  for p in ${packs//,/ }; do
    "$og" run "$work/sheet" --headless --no-check --size 1200x620 --out "$work/fs-$p" \
      --commands "0.02:pack:$p;0.3:sheet;1.5:screenshot:sheet.png;1.6:quit" > "$out/sheets/run-$p.log" 2>&1 \
      && cp "$work/fs-$p/sheet.png" "$out/sheets/og-$p.png" || fail "sheet: og run in $p failed (sheets/run-$p.log)"
  done
else fail "sheet: og cook failed (sheets/cook.log)"; fi
bun "$here/chrome-sheets.ts" "$chrome" "$out/sheets" "$packs" || fail "sheet: Chrome reference sheets failed"
bun "$here/measure.ts" "$out/sheets" "$out/sheets" "$out/cmp" "$packs" > "$out/cmp/rows.txt" || fail "sheet: measure found rows off Chrome (cmp/rows.txt, stderr above)"

# 2. The captions (你在等谁？) and shop-confirm's → in every pack, through the gallery lane.
( cd "$root/packages/engine" && bun src/node/gallery.ts --og "$og" --only "subtitles · captions,meta/shop-confirm" \
    --packs "$packs" --schemes dark,light --out "$out/gallery" ) > "$out/gallery.log" 2>&1 \
  || fail "gallery: captions or shop-confirm defects (gallery/defects.txt)"

# 3. The mixed-script and emoji reference frames, when the test binary is given.
if [ -n "${RT_TESTS:-}" ]; then
  touch "$work/mark"
  OG_UI_CAPTURE=1 "$RT_TESTS" font_packs::mixed_script font_packs::colour_emoji --include-ignored > "$out/mixed.log" 2>&1 \
    || fail "mixed: font_packs reference tests failed (mixed.log)"
  mkdir -p "$out/mixed"; find "$root/crates/og-tests/references/vulkan" -name '*.png' -newer "$work/mark" -exec cp {} "$out/mixed/" \;
else fail "mixed: skipped (RT_TESTS not set)"; fi

# 4. The cold time to the first Han glyph, natively and in Chrome on this GPU.
cp -R "$here/cjk-game" "$work/cjk-native"; link "$work/cjk-native"
bun "$here/cold-native.ts" "$og" "$work/cjk-native" "$out/cold-native" || fail "cold-native: Han text did not draw by tick 30, or the run failed"
cp -R "$here/cjk-game" "$work/cjk-web"; link "$work/cjk-web"
bun "$here/cold-web.ts" "$og" "$chrome" "$work/cjk-web" "$out/cold-web" || fail "cold-web: a line never drew or went blank, no first frame, or a software adapter"
FACE_DELAY_MS=5000 bun "$here/cold-web.ts" "$og" "$chrome" "$work/cjk-web" "$out/cold-web-pending" || fail "cold-web-pending: with the Han faces held 5 s, a line never drew or went blank"

[ -s "$out/failures.txt" ] && { echo "text-l4: $(wc -l < "$out/failures.txt") failure(s), $out/failures.txt" >&2; exit 1; }
echo "text-l4: all steps passed ($out)"
