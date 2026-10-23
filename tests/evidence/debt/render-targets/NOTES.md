# Debt `render-targets`: evidence

Branch `debt/render-targets`, cut from main 586019b8. It pays off the
"Left out and limits" of depth S8 (`tests/evidence/depth/render-targets/NOTES.md`).
All frames were drawn on lavapipe (tier `vulkan-cpu`, llvmpipe LLVM 15,
debug build) and read one by one; they are copied into this directory.
Following AGENTS.md, nothing here ran the gate, `og test`, web-reference or
a wasm or release build: the checks were `cargo check`/`clippy`, the unit
and integration tests of the edited crates (`og-render --lib`,
`og-render --test webgl2`, `og-probe`, `og-runtime --lib`, `og-tests
--test targets`, `og-tests --test culling`), `og api gen`/`--wiki`/`check`,
the knowledge package's tests (42 pass), and `og run --commands` for the
fx-showcase stations.

## What landed, one commit per item

### 1. A view draws what the main pass draws on its layers (29d90281)

An og.CameraView or og.Mirror now draws, from its own eye and filtered by
its `layers`:
- opaque meshes and models, skinned meshes, terrain and sky (as before);
- transparents, sorted back to front from the view's eye (`view_blended`),
  including transparent skinned meshes;
- particles, with a per-view particle buffer and bind group; the new
  `og.ParticleSystem.layers` field masks them;
- lines and points (`og.Lines`/`og.Points` `layers`, a per-view line
  buffer);
- world sprites and world text (`og.Sprite.layers`), built per view by
  `sprites::build` with the mask and appended after the main frame's
  instances. Screen sprites stay on the main frame.

The cost is explicit: a view draws only what its mask admits. The new
RenderScene fields `sprite_layers`, `line_layers`, `point_layers` and
`particle_layers` carry the masks; og-runtime fills them, and no longer
drops off-layer sprites, lines and particles at extraction (feel.rs,
geometry.rs, fx.rs).

What a target carries, now in design §2.6 and the og-game skill §23:
- An `ldr` target or a viewport view carries exposure and the tone curve,
  so its texture can be shown on any material.
- An `hdr` target carries linear radiance before exposure.
- Bloom, AO, SSR, depth of field, grading, LUTs and custom posts stay on
  the main frame.

The tests:
- A new gfx scene, `views-layers` (`target_scenes::view_kinds_scene`):
  layer 1 holds a glass box, a sprite, a particle system and a line loop.
  The monitor's camera sees layers 0 and 1; the main camera sees layer 0.
- `a_view_draws_transparents_sprites_particles_and_lines_on_its_layers`
  (`og-tests/tests/targets.rs`) renders the scene once per kind with and
  without the layer-1 object. It asserts that pixels moved on the monitor
  and nowhere else. The moved-pixel counts, all on the monitor and 0
  elsewhere: transparent 383, sprite 400, particles 189, lines 678.
- On the old code each count is 0 on the monitor, so the test fails for a
  view missing any one kind.

Frame read, `views-layers.png`:
- The monitor on the back wall shows the room from its camera: the cyan
  glass box, the pink sprite, the green particles and the yellow line loop
  around the yellow box.
- The main frame around it shows none of the four.

### 2. The main camera keeps GPU culling in a frame with views (57e2c1dc)

Before, any view switched the whole frame to CPU culling. Now:
- The main camera stays on the GPU path.
- The views (and reflection-probe faces) cull on the CPU, and their lists
  append the instances they draw after the GPU's (`cull(.., whole)`, with
  offsets rebased in `sorted`). This also fixes a latent subtraction
  overflow in probe-face lists under GPU culling.
- The GPU cull passes test the layer mask (`BatchGpu.layers`,
  `PassGpu.info.w`; shadow passes pass every layer), so a GPU-culled main
  camera keeps its layers logic.

The test: `culling.rs` now covers `views-monitor`, `views-mirror`,
`views-split` and `probe-interior`.
- Its assertion "Culling::Gpu culls on the GPU on every frame exactly on
  the full tier" fails on the old code for the three views scenes.
