# A1 `art/contract`: the art contract as data

Stream A1 of docs/art.md (§2.1, the A1 row of §3), branch `art/contract`
from `84c6bf12` (origin/main). Data only: nothing reads the art resources
yet (conformance is A2/A3, runtime defaults A4), so every existing game
cooks and draws as before.

## What landed

| Commit | What |
|---|---|
| `843a1ff3` | og-api: module `art` in `crates/og-api/api.ogt`, seven resources (`ArtPalette`, `ArtPixels`, `ArtTexels`, `ArtShape`, `ArtLine`, `ArtMotion`, `ArtCamera`) and six enums (`ArtRotation`, `ArtFilter`, `ArtMips`, `ArtNormals`, `ArtLinePlace`, `ArtProjection`), every field no constraint at zero. `og-cook/src/look.rs` `LOOK_RESOURCES` grows by `ART_RESOURCES`; the cook's look checks cover them (enums, ranges, the `accents` and `spriteSizes` lists, the `colors` palette handle and `@palettes/*`). The runtime's `applyLook` inside a system leaves an art resource the system did not declare as it is, instead of refusing the look (kestrel-stair, lantern-vale, hollowbell apply looks from systems declaring only the frame's resources). Generated outputs regenerated (`og api gen --wiki`): `host.rs`, schema, `.d.ts`, `bindings.json`, `wiki/art.md`, index, load, render. |
| `5367d55f` | The 37 presets in `templates/looks/` carry their catalogue entry's `palette` and `assets` as art blocks, each with a comment saying where the numbers come from; only additions, no frame field touched. `classical-painting-collage` fixes nothing (authored art). |
| `514fdba8` | `project.ogt` gains `look "looks/<id>.ogt"` (`og_cook::look::project_look`): the cook reads the named look's art contract before any asset cooks and notes on that look's report which art resources it sets; a line naming no look of the bundle is that asset's error, a malformed line fails the cook. The runtime takes it as `GameDesc.look`'s default (`engine::with_project`, as tick/seed/scenes/scene); both naming different looks is refused; `assets/`-prefixed ids are one look. Tests: og-cook unit + `tests/look_builtins.rs`, og-runtime `tests/art_contract.rs`. |
| `30500d0c` | `@origingame/create-game` writes the line into the scaffold's `project.ogt` (after `entry`, or replacing a template's own line), the same id as `GameDesc.look`; the look file's header and the README name the art blocks. |
| `a3d06f02` | og-game §2 (project.ogt's `look`) and §15 (the art contract: blocks, fields, example, zero = no constraint, the project line, reading it from game code, systems applying looks); og-taste §1 and §12 (the blocks, where each comes from in an entry, an override example). og-game names `reference/art.md` (the knowledge validator asks it). |
| `9b8bac77` | CHANGELOG `## Unreleased`: one line per IDL member (7 resources, 6 enums, `GameDesc.look`, `LookInfo.resources`, `og.render.applyLook`, `og.load.look`) plus the presets and the project line; `changelog.json` `unreleased.added` has the same 18 entries; no breaking entry, simulation unchanged. |

Art blocks per preset (`Art` prefix dropped):

```text
16bit-pixel            Palette Pixels Line Camera
8bit-pixel             Palette Pixels Camera
anime-cel-3d           Palette Texels Shape Line
black-figure-pottery   Palette Line Camera
bold-comic-illus.      Palette Texels Shape Line
cinematic-realism      Texels Shape Camera
classical-painting-c.  (none: authored)
coarse-pixel           Palette Pixels Camera
decorative-storybook   Palette Shape Line
detailed-doodle        Palette Line
detailed-pixel         Palette Pixels Camera
expressionist-painting Texels
faceted-low-poly       Palette Texels Shape
gothic-contour         Line
heavy-ink-shadows      Palette Shape Line
illuminated-manuscript Palette Line Camera
inked-comic-3d         Texels Shape Line
layered-silhouette     Palette
low-resolution-3d      Palette Pixels Texels Shape Camera
luminous-layered-p.    Shape
luminous-pixel         Palette Pixels Camera
lush-painted-worlds    Texels Shape
miniature-diorama      Palette Texels Shape
minimal-geometric      Palette Shape
monochrome-dither-3d   Palette Pixels Shape Camera
monochrome-pixel       Palette Pixels Camera
muted-cinematic-3d     Shape
neon-vector            Palette
painterly-3d           Texels Shape
paper-craft            Palette Shape
pencil-hatching        Shape Line
pixel-3d-hybrid        Palette Pixels Shape Camera
rubber-hose-animation  Palette Line Motion
sculpted-clay          Shape Motion
topdown-tile-pixel     Palette Pixels Camera
voxel-worlds           Palette Texels Shape
watercolor-wash        Palette Texels Shape
```

## Shared files touched

- `crates/og-api/api.ogt`: the `art` module appended after the last module;
  `GameDesc.look`, `LookInfo.resources`, `og.render.applyLook` and
  `og.load.look` docs. Nothing reordered.
- `templates/looks/*.ogt`: all 37, additions only (`git diff 84c6bf12 --
  templates/looks | grep '^-[^-]'` is empty).
- `crates/og-cook/src/cook/mod.rs`: one block that reads project.ogt's
  look before the cook and one that notes it on the look's report (A2/A3
  add their own calls into `cook/conform/`). `look::contract` in
  `cook/look.rs` returns the flattened look's art blocks: the entry point
  A2/A3 read.
