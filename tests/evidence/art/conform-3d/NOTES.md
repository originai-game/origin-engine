# A3 `art/conform-3d`: models, their textures and motion held to the art contract

Stream A3 of docs/art.md (the 3D and animation rows of §2.3, the A3 row of
§3), branch `art/conform-3d` from `554df51e` (origin/main, with A1). The
cook reads project.ogt's look through A1's `cook::look::contract` and
conforms every model (glTF, `type model`, `.vox`), the textures it uses,
its clips (`#anim`) and every flipbook. Sources are never written.

## What landed

| Commit | What |
|---|---|
| `d7b9de23` | `crates/og-cook/src/cook/conform/{mod,mesh}.rs`: `Conform::of` reads ArtShape, ArtTexels, ArtPalette, ArtLine, ArtMotion; `None` when none of them constrains anything here, so nothing changes. Per primitive, before welding (`model::cook_primitive`, so the model blob and the `#anim` agree vertex for vertex): voxelise (the `.vox` greedy mesher, colours on the palette or median-cut to `maxColors`, the edge doubled while past the budget), decimate to `propFaces` or `characterFaces` (a model with a skin) counting every node that draws the mesh (meshopt simplify, then by position, then sloppy, then the largest triangles), bevel the hard edges (inset faces, a quad on each hard edge, a fan at each corner), bake vertex colours where the style is untextured, and set flat, smooth or angle normals. With `ArtLine.width` the model blob gains an `outline` section (header plus a hull normal per cooked vertex; layout in the module docs). Textures: area resample to `density` over the model's longest side (to a power of two) and `maxSize`, strict palette mapping (Oklab nearest), `filter nearest` cooks colour as pixel art, `mips none` drops mips. Clips resample at `stepFps` as STEP channels; past `maxFrames` the rate is lowered, with a warning. Flipbooks: durations held to whole steps; past `maxFrames` every k-th frame is kept, its dropped neighbours' time added to it, with a warning. `conform off` in an asset's `.ogt` or its import settings opts it out. Each conformed output's stamp carries the contract's description, so cache keys follow the look's values. |
| `2fd618a1` | A model's colour texture cooked as pixel art is named `kind pixel` in the model and sampled nearest by `CookedModel::add_to_with`. |
| this commit | A flat style bakes only with no ArtTexels constraint. faceted-low-poly's `maxSize 64 filter nearest` keeps its textures. Voxels sit on one grid in model space, so turned parts step. The diorama project, these notes, the frames. |

## Shared edits (for the merge with A2)

- `cook/mod.rs`: `pub mod conform;` (A2 will add the same line if it
  creates `conform/mod.rs`: keep one). `MeshOptions` gains `conform:
  Option<conform::mesh::Conform>`. One block after the contract is read
  turns it into `options.mesh.conform` (the palette read from `@palettes`
  or assets/). `conform_off`/`mesh_for` helpers. The model, proxy,
  `#anim`, `#tex<n>` and flipbook stamps take the contract's description
  only when it is non-empty. `run` passes the conformed options to model,
  proxy, animation, glTF-texture and flipbook work.
- `conform/mod.rs`: module docs plus `pub mod mesh;`. A2 adds `pub mod texture2d;`.
- `cook/model.rs`: `FVertex`'s fields `pub(super)`; the hook in
  `cook_primitive`; the baked materials, the `outline` section and notes in
  `cook`; the PIXEL kind of the textures in `cook`.
- `cook/anim.rs` (rate and STEP), `cook/flipbook.rs` (`cook` takes the
  motion; `conform` dropped from the sheet text), `cook/vox.rs` (`Quad`,
  `mesh_model` `pub(super)`), `cook/model_text.rs` (`conform off` accepted),
  `runtime.rs` (nearest for PIXEL model textures).

## Opt-outs and the report

Every conformed asset gets a note starting with `conform:` (the triangles
before and after, the budget and whether it decimated, normals, voxels,
bevel, the bake's distinct colours, the outline, a texture's resize and
filter). Warnings start with `conform: warning:` (frames dropped past
`maxFrames`). `conform off` notes `conform: off (`conform off`): kept as
made`. `og cook --report out.md` lists these per asset (`- `path`: note`).
Notes appear on the cook that makes the output, as every cook note does.
An up-to-date output carries none: this was already the report's
behaviour, and A6's `og style check` should recompute rather than read it.
`conform` blocks in project.ogt naming paths are not done: per-asset
`conform off` only.

