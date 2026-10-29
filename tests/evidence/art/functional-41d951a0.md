# Functional lane at 41d951a0 (after the look-leak and rumpus fixes)

Machine: macrunner, main checkout `/Volumes/app/origin-engine`, `main`
fast-forwarded to `origin/main` at `41d951a0`. No other gate held the lock.
Follows `functional-c70faca9.md`, which held back three items:

- `8b5aebff`: the engine no longer carries a turned-off Sky across frames.
- `cdf237d9`: the looks station saves and restores its sky.
- `4d3af9d2`: the `MaterialSurface` column factory is exported.
- `41d951a0` (G1's rumpus fix): the surface bump no longer produces NaN
  where the normal lies in the face; rumpus-isle's look sets `surface none`.

## Result: GREEN

Every check passes. The five fx-showcase looks G1 and G2 move are
re-recorded on Metal. rumpus-isle passes as recorded, and nothing else
moved.

## Commands and step times

1. `scripts/check.sh functional`: gate PASSED (5/5 steps, 125 s; 2 m 5 s
   wall)

   | step | result | time |
   |---|---|---|
   | fmt | PASS | 2 s |
   | clippy | PASS | 7 s |
   | test | PASS: nextest 1486 tests run, 1486 passed, 19 skipped (98 s) | 122 s |
   | drift | SKIPPED: no input changed since it passed on tree 36eeefcb1879 | 0 s |
   | bun-functional | PASS | 13 s |

2. `packages/engine` (`OG_GATE=1 OG_WASM_STALE=1 bun run test`): 301 pass,
   30 skip, 1 fail; bun test 75 s, 6 m 42 s in total.
   - `api-surface`'s column-factory test passes (4d3af9d2).
   - `rooms.test.ts` "a room playtest's quit and its then step land on one
     tick for every place" ran 67.6 s under `--parallel` with no assertion
     message, a load timeout. Alone it passes in 1.35 s (3/3). This is a
     second flake of the same kind as `feel.test.ts`'s, which passed this
     run.
3. `og test` directly:

   | project | result |
   |---|---|
   | `templates/fx-showcase --filter tests/looks.og.ts` | FAIL on exactly the 5 expected looks; after re-recording, PASS 78/78 |
   | `examples/rumpus-isle` | PASS 9/9 suites, no frame moved |
   | `examples/hollowbell` | PASS 13/13 suites |
   | `examples/brasswick` | PASS 8/8 suites |

   `look:none restores the plain diorama` passes.

4. The art samples (`tools/art-samples/capture.ts`, all 37 styles):
   - 34 are byte-identical to the frames committed in `d2929e9f`.
   - sculpted-clay, paper-craft and miniature-diorama differ in bytes only.
     1, 1 and 15 pixels differ, each by at most one level of 255 (likely the
     bump texels the NaN fix touches).
   - Nothing was re-recorded.

## fx-showcase looks re-recorded (5 frames)

| Look | outliers before | read |
|---|---:|---|
| heavy-ink-shadows | 95.6 % | black sky, ink-black shadow sides, as in its art-sample frame |
| low-resolution-3d | 95.3 % | the dusk preset: darker sky, the chrome sphere reflects it nearly black |
| sculpted-clay | 32.6 % | `surface clay`: every sphere and the tree carry the clay bump |
| miniature-diorama | 6.3 % | `surface plastic`: the gold and chrome spheres turn matte plastic |
| paper-craft | 2.9 % | `surface paper`: matte faceted paper, no reflections |

The station, the UI, the crate and the layout are unchanged in each frame.
No webgl2 reference was touched.

## Missing references

No test asked for a missing reference.