- `crates/og-runtime/src/engine.rs` (`ProjectGame.look`, `Given.look`),
  `crates/og-runtime/src/look.rs` (undeclared art resources skipped).
- Generated: `crates/og-api/src/generated/host.rs`,
  `packages/engine/schema/api.schema.json`, `packages/engine/types/`,
  `packages/knowledge/wiki/` (`og api gen --wiki`: 36 of 36 unchanged
  after the last edit; `og api check` matches).
- `CHANGELOG.md`, `packages/engine/changelog.json` (unreleased).

## Verification

- `cargo check -p og-runtime -p og-cook --tests`, `cargo clippy -p og-cook
  -p og-runtime --tests -- -D warnings`, `cargo fmt --check`: clean.
- `cargo test -p og-cook --lib look`: 14 passed;
  `--test tests look_builtins`: 3 passed; `cargo test -p og-runtime --test
  tests art_contract`: 2 passed; `cargo test -p og-cli --bin og upgrade`:
  1 passed (reads changelog.json).
- `packages/create`: `bun test tests/`: 16 passed.
- `packages/knowledge`: `bun run test` (validator + 50 tests): passed.
- `og api check`: the generated files match api.ogt.
- A scaffold (`create --look 16bit-pixel --profile 2d`) cooks with the
  line; overriding a preset's art field to `0`, `false` or an empty list
  (`spriteSizes` with no values) cooks and lands in the flattened look.

## Frames read (lavapipe, 640×360, `og run --headless --commands`)

`og run` in this orb loads the published 0.6.6 addon from bun's cache
unless told otherwise, and that runtime refuses the presets' new blocks
(`'ArtPalette' is not a resource a look sets`): the frames were taken
with dev addons pointed at by `OG_NATIVE_ADDON`, one built at
`84c6bf12` (with its own `og`, so its presets cook without art blocks)
and one at this branch, and compared pixel for pixel
(`magick compare -metric AE`):

| Frame | Base vs branch |
|---|---|
| fx-showcase looks station: none, 16bit-pixel, anime-cel-3d, voxel-worlds, monochrome-pixel, faceted-low-poly (applied by the station's system through `look:<slug>`) | 0 pixels differ, each |
| a 2D scaffold extending 16bit-pixel, at 1.5 s | 0 |
| the same scaffold with `GameDesc.look` removed (the project line applies the look) | 0 against the scaffold with it, same world hash |
| hollowbell title at 2.5 s | 0 |
| lantern-vale title at 2.5 s | 0 |

Read: the station frames show the six treatments as before (the pixel
grid and dither of 16bit-pixel, the handheld-green dither of
monochrome-pixel, the anime-cel outlines; voxel-worlds and
faceted-low-poly read as the plain diorama, as docs/art.md §1 says: their
style is in assets A3 conforms). The montage is `frames-lavapipe.jpg`.
Golden replays (`og replay compare`, branch addon): `game-3d` and
`game-2d` same over 300 ticks.

## For the coordinator

- `world_hash` (the run's printed hash) moves for every game: the seven
  records join every world's registry. Frames and the replays' portable
  hash do not move. Say so at the merge if a check reads `world_hash`.
- The examples' music (`assets/music/*.wav`) is made by `bun
  tools/make-assets.ts` after a clone; an orb has to run it before `og
  run` of hollowbell, lantern-vale or kestrel-stair.
- No metal frame needs recording: no reference frame moves.