- With the change: 2 passed, every scene's GPU and CPU frames at rmse 0.
- Draw counts, GPU against CPU: views-monitor 57 / 37, views-mirror 51 /
  35, views-split 54 / 39.
- The GPU path issues more draws because the indirect draws are per batch
  per pass. Its triangles are fewer than or equal to the CPU path's, as the
  test asserts.

Numbers, from a scratch test (deleted): spheres-20k plus N monitor views,
Culling::Auto, lavapipe, debug build, mean of frames 10 to 30. The CPU
time is `render()`'s wall time; the frame is the whole frame with the GPU
wait, which on lavapipe is the CPU rasteriser.

| Views | GPU culling | Draws | Triangles | `render()` CPU | Frame with GPU |
|---|---|---|---|---|---|
| 0 | yes | 12 | 7,306,754 | 4.18 to 4.34 ms | 2170 to 2214 ms |
| 1, before | no | 14 | 7,680,004 | 11.02 ms | 3154 ms |
| 1, after | yes | 22 | 7,306,756 | 6.78 ms | 3235 ms |
| 2, before | no | 23 | 7,680,006 | 12.54 ms | 3990 ms |
| 2, after | yes | 34 | 7,306,758 | 9.57 ms | 4294 ms |

- The CPU cost of a frame with views falls by 38 % (one view) and 24 %
  (two).
- The main camera draws 373,248 fewer triangles, because its occlusion
  test comes back.
- On lavapipe the GPU frame is a few percent slower: the GPU cull's
  compute passes run on the CPU rasteriser, and the views' instances
  append to the instance buffer. Real GPUs need measuring, at the wave's
  end.
- The pass timings (`pass_timings`, what ProbeTimings reports per pass)
  were view-0 0.05 ms and main 0.015 ms of recorded GPU time.
- These are not ProbeTimings from an `og run` of a game. No showcase
  station has 20,000 instances, so Culling::Auto keeps every station on
  the CPU path.

### 3. A render target's depth texture on every tier, WebGL2 included (81c36401)

The implementation, not a refusal:
- A target with `depth` runs its own depth prepass: an RG32F linear-depth
  colour attachment, the gbuffer, outline and depth, through the
  `prepass_pipeline_of` pipelines.
- `views_depth.wgsl` loads that colour texture, with an explicit
  non-filterable layout, so WebGL2, which cannot sample a depth texture
  for this, reads it as colour.
- `resolve_depth_source()` takes no MSAA argument and is listed for the GL
  tier in `shaders.rs`.
- `RenderCapabilities.renderTargetDepth` is true whenever a renderer
  exists; api.ogt, design §2.6 and the skill say so.
- No warning path is needed: no tier refuses depth.

The tests:
- A new gfx scene, `views-depth` (`target_scenes::depth_scene`).
- `a_targets_depth_texture_holds_the_views_linear_depth` checks the box,
  the wall, the sky and an off-layer box against sRGB-encoded linear depth,
  within 3.
- `og-render/tests/webgl2.rs` `a_targets_depth_resolve_is_the_gl_tiers_and_translates`
  translates the GL tier's resolve through naga to GLSL ES. The old resolve
  read a depth texture and was not on the GL tier's list, so the test fails
  on the old code.
- The orb has no GL adapter, so WebGL2 is checked through translation only.
  The browser tier's run is the coordinator's web-reference.

Frame read, `views-depth.png` (the depth texture shown full screen),
region means:

| Region | Mean | Reading |
|---|---|---|
| Sky, 600×100 at (300,40) | 255 | the far plane |
| Wall, 300×100 at (100,300) | 147 | |
| Box, 200×200 at (540,260) | 97 | darker: nearer |
| Floor's bottom row, 1280×20 at (0,690) | 113 | the gradient runs to nearer at the bottom |

The off-layer box is absent.

### 4. PostEffects: eight custom post slots, not a list (fe5148fd)

Decision: no list field in records now. Instead:
- `PostEffects.custom1..custom8` replace `custom1..4`.
- `RenderCapabilities.maxCustomPosts` is og-render's `MAX_CUSTOM_POSTS`
  (8) on every tier, where it was min(4, 8).
