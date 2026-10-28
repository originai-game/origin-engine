# Functional lane at aa49ce7c (art follow-ups F1 and F2)

Machine: macrunner, main checkout `/Volumes/app/origin-engine`, `main`
fast-forwarded to `origin/main` at `aa49ce7c`. No other gate held the lock
during any of the runs below.

- F1 (`b281b8fc`): og-cook drops faces thinner than the position grid and
  fixes voxel normals on turned parts.
- F2 (`aa49ce7c`): the real Bayer matrix in `stylise.wgsl` and og-cook's
  `palette.rs`; rubber-hose-animation and gothic-contour drop posterize.

## Result

The functional lane passed at `aa49ce7c` with no reference failing. None of
the frames F1 and F2 move are among its checks:

- `stylise-pixel` is compared only by `og-gfx`.
- fx-showcase's looks suite runs in the full lane's bun-test.
- The art samples, hollowbell and brasswick have their own `og test` runs.

Each of these was run directly. Every frame that moved was read before it
was recorded, and 31 Metal frames were re-recorded. No frame was held back,
and no webgl2 reference was touched.

## Commands and step times

1. `scripts/check.sh functional` at `aa49ce7c`: gate PASSED (5/5 steps,
   122 s; 2 m 2 s wall)

   | step | result | time |
   |---|---|---|
   | fmt | PASS | 2 s |
   | clippy | PASS | 8 s |
   | test | PASS: nextest 1475 tests run, 1475 passed, 19 skipped (90 s) | 120 s |
   | drift | SKIPPED: no input changed since it passed on tree 2f85ebb7bc30 | 0 s |
   | bun-functional | PASS: knowledge 60/60, create 18/18 | 13 s |

2. `packages/engine` (`OG_GATE=1 OG_WASM_STALE=1 bun run test`): PASS,
   302 pass, 30 skip (the cross-host tests), 0 fail. bun test took 80 s and
   the whole run 7 m 46 s, including build.sh's wasm-opt. The known
   `feel.test.ts` timeout did not occur this run.

## What moved, and what was re-recorded

| Where | Check | Before | Re-recorded |
|---|---|---|---|
| `crates/og-tests/references/metal/stylise-pixel.png` | `og-gfx stylise-pixel` | RMSE 0.112 | 1 frame; RMSE 0 after |
| `examples/art-*/references/metal` | `tools/art-samples/capture.ts` | 9 of the 14 named styles moved | 9 frames |
| `templates/fx-showcase/references/metal/looks-*.png` | `og test templates/fx-showcase --filter tests/looks.og.ts` | 8 failed | 8 frames; 78/78 after |
| `examples/hollowbell/references/metal` (look extends 16bit-pixel) | `og test examples/hollowbell` | frames suite failed, 8 frames | 8 frames; 13/13 suites after |
| `examples/brasswick/references/metal` (look extends detailed-pixel) | `og test examples/brasswick` | frames suite failed, 5 frames, RMSE 0.012 to 0.016 | 5 frames; 8/8 suites after |

### The art samples

Each style was captured into a scratch directory at 1280×720. It was
measured against the committed Metal frame and against the lavapipe
evidence in `tests/evidence/art/samples/frames/`, which `e39ba3de`
re-rendered after the Bayer fix. Values are the mean difference at
320×180, out of 255.

| Style | new vs old Metal | new vs lavapipe | old vs lavapipe |
|---|---:|---:|---:|
| monochrome-dither-3d | 74.3 | 3.4 | 74.6 |
| monochrome-pixel | 19.3 | 3.7 | 19.7 |
| rubber-hose-animation | 16.1 | 1.7 | 16.3 |
| gothic-contour | 11.0 | 1.5 | 11.1 |
| 16bit-pixel | 5.3 | 4.6 | 6.9 |
| pixel-3d-hybrid | 4.5 | 3.3 | 5.4 |
| detailed-pixel | 1.8 | 1.9 | 2.7 |
| luminous-pixel | 1.8 | 3.2 | 4.0 |
| voxel-worlds | 0.07 (0.12 % of pixels) | 1.25 | 1.20 |

- Every Bayer frame moved toward its lavapipe frame.
- Read at full size and in ×3 crops:
  - The dither is a fine per-pixel Bayer pattern, not 4×4 clusters.
  - rubber-hose-animation reads sepia, with no olive or pink bands.
  - gothic-contour has smooth blue-grey tones, with no posterized bands.
  - monochrome-dither-3d and monochrome-pixel show the scene's shapes
    through the dither, where the old frames were coarse checkerboards.
- voxel-worlds moved only in the shading of turned parts (F1's normals).
  The scene is otherwise the same.

faceted-low-poly, sculpted-clay, paper-craft, minimal-geometric and
miniature-diorama are byte-identical to their committed frames, so nothing
was recorded for them. F1's dropped faces were sub-grid slivers that drew
nothing. `og style check` does see F1:

- voxel-worlds, paper-craft and minimal-geometric now pass, where they
  failed `ArtShape.normals flat` at 54fe4740.
- faceted-low-poly's hero drops from 430 to 336 faces. It is still over its
  200-face budget.

### fx-showcase looks, hollowbell, brasswick

- **fx-showcase:** the looks suite failed on exactly the eight Bayer slugs
  above, from 0.019 % to 61.3 % outliers. Every other look was within
  tolerance. Read old against new:
  - The station, the spheres, the crate and the UI are unchanged.
  - The dither is finer, with no olive, pink or brown banding.
  - rubber-hose-animation and gothic-contour lose their posterized flats.
- **hollowbell:** title, play, combat, talk, shop, pause, settings-zh and
  gloam moved from 0.03 % to 29 % outliers. Read:
  - The scene, the sprites, the HUD and the text are identical.
  - The brick and dark backgrounds dither in a finer Bayer pattern.
- **brasswick:** title, start, door, stomp and gate moved, by RMSE alone
  (0 % outliers). Same reading: only the dither in the dark areas is finer.

## Missing references

No test asked for a missing reference.
