# Art wave A2: conform 2D (`art/conform-2d`)

Stream A2 of docs/art.md §3: the 2D rows of §2.3. With a project's art
contract (`cook::look::contract`: the look named by project.ogt's `look`
line, flattened), the cook conforms the textures of sprites, flipbooks,
tile sets, 2D rigs and UI images. Base: `554df51e` (A1 merged).

## Commits

| Commit | What |
|---|---|
| `9cdad4c4` | og-api: `ArtLine.brush`, new enum `ArtBrush` (diamond, circle, square), appended to the art module; generated files regenerated, `og api check` matches |
| `36abaca9` | og-cook: the conform pass (`cook/conform/`), its tests (`tests/conform_2d.rs`), third_party notices |
| `b6f86817` | the cook report: `Report.conform` in Markdown, `--json`, `cook-report.json` and `og cook --report` |
| `8b8cd430` | og-cook README: §Art conformance, the index texture's format |
| `c6998292` | CHANGELOG.md and changelog.json (`og.ArtLine.brush`, `og.ArtBrush`, the conformance) |
| `1bb65d66` | og-render: the texture filter sync runs before the frame's sprite groups are made (a crash the conformance exposed, below) |
| `a4a0a711` | conform: a tile's edge does not take the inside line (found reading the frame) |

## What it does

Full description: `crates/og-cook/README.md` §Art conformance. In brief:

- **Which textures**: those a flipbook or tile set names (sheets, cell by
  cell); those a 2D rig, a UI style's `atlas`/`icons` or a map's embedded
  tileset names (fixed layout: palette and 1-bit alpha only); `kind pixel`
  textures, and every PNG/JPEG of a `profile "2d"` project. An image's import
  settings can say `conform sprite|ui|off` and `conform { size N }`. Normal
  and linear textures (data) and `.ktx2` are skipped.
- **Pixel grid** (when `ArtPixels` sets `unit`, `logicalHeight` or
  `spriteSizes`): the cell size is the smallest candidate that redraws the
  cell within 0.02 mean Oklab distance of the largest candidate's
  downscale. Never upscaled. Downscale is a dominant-colour box filter, never
  bilinear; alpha is 1-bit.
- **Palette**: Oklab nearest colour (fixed point), the palette's dither
  (bayer2/4/8, blue noise, IGN, anchored at each cell's corner), `strict`,
  `maxColors`, `rampSteps`, `accents` kept for tiles; without `colors`, the
  asset's own palette (median cut in Oklab).
- **Line**: Pixelorama's OutlineInline ported (diamond, circle, square;
  outside, inside, both), drawn into each cell, never across cells; a
  tile's edge is not treated as clear (tiles continue into each other).
- **Index texture** `<id>#index` (for A4): same size and layout, `kind
  pixel`; RGB the conformed colour, **alpha = palette index + 1** (0 clear);
  palette `ArtPalette.colors` in order or `<id>#palette`.
- **Prerotated**: `<id>#rotations` (one row per cell, N columns, cleanEdge
  port, never blends) and `<id>#rotations-index`.
- **Painted 2D**: Stylise's paint fields (Kuwahara, anisotropy, bleed, edge
  darkening, canvas grain) in texel space, only when the look also sets an
  art field.
- **Sources are never written**: the pass rewrites in-memory sources
  (`Source.conformed`, a sheet's text, extra outputs as sources); stamps
  carry the contract's hash (look text, palette and canvas bytes, pass
  version), so the cache and determinism follow the look.
- **Opt-outs**: `conform off` per asset (import settings, or a flipbook's,
  tile set's, rig's own `.ogt`), and project.ogt's `conform { off "glob" ... }`.
  Every change, every asset left as it was and every opt-out is in
  `og cook --report`, the Markdown report, `--json` and `cook-report.json`.
- **Off switch**: nothing runs unless the contract sets an art field that
  is not zero; then `cooked/` is byte-identical to before (tested).

## Shared files touched (for merging beside A3)

- `crates/og-cook/src/cook/mod.rs`:
  - `mod conform; pub use conform::{ConformAction, ConformEntry};`
  - the field `Source.conformed: Option<Arc<Vec<u8>>>` and its `None` init
  - **the hook**: one call
    `conform::texture2d::conform(root, &assets_dir, &mut sources, &known, contract.as_ref())`,
    after the contract is read, before `plan_groups`. A3's
    `conform::mesh::...` goes beside it.
  - `Work::Texture` decodes `s.conformed` when set
  - `Report.conform` and its notes, Markdown table, JSON
- `crates/og-cook/src/cook/conform/mod.rs` (new; `pub mod texture2d;`, the
  shared `ConformEntry`/`ConformAction`, the `conform` block's
  `ProjectConform`, the per-asset `AssetConform`, `strip`). A3 adds
  `pub mod mesh;` and can reuse the report types.
