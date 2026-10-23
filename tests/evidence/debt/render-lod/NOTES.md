# debt/render-lod: the debts depth S6 left, paid

Branch `debt/render-lod` from main `b80aa70c`, pushed to origin. Each frame
here was rendered on lavapipe (`vulkan-cpu`, llvmpipe 15, the full tier) in
the orb and read before it was committed. I ran no gate, `og test`,
web-reference, or wasm/release build. The loop I did run: `cargo check`
and clippy on the crates I touched, their unit tests, `og api gen`,
`og api gen --wiki` and `og api check`, `og run --headless --commands`,
and og-tests' `probes` tests plus S6's three gfx behaviour tests.

## What landed

| Commit | Debt | What |
|---|---|---|
| `d8bd0c45` | 1, 2, 3 | og-render `probe_select.rs` (new): each instance takes the smallest probe box holding the centre of its bounds. `instances.rs` `Run::probe` keys the sort and the skinned batches key the same way, so one mesh and material in two rooms becomes two batches, each binding its own probe's frame group. Terrain levels, scatter chunks, water surfaces and the 3D fluids pick a probe per draw (the box holding their centre, else the smallest box their bounds reach). Captures now draw skinned meshes in their pose, terrain, scatter and the sky, then transparent meshes sorted back to front from the probe. The terrain, scatter, custom-shader, water and fluid shaders now sample through `frame.wgsl`'s `ambient_light`/`reflected_light`, with `ambient_light_level` added for shaders that cannot take derivatives. `probes::assign`, the aggregate-centre choice, is removed. |
| `bde7dcc1` | 4 | The prepass reflectivity is the gloss and metalness the lit pass shades: the factors times the maps' g and b, through `frame.wgsl` `reflectivity_bits`. Terrain writes its blended layer roughness (it is never metal). Scattered copies write their material's factors and maps (nothing when unlit). |
| `7dcb6653` | 6 | og-render `probe_content.rs` (new): `ProbeContent` hashes, by value, what a capture draws in its box: meshes, material factors and texture pixels, transforms to the millimetre, skinned meshes in bind pose, terrain revisions and layers, scatter, local lights reaching the box, the sun when it shines, and the environment. Mesh and texture hashes are cached per revision. The runtime key includes that hash and is resolved after extraction. Each bake gets a record beside it, `cooked/probes/<key>.ogt`, holding the model assets in the room with their cooked hashes and, when game code built part of the room, the hash of `src/`. `og doctor --project` reports `stale-probe`. |
| `d6248b63` | 1 | A capture draws scattered copies only within the box's half extent of it. Farther than that, a blade is smaller than a texel, and a dense field drawn six times was most of a lavapipe frame. |
| `068d18f5`, `d36a6b05` | 1, 2, 3 | og-tests `probe_scenes.rs`, with scenes `probe-two-rooms` and `probe-outdoor` (both `NATIVE_SCENES`) and `tests/probes.rs`. |
| `362b955f` | 4 | `ssr_reads_roughness_maps_and_terrain_layers`. |
| `d5a5a27b`, `f4ab30e5` | 5 | og-probe `expect:entity:<id>:<path><op><value>` (a field of `dump_entity`) and `expect:timings:<path><op><value>`. Both are unchecked without a renderer for `render.*`, `triangles`, `render` and `passes`. Playtests `templates/fx-showcase/tests/lod.og.ts` and `reflections.og.ts`. The IDL doc of the `expect` command and design §3.4 are updated, and the generated outputs regenerated. |
| `4a9c962c` | all | `docs/design.md` §2.6 and og-game skill (reflection probes, SSR). |

On debt 2, I weighed a per-instance GPU index against splitting batches.
A per-instance index would need every probe's cube bound at once: either a
cube array, which WebGL2 lacks, or one texture slot per probe, and the
skinned lit pipeline on WebGL2 has none left. Splitting the batches reuses
S6's frame-group switching, and it costs an extra draw only where a batch
actually spans rooms. Within a draw, the shader still blends the probe
into the environment per fragment (`probe_weight`).

