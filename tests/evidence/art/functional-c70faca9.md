# Functional lane at c70faca9 (art follow-ups G1, G2, G3)

Machine: macrunner, main checkout `/Volumes/app/origin-engine`, `main`
fast-forwarded to `origin/main` at `c70faca9`. No other gate held the lock.

- G2 (`08c9da54`): `MaterialStyle.shadowDarkness`; the cook paints mesh
  colour textures in texel space under a painted look; presets for
  heavy-ink-shadows and low-resolution-3d.
- G3 (`63717bdc`): the style check's grid detector; mesh textures sized by
  UV span (conform-texture v3); a top-down tile set; a realistic cinematic
  set.
- G1 (`c4eb7ea1`): `MaterialStyle.surface` (clay, paper, plastic); type
  models built at the budget's segments; conform character; shape-3d
  presets.

## Result: NOT GREEN, stopped

The gate passed and the 11 art-sample frames these changes move are
re-recorded. Three things fail that I did not re-record, because the frames
look wrong or the failure is code:

1. **fx-showcase looks: a look leaves state behind.** 33 of 37 looks fail,
   and `look:none restores the plain diorama` fails (RMSE 0.368). The
   failures start at heavy-ink-shadows: every look after it in the station's
   order fails, and the looks before it pass (except sculpted-clay, which
   moves on purpose). Read:
   - From heavy-ink-shadows on, every frame has a near-black sky, and the
     chrome sphere draws black. coarse-pixel, anime-cel-3d and
     layered-silhouette, which nothing changed, draw this way too.
   - `looks-none.png` keeps the black sky and the black chrome sphere;
     `looks-plain.png` (before any look) is the normal diorama.
   - Most likely a field G2 added (`MaterialStyle.shadowDarkness`, or what
     heavy-ink-shadows' new preset sets) that a later look and `look:none`
     do not reset. Not bisected.
2. **rumpus-isle frames (look extends miniature-diorama).** 9 of 11 frames
   fail (title, lobby, card, bump, splash, scores, berry, crumble, podium;
   1 % to 20 % outliers). Read:
   - The islands' grass is a much more saturated green; that may be the
     intended miniature-diorama change.
   - In bump and berry, black blobs appear at the bottom centre over the
     ring's edge, by the HUD. That is a defect, so none of the 9 frames
     were re-recorded.
3. **`packages/engine` api-surface test.** `every column factory and
   constant the IDL declares is a named export` fails on `MaterialSurface
   (column factory)`. G1 (`44514cd8`) added the enum to the IDL, and
   `packages/engine/src/index.ts` exports no factory for it, as fb1c2ec5
   once fixed for the Art* enums. This is a code fix, outside what I was
   authorized to commit.

## Commands and step times

1. `scripts/check.sh functional` at `c70faca9`: gate PASSED (5/5 steps,
   191 s; 3 m 12 s wall)

   | step | result | time |
   |---|---|---|
   | fmt | PASS | 2 s |
   | clippy | PASS | 13 s |
   | test | PASS: nextest 1484 tests run, 1484 passed, 19 skipped (147 s) | 189 s |
   | drift | PASS | 52 s |
   | bun-functional | PASS: knowledge, create | 18 s |

2. `packages/engine` (`OG_GATE=1 OG_WASM_STALE=1 bun run test`): FAIL, 300
   pass, 30 skip, 2 fail; bun test 143 s, 8 m 43 s in total.
   - `every column factory and constant the IDL declares is a named export`:
     `MaterialSurface (column factory)` (above).
   - `og.Sprite draws at its entity, y-sorted, with a blob shadow under it`:
     the known `--parallel` timeout; alone it passes in 132 ms.
3. `og test` directly:

   | project | result |
   |---|---|
   | `templates/fx-showcase --filter tests/looks.og.ts` | FAIL (above) |
   | `examples/hollowbell` | PASS 13/13 suites |
   | `examples/brasswick` | PASS 8/8 suites |
   | `examples/rumpus-isle` | FAIL, frames suite 9 frames (above) |

## Art samples re-recorded (10 frames)

All 37 styles were captured with `tools/art-samples/capture.ts` into a
scratch directory at 1280×720. 27 are byte-identical to their committed
Metal frames. The 10 that moved are exactly the named styles, and each new
frame sits close to its refreshed lavapipe frame in
`tests/evidence/art/samples/frames/` (mean difference at 320×180, of 255):

| Style | new vs old Metal | new vs lavapipe | old vs lavapipe |
|---|---:|---:|---:|
| heavy-ink-shadows | 116.8 | 3.1 | 116.9 |
| low-resolution-3d | 63.5 | 2.2 | 63.3 |
| cinematic-realism | 45.4 | 1.5 | 45.3 |
| topdown-tile-pixel | 34.3 | 5.0 | 34.5 |
| muted-cinematic-3d | 23.2 | 1.0 | 23.3 |
| paper-craft | 18.8 | 3.4 | 20.2 |
| sculpted-clay | 18.8 | 2.9 | 20.5 |
| painterly-3d | 6.9 | 1.2 | 7.3 |
| miniature-diorama | 5.9 | 0.7 | 6.0 |
| faceted-low-poly | 1.5 | 0.7 | 2.0 |

Read against the lavapipe frames and the strips in `full/` and `shape-3d/`:

- cinematic-realism and muted-cinematic-3d draw the realistic set (the
  man, the camp, the conifers, the lamp post).
- topdown-tile-pixel draws the tile set with its auto-tiled path, cliff and
  water.
- heavy-ink-shadows and low-resolution-3d match their new presets: a black
  sky with ink shadows; a dusk palette.
- paper-craft has fibre grain on the ground and its card path; sculpted-clay
  has a clay grain on the ground.
- faceted-low-poly, miniature-diorama and painterly-3d move only slightly.

The 10 frames are committed. The brief named 10 styles (the
heavy-ink-shadows and low-resolution-3d presets, the cinematic set,
topdown-tile-pixel, four shape-3d styles), and all 10 moved.

## Contact sheet

`tests/evidence/art/samples/contact-sheet.jpg` is rebuilt from the
refreshed `frames/*.jpg` and `packages/knowledge/previews/*.webp`, laid out
as A7's: each frame beside its preview at 320×180, captioned
`<slug> (<family>): frame | preview`, 4 columns with 6 px gaps on
`#16161a`, JPEG quality 80, sorted by slug. This Mac's ImageMagick has no
Freetype for the captions, so the same layout was drawn with Pillow.

## Missing references

No test asked for a missing reference.