- `crates/og-cook/src/cook/look.rs`: `flattened` (the flattened look text
  for the stamp's hash); a test helper gains `conformed: None`.
- `crates/og-cook/src/cook/flipbook.rs` (`texture_bytes` reads the
  conformed image), `crates/og-cook/src/cook/stats.rs` (`conform` in
  `cook-report.json`).
- `crates/og-cli/src/cook.rs` (`conform_lines` under `--report`),
  `crates/og-cli/src/trace.rs` (test fixture `conform: vec![]`).
- `crates/og-api/api.ogt` (append only) and its generated files.
- `crates/og-render/src/lib.rs` (the filter sync moved, below).
- `crates/og-cook/README.md`, `CHANGELOG.md`, `packages/engine/changelog.json`.

## Verification

- `cargo test -p og-cook`: 91 lib + 113 integration tests passed,
  including the conform unit tests (outline, palette, resample, cleanEdge,
  paint, colour, the `conform` block) and `tests/conform_2d.rs`:
  - `a_sprite_drawn_at_2x_comes_back_to_its_grid_on_the_palette_with_its_line`
  - `the_downscale_takes_each_boxs_dominant_colour_never_a_blend`
  - `a_flipbook_and_a_tile_set_follow_their_texture_to_its_grid`
  - `prerotated_sprites_bake_their_turns`
  - `opt_outs_keep_their_texels_and_are_listed`
  - `without_a_contract_the_cook_is_byte_identical` (no look; and a look
    whose art blocks are all zero: every file of `cooked/` equal)

  The expected texels are written out by hand in the fixtures, with
  asymmetric shapes, not computed by the code. Mutation checks: a mean
  instead of the dominant colour fails the downscale test; counting a
  tile's edge as clear fails `an_inline_counts_the_cells_edge_as_outside`.
- `cargo test -p og-render --lib`: 89 passed. `cargo test -p og-cli --bin og
  trace`: passed.
- `cargo clippy -p og-cook -p og-render -p og-cli --tests -- -D warnings`,
  `cargo fmt --check`: clean. `og api check`: matches.
- Not run, per the stream rules: scripts/check.sh, test lanes, `og test`,
  web-reference, wasm or release builds.

## Frames read (lavapipe, 640×360, `og run --headless --commands`)

These are copies of `templates/game-2d`, run with a dev addon from this
branch (`OG_NATIVE_ADDON`) at 3.5 s. `frames-lavapipe.jpg` is the full
frames: top row is the base (no `look` line) and then 16bit-pixel
conformed; bottom row is monochrome-pixel without and then with the
`look` line. `sprites-3x.png` is the same crops at 3× nearest-neighbour.

- **16bit-pixel conformed** (the template's look, `extends "16bit-pixel"`,
  with `look "looks/origin-game-2d.ogt"` added to project.ogt). The
  report: courier's 32×40 frames go to 26×32 by the dominant-colour box
  (270×82 → 222×66), with a warning that 26×32 is not in spriteSizes
  16/32/48. Courier, coin and city tiles keep their own palettes (15, 4 and
  18 colours, within maxColors 64 and rampSteps 6). Each gets an `#index`
  and a `#palette`. The rig and the UI atlas keep their layout.
  In the frame, the courier (the small grey-haired figure at the left)
  now has a 1-texel black inside outline round head, cap, legs and the
  orange scarf. Its texels are visibly coarser, with no blended
  edge pixels. The coin has a dark rim. The rooftops' silhouettes carry the
  line at their crenellations, and the full wall tiles have none (164
  line texels in the tile set). The keeper (the red-caped rig) and the HUD
  are as before. The world hash `a457aaa3730bc835` matches the base, so the
  simulation is unchanged.
- **monochrome-pixel conformed** (`extends "monochrome-pixel"`:
  `@palettes/handheld4` strict, maxColors 3, bayer8). The report:
  courier's 32×40 frames go to 19×24 (warning: not in 12/16/24), and every
  2D texture maps to 3 colours of handheld4 with an `#index`. In the frame,
  without the line the frame's own treatment already turns everything
  green. With it, the courier reads as a chunkier 19×24 figure in the
  three light greens, with flat, hard-edged shading instead of the frame
  dither breaking up its original colours. The ledge tiles carry the Bayer
  pattern in their texels, and the keeper's shading is flatter, in 3 tones.
  8% of the pixels differ between the two
  (`magick compare -metric AE`: 18540). The world hash `31538d17ec791d5c` is
  the same with and without the line.

## Found and fixed on the way

- **og-render crash** (`1bb65d66`): the overlay's sprite bind groups were
  made before the per-frame texture filter sync. The sync then dropped
  any group whose texture had just turned `Nearest`, and the overlay pass
  panicked with `made before the pass`. This happens on the first frame a
  UI texture cooked as `kind pixel` draws; the conformed UI atlas is the
  first such texture in the templates. The sync now runs before any group
  is made. No regression test was added: it needs a GPU frame with an
  overlay sprite on a nearest texture, which is a gfx-lane test. I would
  suggest one in og-tests for the coordinator's lane.
- **Tile seams** (`a4a0a711`): the inside line treated a tile's cell edge
  as clear, which drew a grid on every wall. That is fixed, with a unit
  test.

## For the coordinator

- **Behaviour change**: a game whose project.ogt names a look extending a
  preset with art blocks now cooks its 2D art conformed. Scaffolds made by
  A1's `create` write that line, but existing games without the line are
  unaffected. This belongs in the release's breaking entry; the changelog
  says so.
- Reference frames: a template or example whose project.ogt has a `look`
  line and draws 2D art moves. That includes templates/game-2d scaffolds,
  but not the template itself, which has no `look` line in project.ogt.
  Re-record in the functional lane if one does.
- `@styles/pixel/icons.png` is linear (an SDF), so it is skipped: the HUD
  hearts stay red in the monochrome frame. Tinting those to the palette is
  a runtime default (A4).
- Courier frames land at sizes outside `spriteSizes` (warned, not
  forced). Forcing a size, by padding or cropping to the nearest
  spriteSize, is a policy choice I left out; `conform { size N }` sets
  one per asset.
- The index texture's format is the one A4 reads: README §Art conformance.