## Verification

- `cargo clippy -p og-cook --tests -- -D warnings`, `cargo fmt`: clean.
- `cargo nextest run -p og-cook`: 192 of 192 passed at the head.
- `tests/conform_mesh.rs` (expected values derived in the test):
  byte-identical `cooked/` with no look versus a look of frame blocks
  only, and versus a look whose art blocks are all zero (every file but
  the wall-clock `.cook-times.ogt`); a 2,208-triangle sphere under
  `propFaces 500` drawn by 1 and by 2 nodes is ≤ 500 drawn and > 250; the
  share of corners on their face normal goes from under 0.1 (source) to
  1.0 under `normals flat`, with one colour per face and ≤ 4 colours;
  voxel corners on the 0.1 m grid within 1e-4, axis-aligned normals, the
  voxel count between the cells whose centre is inside the ball and the
  cells touching it, every colour a dusk16 colour; textures at the
  density (5 m × 16 → 64) and `maxSize 32` with `filter nearest` as pixel
  art; the walker's clips at 12 Hz with `ceil(d·12)+1` keys, every channel
  STEP, ≤ 5 frames under `maxFrames 5` with the warning; `conform off`
  gives the same model bytes as a project without a look; two cooks of a
  conformed project are identical.
- Unit tests in `conform/mesh.rs`: a cube's bevel (12 hard edges, 32
  triangles added, every corner 0.1 in from two faces, all faces
  outward), the median cut, the flipbook step and drop.
- Not run (as briefed): scripts/check.sh, test lanes, og test,
  web-reference, wasm and release builds.

## Frames read (lavapipe, 640×360, `og run --headless --commands`)

The fx-showcase looks diorama builds its tree, crate and spheres from
runtime geometry (`og.render.boxGeometry`, `sphereGeometry`), which the
cook never sees. So the station itself cannot show conformance.
`diorama/make.py <out> <slug>` writes a project with them as model
assets (the tree and five spheres `type model`, the crate a glb with
`crate.png`), placed and lit as `looks.ts` does, with `look
"looks/main.ogt"` extending the preset. Run with the base (`554df51e`)
and the branch's dev `og` and addon (`OG_NATIVE_ADDON`, the file named
`.node`, or bun reads it as a script). Montage: `frames-lavapipe.jpg`
(left base, right conformed).

| Look | Pixels differing | Read |
|---|---|---|
| faceted-low-poly | 17,432 (7.6 %) | The spheres at 500 triangles each (from 1,024) show irregular decimated facets on the gold and chrome, flat-shaded; the crate keeps its planks at 64 texels, nearest. The tree, already 48 triangles, is unchanged. The preset's frame part already flat-shades, so the base reads faceted too. The difference is the triangle count, and 500 per prop still reads round at this distance: the preset's budget, not the conformer, decides how faceted it looks. |
| voxel-worlds | 38,602 (16.8 %) | Every sphere is built of 0.1 m cubes (1,846 each, 828 triangles), single-colour, clearly blocky with stepped silhouettes. The crate is a voxel block of 43 colours, which read as one brown at this size. The tree's middle tier, turned 45°, is a staircase. The first and third tiers, at 0° and 90°, sit on the grid. Reads as voxel worlds; the base does not. |
| paper-craft | 2,519 (1.1 %) | Flat normals and baked colours: the crate becomes one brown card (its texture's mean, the style untextured), the spheres keep their 1,024 flat facets (under the 2,000 budget). The preset's frame part already flat-shades and outlines, so only the crate changes visibly. |
| sculpted-clay (extra, bevel) | — | The tree's 48 and the crate's 12 hard edges chamfered at 2 cm (+128, +32 triangles), smooth normals: a thin soft highlight along the leaf boxes' edges. |

## For the coordinator

- No reference frame moves. The fx-showcase diorama and every template
  draw runtime geometry or have no look line.
  `without_a_look_or_with_zero_art_blocks_the_cook_is_byte_identical`
  holds the rest.
- A game whose project.ogt names a look extending a preset with ArtShape
  now has its models conformed: the 0.7.0 breaking entry (docs/art.md §5).
- Faceted low poly's 500-face prop budget barely facets a sphere: A7's
  sample scene may want a lower preset value (A1's file, coordinator's
  call).
- The `outline` section is data only. Nothing reads it yet (A4's runtime
  mesh outline).