- og-runtime's `POST_SLOTS` is `MAX_CUSTOM_POSTS`, so the two cannot drift.

Why not `custom: Post[]`. The obstacle is not the IDL. A record field is a
reflected column, and the reflect module states the rule: "A field holds
one value: there is no array or list column". It names numbered fields
(`p0` to `p7`, og.CameraRail) as the pattern. That schema
(`og_core::reflect::FieldType`, Copy, with no list variant) is the one every
reader shares. A list column needs every one of them to change:
- og-core `FieldType`: a fixed-capacity array variant such as
  `Array(elem, n)` or `List(n)` of a scalar kind. It needs a JSON name, a
  layout hash, a byte size and alignment, and a count lane so `[]` and
  `[0]` differ.
- og-ecs: snapshot, restore and migration by field (a capacity change
  keeps what fits, as `str(N)` does).
- og-net `state.rs`: field replication.
- og-probe `value.rs`: `get:`/`set:`, `patch` and dumps take a JSON array.
- og-cook scene text and looks: a list literal on one line.
- og-runtime `codec.rs`: `Kind` and the JSON↔bytes encoding.
  `look.rs` loads a handle per element.
- The TS runtime (`packages/engine/src/runtime.ts`): a resource accessor
  that answers an array copy and writes a whole array (`post.custom.push`
  would silently do nothing on a copy). That needs a documented decision.
- og-idl: validate (`List.X` as a column for records only), and gen ts,
  tables, wiki and host (`FieldDecl` gains an element kind and a capacity).
- The inspector and Tune: a list is never tunable.

The precedent is `str(N)` (294888a2), which touched some twenty files across
og-core, og-ecs, og-cook, og-runtime, the TS runtime and og-idl. That is
a cross-stream change to the schema, which a parallel debt stream should
not land beside other streams. The numbered slots give every pass
og-render runs now; a list column would be its own stream, owned by
og-core.

The test: the fx-showcase shaders station chains a warm grade
(`assets/post/tint.ogt`, new) in `custom5` and the CRT in `custom8`, with
slots 1 to 4, 6 and 7 left 0. `tests/shaders.og.ts` adds a check of the
grade's warmth on the ground behind the panel and keeps the CRT's black
corners. On the old code there is no `custom5` or `custom8`, so neither
pass runs and both checks fail.

Frame read, `station-shaders.png`:
- The station is warm and amber, inside the CRT's curved, scanlined screen
  with black corners.
- The ground region, 200×100 at (900,280), has mean RGB 170 150 110 (red
  over blue by 60).
- With the grade's colour set to 1 1 1 the same region was 155 154 147
  (by 8). The check's threshold is 35.
- The sky region, 300×80 at (400,120), goes from 206 212 221 to
  215 209 196.

### 5. Describe names the views each entity shows in (d66b3296)

Decision: `describe` keeps reporting main-camera visibility, and names the
views. A camera parameter would make the answer depend on a choice the
reader must know to make, and `expect`/`pick` stay on the main camera.
- Every entry gains `views`: the ids of the og.CameraView and og.Mirror
  entities whose own id pass also shows it.
- `Description.views` lists each view: its id, name, kind (`camera` or
  `mirror`), image size, and the entities its own pass sees, with boxes
  in the image's pixels.
- An entity on a layer only a view admits is absent from the main list
  and found under that view.

Where it lives:
- og-render `id_pass.rs`: `IdCamera`, `render_ids_from` and
  `view_id_cameras`. The id pass draws from any eye on its layers, at a
  quarter of the view's pixels.
- og-runtime `describe.rs` runs it per view; og-probe `hooks.rs` has
  `Seen.views` and `SeenView`.
- api.ogt has `DescribedEntity.views`, `DescribedView` and
  `Description.views`, and design §3.4 and skill §23 have the contract.
  The brief said §3.3, but that section is the command channel; the probe
  contract is §3.4.

A fix found on the way: the mirror's near plane sat exactly on the glass.
The CPU id pass kept the glass quad on the near plane and saw only it. The
near plane now sits at `dist × (1 + 1e-3)`. `views-monitor`,
`views-mirror` and `views-split` still pass against their Metal references
on lavapipe.

