# Art wave A4: the art contract at run time (`art/runtime`)

Stream A4 of docs/art.md §3: the runtime rows of §2.4. Base: `58356904`
(A1 and A2 merged). Each piece is off at the zero value. Without
project.ogt's `look` line, none of the contract defaults apply, so an
existing game draws as before.

## Commits

| Commit | What |
|---|---|
| `3c506484` | og-tests: `sprites2d::an_overlay_sprite_on_a_nearest_texture_draws_on_its_first_frame`, the regression test for 1bb65d66. With the filter sync moved back after the overlay's sprite groups are made, it panics with "made before the pass". |
| `efd28441` | og-render: `TextureFilter::CleanEdge` (with `is_nearest()`), `RenderScene::sprite_outlines` (`SpriteOutline`, `MAX_SPRITE_OUTLINE` 4), and `RenderScene::paletted_texture(index, palette, row)`. In sprite.wgsl: the outline (instance locations 9 and 10, the quad grown by the width, a clip to the region, a diamond test at logical-pixel centres, blended under soft alpha) and a WGSL port of cleanEdge (`clean_edge()`, selected per sprite group). Also a README row and three pixel tests in sprites2d.rs. |
| `08b59d05` | og-anim: `install_with(world, Option<StepDefault>)` reads a resource's f32 field each tick as the step for players whose own `stepFps` is 0. The component is never written. Test: `playback::a_player_without_its_own_step_takes_the_contracts`. |
| `d9c5e3fa` | og-api: adds `Sprite.outline`, `outlineColor`, `palette`, `paletteRow` and `Tilemap.palette`, `paletteRow` (appended, nothing reordered). Regenerated with `og api gen --wiki` (the wiki moves only by line numbers). Also the CHANGELOG lines, the changelog.json `unreleased.added` entries and the breaking entry's A4 sentence. |
| `a6546031` | og-runtime: `src/art.rs` and its wiring (below). |

## What it does

The rule: under project.ogt's `look` (`art::contract`), the look's art
resources set runtime defaults. They are read at extraction, except the
camera, which is written when a look is applied, and the animation step.

- **Sprite outline.** `Sprite.outline` gives the width in logical pixels
  (0 to 4). `outlineColor` is linear. The sprite shader draws the outline
  from the texture's alpha as a diamond of whole logical pixels outside
  the art, and never takes a neighbouring frame of the sheet. A logical
  pixel is 1 / `ArtPixels.unit` world units under the contract, else one
  texel. When the width is unset, the contract's `ArtLine.width` applies
  only to a texture the cook did not line: a texture sampled linear,
  where the contract sets a unit. Pixel art already carries the cook's
  line, so it gets no second one. `outlineColor` at its default takes
  `ArtLine.color`.
- **Palette swap.** `Sprite.palette` / `Tilemap.palette` is a texture of N
  rows, and `paletteRow` picks the row (past the last row, the last row
  is used). The renderer draws the cook's `<id>#index` texture through that
  row. The format is in og-cook README §Art conformance. `<id>#index` is
  loaded render-only (`assets::link_index`) and never becomes a game
  handle. The swap is computed once on the CPU and cached per
  (index, palette, row) by texture revisions, so the shader is unchanged.
  Flipbook frames and afterimages swap too. A texture without an index
  texture draws as it is, with a single warning.
- **cleanEdge.** `ArtPixels.rotation cleanEdge` turns every
  nearest-sampled texture into `TextureFilter::CleanEdge`, and turns it
  back when the contract no longer applies. This is torcado's cleanEdge
  (MIT; the notice is in third_party/cleanedge from A2), run in the
  sprite shader. There is no TextureFilter in the IDL: the look selects
  the filter.