## Frames read and numbers

- `frames/gfx-probe-two-rooms-lavapipe.png`: two open rooms, red on the
  left and blue on the right. The chrome spheres share one mesh and one
  material. The left sphere's upper half shows the red ceiling and walls,
  measuring (222,133,109). The right sphere shows the blue room, measuring
  (128,170,218). Without probes both measure the same sky, (164,196,224)
  and (167,197,225). (`each_of_two_rooms_reflects_its_own_room`)
- `frames/gfx-probe-outdoor-lavapipe.png`: a 48 m meadow with grass, a
  translucent red pane, magenta skinned cylinders posed to the right, and
  a chrome sphere beside the probe. The sphere reflects the meadow below,
  the red pane and the magenta figures. The capture was read back face by
  face, against the same scene without the figures and the pane:
  - +x (figures) measured (0.48,0.31,0.62) against (0.37,0.45,0.55).
  - -z (pane) measured (0.86,0.21,0.25) against (0.35,0.43,0.53).
  - -y is green, (0.03,0.07,0.03): the terrain and grass.

  Before this work, -y would have been the sky's ground and +x and -z the
  control's colours. (`a_capture_draws_skinned_meshes_terrain_grass_and_glass`)
- Terrain and grass take the probe: raising the probe's intensity from 1
  to 4 brightens the meadow inside the box from (57,89,42) to
  (102,152,94). The far meadow outside the box stays at (66,101,51) in
  both frames. (`terrain_and_grass_in_a_probes_box_take_its_light`)
- SSR, under the red sphere of `ssr-wet-floor`:
  - The floor of roughness factor 1 with a 0.04 roughness map mirrors the
    sphere at (55,31,30); without the map it measures (42,42,42).
  - A terrain layer of roughness 0.05 mirrors it at (55,31,30); with SSR
    off it measures (31,31,31).

  (`ssr_reads_roughness_maps_and_terrain_layers`)
- No regression in S6's tests, whose numbers are unchanged: the probe
  interior's lower half measures (134,140,156) with the probe and
  (128,119,108) without; `ssr-wet-floor` measures (55,31,30) against
  (31,31,31); `lod-levels` passes.
- `frames/station-lod-lavapipe.png` and `station-lod-bias0-lavapipe.png`
  (the `lod.og.ts` script) show the forest under the sun, the same at full
  detail. All 8 expectations pass:
  - With LOD on, tree-20 draws level 1, tree-1980 level 2, and the frame
    has 1,300,812 triangles.
  - With `lodBias` 0, both draw level 0 and the frame has 5,150,788
    triangles.

  Run with `--no-render`, the same script gives 1 pass and 7 unchecked.
- `frames/station-reflections-probe-on-lavapipe.png` and `…-off-…` (the
  `reflections.og.ts` script) show the chrome sphere reflecting the room
  (the blue wall, the ceiling, the gold sphere). With the probe hidden it
  reflects the sky. The upper-half region measured #b4adab with the probe
  and #bccfe6 without, each within its tolerance of 20 and outside the
  other's. 3/3 pass.
- Bake key, on the fx-showcase reflections station:
  - `og bake probes` wrote `probes/683bc6a48275a81e` and its record, and
    the next run reported `bakedFromCache: true`.
  - I then recoloured the gold sphere in `src/depth.ts`. The run reported
    `bakedFromCache: false`, and `og doctor --project` warned
    `stale-probe … the game's code, which builds part of its room, changed
    since its bake`.
  - With the source restored, the doctor printed no probe finding and the
    run went back to the cache.

## Shared files touched