The tests:
- og-render `a_view_sees_what_its_layers_admit_from_its_own_eye`.
- og-probe `describe_names_the_views_each_entity_shows_in`.
- Both fail on the old code, which had no view pass and no `views` field.

Read from `og run` of the views station (`station-views.png`,
`station-views.describe.json`):
- `views` lists `vw-camera` (camera, 480×270, 12 entities), `vw-mirror`
  (mirror, 512×384, 9 entities, the room's: ground, walker, pillar, back
  wall, bezel, ball, screen, crate, right wall) and `vw-split` (camera,
  358×230, 11 entities).
- `vw-ghost`, the layer-1 crate, is absent from the main list and present
  under `vw-camera` only.
- Main entries name their views, for example `vw-left: [vw-camera,
  vw-split]` (the mirror does not show it) and
  `vw-crate: [vw-camera, vw-mirror, vw-split]`.
- The frame shows the mirror reflecting the walker, the ball and the
  pillar, the monitor showing the room with the magenta ghost crate, and
  the split view at the bottom right.

## Shared files touched

- `crates/og-api/api.ogt`: Lines, Points, CameraView and Camera.layers
  docs, `ParticleSystem.layers`, `PostEffects.custom5..8` and their docs,
  `RenderCapabilities.renderTargetDepth` and `maxCustomPosts`,
  `DescribedEntity.views`, `DescribedView`, `Description.views` and the
  describe command's doc. Also the generated `host.rs`, `index.d.ts`,
  `bindings.json`, `api.schema.json` and the wiki; most wiki changes are
  line-number churn.
- og-render:
  - `lib.rs` (the view passes: transparents, particles, lines, sprites,
    the depth prepass, the cull lists);
  - `views.rs` (the mirror's near plane, plan without `depth`);
  - `scene.rs` (the layer fields);
  - `lines.rs`, `sprites.rs`, `particles_gpu.rs` (per-view buffers and
    masks);
  - `cull_gpu.rs` and `shaders/cull.wgsl` (the layer mask);
  - `shaders/views_depth.wgsl`, `shaders.rs` (the GL tier's list),
    `id_pass.rs`, `tests/webgl2.rs`.
- og-runtime: `extract.rs`, `feel.rs`, `fx.rs`, `geometry.rs`, `views.rs`,
  `describe.rs`.
- og-probe: `hooks.rs`, `describe.rs`, `lib.rs`, `tests/expect.rs`.
- og-tests: `src/scenes.rs`, `src/target_scenes.rs`, `tests/targets.rs`,
  `tests/culling.rs`.
- `docs/design.md` §2.6, §3.4 and the custom post paragraph, and
  `packages/knowledge/skills/og-game/SKILL.md` §23.
- fx-showcase: `src/main.ts`, `src/e5.ts`, `assets/post/crt.ogt`
  (comment), `assets/post/tint.ogt` (new), `tests/shaders.og.ts`.

## Metal references owed (coordinator)

- gfx, new: `views-layers` and `views-depth`. Their tests fail only for
  want of a reference.
- gfx, to re-record: `views-monitor`, `views-mirror` (the near plane moved)
  and `views-split`. They match the committed Metal frames on lavapipe
  within tolerance, but the view passes changed. Re-record on Metal and
  read them.
- fx-showcase station frames: `views` (the views draw transparents,
  sprites and lines now, and the mirror's near plane moved) and `shaders`
  (the warm grade changes every pixel). Every station whose frame shows a
  view or a custom post should be re-read in the functional lane.
- Run `tests/shaders.og.ts` with `og test`. Its numbers were read from
  `og run` frames, not through `og test`.

## Left out

- In views: trails, 2D fluids, sprite glow and 2D lighting, and soft
  particles (the view has no soft-particle depth input). Bloom, AO, SSR,
  depth of field, grading, LUTs and custom posts per view are left out by
  design: those belong to the main frame.
- A list column in records (item 4); the account above is what it takes.
- ProbeTimings from an `og run` of a large scene for item 2: the numbers
  are the renderer's own, from a scratch test.
- No real-GPU or browser measurements of item 2 or item 3. Those are the
  wave's end.