- **Camera.** `ArtCamera` applies only when a look is applied
  (`look.rs` → `art::apply_camera`), and only under the contract.
  `projection` sets `Camera.orthographic`. `fixedAngle` sets the rotation
  from `pitch` (below the horizon) and `yaw` (about +Y). `fovMin`/`fovMax`
  clamp `fov`. Fields at zero leave the camera alone. Inside a system that
  does not declare `og.Camera` among its writes, the camera is not
  touched. A game that sets its camera in `start` keeps its own, because
  the look applies first. `pixelPerfect` uses the existing pixel-snap
  path (`fx::snap_to_grid`) with the contract's grid: `logicalHeight`
  rows, else 1 / `unit`. `Stylise.pixelSnap` still wins when it is set.
- **stepFps.** `ArtMotion.stepFps` is the default of
  `AnimationPlayer.stepFps` for players at 0, read each tick (the
  component keeps 0).
- **SDF HUD icons** (carried over from A2). The cook skips
  `@styles/pixel/icons.png` because it is a distance field. Under a
  strict palette, `art::palette_icons` tints icon-atlas sprites and
  icon-font texts to the Oklab-nearest palette colour at emit time.

## Frames (lavapipe, 640×360 unless noted)

- `outline-palette.png` (the `/tmp/g2d-demo` scratch game: game-2d with
  its look line and demo sprites, crop ×3). From left: the courier
  unswapped, then through palette rows 0 and 1 of a 24×2 palette (reds,
  then blues). Then a linear arrow, and the coin turned 0.5 rad.
- `contract-line.png` (crop ×4, with `set:og.ArtLine.color=1,0,0`). The
  linear arrow takes the contract's one-logical-pixel line, in red. The
  pixel-art coin and courier, which the cook lined, take none. At the
  default 0.02 colour the line is dark on the dark sky and hard to see.
  This is why the colour was set to red for the frame.
- `cleanedge-coin.png` (crop ×6). The turned coin through cleanEdge
  (left) has a round, clean rim. After `set:og.ArtPixels.rotation=1`
  (nearest, right), the rim shows stair-step teeth. Base vs branch on
  game-2d with its look line differs by 38 px. With rotation set to
  nearest the branch equals base exactly (0 px), so those 38 px are
  cleanEdge.
- `tile-palette.jpg`. The tilemap with `palette: swap, paletteRow: 1`:
  the city's purples draw as blues, and the sprites are untouched.
- `icons-strict.jpg`. game-2d with a look extending monochrome-pixel
  (strict handheld4). Base (left) has red hearts and a yellow coin icon.
  The branch (right) draws the hearts, coin and pause icons in the
  palette's greens.
- `camera-preset.jpg`. game-3d (scratch copy with the template's
  quaternion line removed) with `ArtCamera { projection perspective
  fixedAngle true pitch 20 yaw 15 fovMax 40 }`. Base (left) keeps the
  level camera. The branch (right) is pitched 20° down and turned 15°,
  with fov clamped 50 → 40 (`get:og.Camera`: quaternion -0.172, 0.129,
  0.023, 0.976; fov 40).
- `step-walker.png` (fx-showcase stylised station with a look line, `ArtMotion
  { stepFps 2 }`, crop ×2). The base walker at 3.1 s and 3.3 s, then the
  branch at the same two times. Base changes pose; the branch holds
  one pose (the step lasts 0.5 s).
- pixelPerfect: with `Stylise.pixelSnap=false` and a sub-pixel camera
  move (+0.03, +0.01), the static skyline differs by 0 px with
  `pixelPerfect true` and by 3 px with it false. No frame is kept for
  this check.
- `anime-cel.jpg` (1280×720, crops). The rows are: the showcase's
  `looks-anime-cel-3d` station as drawn; the same with
  `MaterialStyle.toon=false`; the same with `environmentIntensity 0`;
  then the `metal` reference's crop.

## The anime-cel-3d finding (docs/art.md §2.4)

This is not a bug. At full size, the lavapipe frame and the `metal`
reference both show two-tone cel bands and dark outlines on the matte
spheres and the boxes. `MaterialStyle.toon=false` changes 73 % of the
pixels and makes the spheres smooth. The finding was read from a 480×270
thumbnail in a montage, where the bands and the 2.5 px outline are lost.