- **og-render `lib.rs`**:
  - `mod probe_select`, `mod probe_content` and the re-export of
    `ProbeContent` and `ProbeRoom`.
  - `Batch::probe`.
  - The probe boxes go to the instance sort before `build_batches`.
  - `build_skinned` takes the boxes.
  - The probe-face culling now also produces each face's skinned list and
    sorted transparent list.
  - `draw_list` binds through `probe_select::Binder` (in place of S6's
    `(&[u8], groups, frame)` tuple).
  - `draw_skinned` takes a probe argument, so all seven of its call sites
    changed.
  - `terrain::draw_levels` and `scatter.draw` take an
    `Option<&mut Binder>`, which changes every call site.
  - `water.draw` takes a `&mut Binder`.
  - The `FramePass::Probes` arm draws the skinned meshes, terrain, scatter
    and transparents.
  - The fluid bounds are computed for their probe.

  Much of the line count is rustfmt reflowing those calls.
- og-render, other files:
  - `instances.rs`: `Run::probe`, `set_probes`, and `refresh` now returns
    `Option`.
  - `probes.rs`: `assign` removed.
  - `scene.rs`: `texture_revision` and `terrain_revision`.
  - `terrain.rs`, `scatter.rs`, `water.rs`: as above.
  - WGSL: `frame.wgsl` (`ambient_light_level`, `reflectivity_bits`),
    `pbr_prepass.wgsl`, `terrain_lit.wgsl` (`terrain_look` split out of
    `fs`), `scatter.wgsl`, `custom_lit.wgsl`, `water.wgsl`, `fluid3d.wgsl`.
  - I did not touch `post.rs` or `ssr.rs`.
- `crates/og-api/api.ogt`: only the doc text of the `expect` command
  variant changed. The generated outputs were regenerated: `types`,
  `schema` and wiki `commands.md`.
- og-runtime: `probes.rs` and `probe.rs` (the capture hook calls
  `apply_bakes`).
- og-cook: `cook/probes.rs` (`ProbeRecord`, `status`; `write_probe` takes
  a record).
- og-cli: `bake_probes.rs` (`code_hash`) and `doctor_project.rs`. I did
  not touch `main.rs`.
- og-probe: `expect.rs` and `command.rs`.
- og-tests: `scenes.rs` (two `NATIVE_SCENES` and their build arms) and
  `lib.rs`.
- `templates/fx-showcase/tests/` (two new files), `docs/design.md` §2.6
  and §3.4, and the og-game skill.

## Metal references owed (the coordinator's)

These are in og-tests `tests/probes.rs` and are recorded with
`OG_GFX_CAPTURE=probe-two-rooms,probe-outdoor`:

- `probe_two_rooms_matches_its_reference`: `probe-two-rooms.png`
- `probe_outdoor_matches_its_reference`: `probe-outdoor.png`

The new playtests compare no reference frame. They assert with `expect`
steps, so they need no station reference.

S6's `probe-interior` gives the same numbers as before, so its Metal frame
should not move. Terrain and scatter with a reflection probe, and floors
with roughness maps under SSR, now render differently than before, so any
existing reference that combines them could move. I know of none.

## Left out, or worth knowing

- **Skinned instances choose their probe by their bind-pose bounds**, not
  their posed ones. A character whose pose carries it far from its origin
  into another room keeps its origin's probe.
- **Captures draw no particles, sprites, lines, water or 3D fluids**, and
  they draw scattered copies only near the box. Reflections inside a
  capture are still the environment's (one bounce).
- **The room's hash cannot see code-built geometry when the code is
  absent.** A bake whose room the game's code built carries the hash of
  `src/`, so any source edit marks it stale in `og doctor`, even one that
  does not touch the room. This errs toward reporting. The runtime key
  itself is exact: it hashes what is drawn.
- **Bakes made before this branch** (S6's key) are never sampled now,
  because their key no longer matches. `og doctor` reports them as having
  no record.
- The **`probe-outdoor` scene is a small meadow of its own**. I first
  built it on the island scene, a 24 km sea with a 1.6 km terrain, and a
  single frame took 370 s on lavapipe without the probe too, so the probe
  was not the cost.
- WebGL2 was not run in a browser. `og-render/tests/webgl2.rs` and
  `shaders.rs` pass: naga validation and GLSL ES 300 translation of every
  module.
