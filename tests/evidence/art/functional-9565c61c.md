# Functional lane at 9565c61c (art wave, after the decimator fix)

Machine: macrunner, main checkout `/Volumes/app/origin-engine`, `main`
fast-forwarded to `origin/main` at `9565c61c`. No other gate held the lock
during any of the runs below. Follows `functional-a0fc69f5.md`.

## Result: PASSED

- `scripts/check.sh functional` passed.
- The engine package was run separately (see below) and passed with 0 failures.
- The scaffold's `intro-4` and `intro-8` match the existing Metal
  references again, so no frame was re-recorded.
- The og-tests GPU reference frames did not move.

## Commands and step times

1. `scripts/check.sh functional`: gate PASSED (5/5 steps, 122 s; 2 m 3 s wall)

   | step | result | time |
   |---|---|---|
   | fmt | PASS | 2 s |
   | clippy | PASS | 13 s |
   | test | PASS: nextest 1473 tests run, 1473 passed, 19 skipped (83 s) | 120 s |
   | drift | SKIPPED: no input changed since it passed on tree 909173218d43 (a0fc69f5) | 0 s |
   | bun-functional | PASS: knowledge 60/60, create 18/18 | 11 s |

   The checkout had no wasm-opt output (`target/og-wasm/opt-all.txt`), so
   bun-functional printed "no wasm built here yet; the packages that need
   none". As a result, `packages/engine` did not run in the lane.

2. `packages/engine`, run as bun-functional would have run it
   (`OG_GATE=1 OG_WASM_STALE=1 bun run test`): PASS, 302 pass, 30 skip
   (the cross-host tests, `OG_WASM_STALE`), 0 fail. bun test took 74 s and
   the whole run 3 m 50 s, including build.sh's wasm-opt.
   - `every column factory and constant the IDL declares is a named export`
     passed (fb1c2ec5).
   - `a scaffolded game cooks, passes its playtest and leaves the frame and
     dumps` passed in 9.6 s.
   - `og test without a renderer passes, counts its unchecked frames and runs
     the same game` passed in 21.0 s.
   - `og.Sprite draws at its entity, y-sorted, with a blob shadow under it`
     passed in 0.66 s under `--parallel`, with no timeout this time.

`packages/knowledge` already ran inside bun-functional (60/60), so it was
not run again.

## The intro frames

- `intro-4` and `intro-8` are within tolerance of
  `templates/game/references/metal/`.
- Frame read (`intro-8`):
  - Both HUSK H.I. dropships are whole, with their hull, hazard stripes,
    decals and landing struts.
  - Their shadows fall on the wall and the floor.
  - The UI card, the caption "Here they come.", the camera, the walker and
    the wall are as in the reference.
- The scaffold's conform log no longer lists the dropship (9565c61c's
  `conform { off "models/dropship.glb" }`). It still conforms
  `textures/stone.png` (512×512 → 64×64, 32 colours, nearest). That change
  stays within the frames' tolerance.

## Missing references

No test asked for a missing reference.
