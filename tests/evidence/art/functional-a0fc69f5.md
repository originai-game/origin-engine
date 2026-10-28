# Functional lane at a0fc69f5 (art wave A1–A6)

Machine: macrunner, main checkout `/Volumes/app/origin-engine`, detached at
`a0fc69f5` (origin/main). The checkout's only local state was two untracked
directories, `tests/evidence/r4/bench-complex/{mac,mac-reference}/` (a bench
run of 26 Oct, not tracked by origin/main, read by no gate step), left as
they were. No other gate held the lock during any run below.

## Result: FAILED

The expectation "no reference frame moves" does not hold. Its premise is
wrong for scaffolds. Since `30500d0c` ("create: project.ogt names the
game's look"), every project `create` makes names a look in project.ogt. The
default scaffold's look extends `faceted-low-poly`, so the art contract is on
for the scaffolded smoke game. The A3 mesh conform pass then decimates its
dropship, and the playtest frames `intro-4` and `intro-8` move.

The GPU reference frames of og-tests (the `test` step) do not move: 1472 of
1472 pass.

## Commands and step times

1. `scripts/check.sh functional` (gate FAILED at bun-functional, 89 s wall)

   | step | result | time |
   |---|---|---|
   | fmt | PASS | 3 s |
   | clippy | PASS | 15 s |
   | drift | PASS | 67 s |
   | bun-functional | FAIL (exit 1), in `packages/create` | 6 s |
   | test | CANCELLED when bun-functional failed | 86 s (partial) |

   bun-functional reported "the web module predates a crate change; the
   cross-host tests skip (OG_WASM_STALE=1)". It stops at the first failing
   package, so `packages/engine` and `packages/knowledge` did not run.

2. `scripts/check.sh --only test` (the cancelled step on its own; `--only`
   adds wasm-build, which test reads): gate PASSED (2/2 steps, 310 s)

   | step | result | time |
   |---|---|---|
   | wasm-build | PASS | 151 s |
   | test | PASS: nextest 1472 tests run, 1472 passed, 18 skipped (151 s) | 159 s |

3. The two packages bun-functional never reached, run the way the step runs
   them (`OG_GATE=1 OG_WASM_STALE=1 bun run test`, one at a time):

   | package | result | time |
   |---|---|---|
   | packages/engine | FAIL: 298 pass, 30 skip, 4 fail (bun test 166 s) | 9 m 53 s (build.sh ran wasm-opt itself; the functional lane has no wasm-opt step) |
   | packages/knowledge | PASS: 60 pass | 5 s |

## Failures

### 1. Scaffold playtest frames moved: intro-4, intro-8 (engine `template.test.ts`, `no-renderer.test.ts`)

```
FAIL intro (17 checks, 5106 ms): the intro plays its rail camera, captions, clips, event and fades from timeline:play:intro
  - intro-4.png matches intro-4.png: 3.083 % outliers (≤ 0.5 %), RMSE 0.05754 (≤ 0.01)
  - intro-8.png matches intro-8.png: 4.588 % outliers (≤ 0.5 %), RMSE 0.06988 (≤ 0.01)
og test: 1 of 3 suite(s) failed
```

The two tests share one cause: `a scaffolded game cooks, passes its playtest
and leaves the frame and dumps` and `og test without a renderer passes,
counts its unchecked frames and runs the same game`.

Frames read (the old reference and the new capture,
`target/evidence/template/smoke-game/{references/metal,og-out/test/intro}/`):

- The UI card, caption, camera, walker and wall match.
- The HUSK H.I. dropship in the new frame has lost its hull. Only scattered
  flat quads and triangles remain, with no texture detail, and its shadow on
  the wall and floor is gone or broken.

The cook's conform log of the scaffold (`cooked/.conform.ogt`) names the
cause:

```
output "models/dropship.glb" … "16512 → 96 triangles; ArtShape.propFaces 200 (16512 drawn before, decimated); normals flat"
output "models/walker.glb"   … "108 → 108 triangles; ArtShape.characterFaces 1500 (108 drawn before, within); normals flat"
output "textures/stone.png"  … "conformed as a mesh texture (no model claims it; …); 512x512 → 64x64; ArtTexels.maxSize 64; 499 → 32 colours (ArtPalette.maxColors 32); filter nearest (pixel art: RGBA8, no mips)"
```

Commits involved:

- A1 arms the contract: `30500d0c` makes the scaffold name its look in
  project.ogt. `5367d55f` gives the `faceted-low-poly` preset an art
  contract, and `514fdba8` makes the cook read it.
- A3 applies it. `d7b9de23` (conform::mesh, merged in `e5db08aa`) decimates
  the dropship. `8ac3d7e1` and `c39889b4` (merged in `5b62b60d`) conform the
  unclaimed stone texture.
- `c52344b2` lowered faceted-low-poly's prop budget from 500 to 200. At 500
  the dropship would still drop from 16512 to at most 500 triangles, so this
  commit is not the root.

I attributed these from the conform log and the frame diff; I did not bisect.
No frame was re-recorded. The change is not one whose reason can be named:
a scaffold's showcase model should not fall apart. The fix is a decision for
the coordinator. Options include a decimator that keeps the silhouette, a
scaffold look whose budget fits its own assets, or the scaffold's models
opted out.

### 2. Art enums have no column factory export (engine `api-surface.test.ts`)

```
(fail) every column factory and constant the IDL declares is a named export
+   "ArtRotation (column factory)",
+   "ArtFilter (column factory)",
+   "ArtMips (column factory)",
+   "ArtNormals (column factory)",
+   "ArtLinePlace (column factory)",
+   "ArtBrush (column factory)",
+   "ArtProjection (column factory)",
```

`843a1ff3` (A1) added six enums to the IDL, and `9cdad4c4` (A2) added
`ArtBrush`. `bindings.json` and `index.d.ts` declare factories for them, but
`packages/engine/src/index.ts` does not export one (compare `PathKind`,
`src/index.ts:98`).

### 3. Create test compares a tmp path against its realpath (create `create.test.ts:389`)

```
(fail) the published package briefs from its own presets, catalogue and previews
Expected: "/var/folders/…/og-create-4iPJTy/node_modules/@origingame/create-game/previews/monochrome-pixel.webp"
Received: "/private/var/folders/…/og-create-4iPJTy/node_modules/@origingame/create-game/previews/monochrome-pixel.webp"
```

This test came in `80a24cdd` (A5). On macOS `/var` is a symlink to
`/private/var`. The brief resolves the real path and the test expects the
tmpdir path. The bug is in the test, which fails only on macOS.

### 4. Timeout under load, not reproduced (engine `feel.test.ts`)

```
(fail) og.Sprite draws at its entity, y-sorted, with a blob shadow under it [45334.68ms]
  ^ this test timed out after 5000ms.
```

Run alone (`bun test tests/feel.test.ts`), it passes in 128 ms with 6 of 6
passing. It timed out under `bun test --parallel` on a loaded machine. I
found no art cause.

## Missing references

No test asked for a missing reference.
