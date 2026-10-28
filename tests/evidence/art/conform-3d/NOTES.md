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

## Follow-up `art/conform-3d-2` (from c52344b2)

- **project.ogt's `conform { off ... }` is honoured by the 3D pass.** The
  cook reads `ProjectConform` (A2's) before the 2D pass and gives
  `Conform.off` every model and flipbook kept as made: its own
  `conform off`, or the first project pattern that matches its id. A
  model under an off pattern keeps its geometry, its `#tex<n>` textures
  and its clips (the control's bytes). A flipbook under one keeps its
  source durations. The asset's stamp drops the contract, as for its own
  `conform off`.
- **The mesh pass reports in `Report.conform`.** Per asset, one
  `ConformEntry`: `Changed` with its outputs' lines joined by "; "
  (model, textures, clips), `Unchanged` ("meets the contract as it is"),
  or `Off` with why (`conform off (assets/<id>.ogt)`, or `project.ogt's
  conform block: off "<pattern>"`). Assets A2 already lists as Off are not
  listed twice. `og cook --report` and its JSON read the one list. No
  `conform:` note is left in an asset's notes; each asset still gets
  A2's `art contract: <action>: <what>` note from the entry. Frames
  dropped past ArtMotion.maxFrames are also in `Report.warnings`
  (`assets/<id>: warning: ...`).
- **`cooked/.conform.ogt`** (type `conform-log` v1) keeps each conformed
  output's lines by id and stamp, so an up-to-date cook lists the same
  entries and warnings. It is written only under a contract and is
  removed when empty, so a project with no look, or with zero art
  blocks, still cooks byte-identically. `og build` (web-build.ts) leaves
  it out of the export, beside `.cook-times.ogt`.
- **Tests** (`tests/conform_mesh.rs`):
  `project_conform_off_keeps_models_their_textures_clips_and_flipbooks_as_made`
  (a folder pattern and an id pattern: model and walker bytes equal the
  no-look control and differ when on, `#tex0` 128 rather than 32, coin
  durations 0.125 rather than 1/6, three Off entries naming their
  pattern). It fails when the project block is ignored.
  `the_report_lists_the_mesh_pass_beside_the_texture_pass_and_keeps_it_when_up_to_date`
  (Changed and Unchanged entries, no `conform:` notes, the warning,
  markdown and JSON, and an up-to-date second cook with the same entries
  and warnings). The six earlier tests now read the entries.
- **Shared edits:** `cook/mod.rs` only (the project block read, the log,
  the entries before A2's note loop, and the `Work::Model` off note
  removed), plus one line in `packages/engine/src/node/web-build.ts`.
  `conform/mod.rs` is untouched.

## Follow-up `art/conform-3d-3` (from f4005931): textures no model claims

**The finding.** A6's check on game-3d under faceted-low-poly found
`textures/stone.png` cooked at 512², linear, 10 mips. Nothing in the
template names it: the scene has no material, and `main.ts` builds
`stone` as a colour-only `standardMaterial`. Scenes and `type model`
cannot name a texture (lit-material maps are TypeScript handles), and the
cook does not read game code. The coordinator chose option A: in a 3D
project, a colour texture nothing else claims is a mesh's.

- **One classification, `og_cook::cook::conform::texture_roles(root)`**
  (`TextureRole`: Sprite, Sheet, Fixed, Mesh, Data). It reads the project
  with the cook's own scan (`scan`, factored out of `cook_project` with
  no change in behaviour) and A2's claim code (`texture2d::uses`, now a
  function). It decides with `texture2d::texture_role`, which the 2D pass
  calls too. The order: the image's own `conform ui`/`conform sprite`;
  who names it (a style, rig or map's tile set: Fixed; a flipbook or tile
  set: Sheet); normal or linear: Data; a 2D project or `kind pixel`:
  Sprite; else Mesh. `og style check`'s `measure()` uses it (the one edit
  to A6's file), so the check judges a texture as the pass that conformed
  it. A `.ktx2` has no role and keeps A6's old rule.
- **The mesh pass holds Mesh textures**, via `Conform.textures` and
  `Work::Texture`: `maxSize` (the extent is unknown, so density is not
  applied), `filter`, `mips`, the palette where `strict`. Both opt-outs
  apply (an own `conform off` in the import settings, project.ogt's
  patterns) and are listed as Off. The entry begins "conformed as a mesh
  texture (no model claims it; a HUD or UI image takes `conform ui` in
  its import settings)". `mesh::texture` now takes the extent as
  `Option<f32>`.
- **maxColors on lossless mesh textures.** `filter nearest` cooks a
  texture as lossless RGBA8, where the check judges its colours. Such a
  texture (a model's `#tex<n>` too) now holds to `ArtPalette.maxColors`
  with the same median cut that holds baked vertex colours, unless a
  strict palette already mapped it. The texture desc is now
  `conform-texture-v2`, adding `maxColors`. Without this, stone.png
  measured 499 colours against 32.
- **og-build §8:** the sentence on default mesh textures and `conform
  ui`. The stale "notes beginning conform:" paragraph now describes the
  `conform` entries (follow-up 2). The check's table rows say how a
  texture is classified.

Tests (`tests/conform_mesh.rs`):
`a_texture_no_model_claims_is_a_mesh_texture_in_3d_unless_its_own_line_says_otherwise`.

| Case | Result |
|---|---|
| Unclaimed in a 3D project | 256² becomes 64², nearest, 1 level; role Mesh; a Changed entry with the default text and palette mapping |
| A 4,096-colour gradient under `filter nearest` and `maxColors 8` | 2 to 8 colours, and "4096 → 8 colours" |
| `conform ui` | Stays 256² and is A2's: role Fixed, the entry says "conform ui", with no mesh text |
| `conform off` | Bytes equal a no-look control; Off entry |
| project.ogt's `off "textures/"` | Stays 256²; Off entry naming the pattern |
| A 2D project | Role Sprite; no mesh-pass note or entry |

The byte-identity test gains an unclaimed png.

Verification:

- og-cook nextest 219/219.
- `cargo clippy -p og-cook -p og-cli --all-targets -D warnings` clean.
- og-cli `style_check` unit 7/7 and CLI 2/2.
- Knowledge `validate.test.ts` 25/25.
- `og style check` with the branch's dev `og`, on template copies with a
  `look` line:
  - **game-3d with faceted-low-poly: exit 0**, 4 assets, 0 off.
    `textures/stone.png` is ok: 32 colours against 32, 64x64, nearest,
    1 level. The play frame at 2 s (`--play 2:screenshot`, 640×360,
    lavapipe, `OG_ENGINE_PACKAGE` and the dev addon) is the title card
    over the flat-shaded dropship and walkers, with A6's numbers
    unchanged: 2,738 colours, 742, edges 3.4 %, 8.5 % hard, contrast 0.35.
    stone.png is not drawn, so no frame moves.
  - **game-2d with 16bit-pixel: exit 0**, the same 5 sprites ok as A6
    measured.

No shipped project names a look, so no reference frame moves.