The gold and chrome spheres do read as unbanded, for a real reason: the
toon diffuse is scaled by (1 − metallic), as PBR diffuse is. A metal
therefore shows only its IBL reflection. With `environmentIntensity 0`
they go black, keeping the outline. This is the intended shading of
metals. Making metals band would need a toon specular ramp, which is a
material feature and not part of this stream.

## Courier cells off `spriteSizes`: the cook, not the runtime

The cook should handle these cells. A cell's size is a property of the
asset: the cook bakes it once, together with the sheet layout, the
flipbook rects and the index texture. The runtime only draws the rects
it is given. Resizing at run time would resample pixel art, which is
the blur the contract exists to prevent, and it would not change the
cells' layout. The cook's warning stays, with the fix it names
(`conform { size N }` per asset). A possible follow-up for the cook is
padding a cell up to the nearest `spriteSizes` entry when it is
smaller, instead of only warning.

## Verification

- `cargo test -p og-render --test tests shaders`: 10 passed. Every
  backend translation of the changed sprite.wgsl passes.
- `cargo test -p og-tests --test tests sprites2d::`: 12 passed. This
  includes the first-frame regression test and the outline, cleanEdge
  and palette tests. Each was checked to fail with its feature broken.
- `cargo test -p og-anim`: 75 + 24 passed. `cargo test -p og-runtime
  --lib`: 24 passed. `cargo test -p og-runtime --test tests --
  --test-threads=1`: 61 passed, 2 ignored.
  - Under cargo's default threads, a different stream test fails each
    run (`stream_fetch`, `stream_chunked`, `baked_scenes`). These tests
    share global host hooks across threads and pass alone and under
    nextest's one process per test. They don't touch the art code.
- `cargo clippy -p og-runtime -p og-render -p og-anim --all-targets -D
  warnings` and `cargo fmt`: clean.
- `og api check`: the generated files match api.ogt.
- Zero value: templates/game-2d without a look line, base vs branch, 0 px.
- `og replay compare`: the goldens `game-3d` (templates/game) and
  `game-2d` were the same over 300 ticks. Two more traces were recorded
  on base and replayed on the branch, also the same over 300 ticks:
  game-2d with its look line, and game-3d with its look line and the
  ArtCamera preset above.

## Shared files touched

- `crates/og-api/api.ogt` and its generated outputs:
  `crates/og-api/src/generated/host.rs`,
  `packages/engine/types/index.d.ts`,
  `packages/engine/types/bindings.json`,
  `packages/engine/schema/api.schema.json` and
  `packages/knowledge/wiki/*.md` (line numbers only, apart from render
  and tilemap).
- `CHANGELOG.md` and `packages/engine/changelog.json`.
- `crates/og-render/shaders/sprite.wgsl`, `src/lib.rs`, `src/scene.rs`,
  `src/sprites.rs` and `README.md`.
- `crates/og-runtime/src/engine.rs`, `extract.rs`, `fx.rs`, `feel.rs`,
  `assets.rs`, `look.rs`, `modules.rs`, `anim.rs` and `lib.rs` (new:
  `art.rs`).
- `crates/og-anim/src/system.rs`, `src/lib.rs` and `tests/playback.rs`.
- `crates/og-tests/tests/sprites2d.rs`.

## For the coordinator

- No reference frame is named and no `metal` file is needed. The new
  og-tests checks are pixel assertions on rendered output, not
  comparisons with a committed frame. No existing reference should move:
  games without a look line draw identically.
- `world_hash` (the printed run hash) moves for every game, as A1's did,
  because `og.Sprite` and `og.Tilemap` gained fields. The replays'
  portable hash does not move. `unreleased.simulation` stays
  "unchanged".
- `ArtMotion.stepFps` is the one default that reaches the tick: a pose is
  tick state. It only applies to a game that names a look setting
  `stepFps`, and no golden replay does. A game that adds such a look
  changes its animation on purpose. Of the catalogue presets, only
  rubber-hose-animation and sculpted-clay set `ArtMotion.stepFps`.
- The release's breaking entry now also names A4's defaults, since a
  game with a look line changes frames (cleanEdge, the contract line).
