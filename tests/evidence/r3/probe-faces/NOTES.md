# Round 3 `probe-faces`: evidence

Branch `r3/probe-faces`, cut from origin/main b86bcb91. It pays the
`probe-faces` row of docs/phase-1.md ("Plan 2026-10-24, round 3"):
3D fluids and depth in reflection probe captures, so water in a face
refracts and soft particles fade there. These are the two things
`tests/evidence/debt2/render-views-2/NOTES.md` left out ("Faces have no
depth of their own", "3D fluids in probe captures").

Every check ran in an orb on lavapipe (Mesa, llvmpipe, `vulkan-cpu`,
debug build). Following AGENTS.md, nothing here ran `scripts/check.sh`,
`og test`, web-reference, a wasm or a release build. The loop was:

- `cargo check -p og-render`;
- `cargo clippy -p og-render -p og-tests --all-targets -- -D warnings`
  and `cargo fmt`;
- og-tests `--test probes` (the suite the new tests join), `--test
  targets` (views, whose depth prepass is now shared), `--test fluids3d`
  (the fluid's prepare was split), `--test atmosphere` (water), then
  `--test gfx` and `--test light` under nextest, one at a time.

Each new test was run on the old code by stashing (or checking out at
b86bcb91) `crates/og-render` and rerunning it. The numbers are below.

Results on the branch head:

| Suite | Result |
|---|---|
| og-tests `probes` | 12/12 (3 new) |
| `targets` | 13/13 |
| `fluids3d` | 6/6 |
| og-render `--lib` | 81/81 |
| og-tests `gfx` (nextest) | 57 of 60 |
| og-tests `light` (nextest) | 13 of 15 |
| `atmosphere` | 11 of 12 |

None of the failures is this branch's:

- `sprites-array`: RMSE 0.01255.
- `tilemap-lit`: RMSE 0.02653, 1.917 % outliers.
- `stylise_paint_matches_its_reference`, `baked_batches_matches_its_reference`
  and `every_batch_kind_draws_its_lightmap`: SIGSEGV in the driver.

These five are the lavapipe failures render-views-2 recorded at its base
ad903b18, with the same numbers. Here, the two light ones were rerun on
b86bcb91 and crash there too; the three gfx ones were not rerun on the
base.
- `atmosphere-night`: RMSE 0.01312, against the Metal reference, as
  there is no vulkan-cpu one. It was rerun on b86bcb91 with the same
  number.

## Design

```
per due capture, per face (only while a capture is due)
┌───────────────────────┐   ┌─────────────────────────┐
│ depth_prepass         │──▶│ fluid3d encode          │  (3D fluids only)
│ (views' prepass draws │   │ depth, thick, 2 blurs   │
│  into FaceTargets)    │   │ through the face camera │
└──────────┬────────────┘   └───────────┬─────────────┘
           ▼                            ▼
┌──────────────────────────────────────────────────────┐
│ face pass: opaque, skinned, terrain, scatter, sky    │
└──────────┬───────────────────────────────────────────┘
           ▼  (water or fluids: end the pass, keep colour and depth)
   copy the face layer ──▶ FaceTargets.copy
           ▼
┌──────────────────────────────────────────────────────┐
│ face pass again (Load): water, fluid composite,      │
│ transparents, particles (soft: the face's lindepth), │
│ lines, sprites                                       │
└──────────────────────────────────────────────────────┘
```

- **Where a face's depth comes from: a depth prepass per face.** The face
  does not use its own depth attachment. Water, soft particles and the
  fluid all `textureLoad` a linear view depth (`LINDEPTH_FORMAT`). The
  face's attachment is `Depth32Float`, multisampled under MSAA, and GLSL
  ES 3.0 cannot load a depth buffer, least of all a multisampled one.
  Views made the same choice (`views.rs` module docs). The face prepass is
  the views' prepass. Its draws, inline in the view pass before, became
  the `depth_prepass` closure in `Renderer::render`, which views and faces
  both call. Its attachments come from `views::DepthPrepass::new`, also
  shared. The faces add the scattered copies near the box (the views pass
  `None`, as before).
- **The 3D fluid per face, without a second copy.**
  - `Fluid3dRenderer::prepare` is split. The uniforms come from the free
    function `camera_uniforms`, and the bind groups from
    `Fluid3dRenderer::frame`.
  - The main camera writes the renderer's own buffers, as before.
  - A face calls `Fluid3dRenderer::camera_frame`, which makes uniforms of
    its own over the instances the main frame uploaded.
  - `encode` (the four passes) and `draw` (the composite) are the main
    pass's functions, unchanged. The face-sized targets are
    `fluid3d::Fluid3dTargets`.
- **Water.** `WaterRenderer::face_draws` binds the face copy and the
  face's linear depth, with a uniform made per capture carrying the face's
  size. Before, it bound white and a 1×1 far plane with a screen of 1×1.
  Screen-space and planar reflections stay off in faces.
- **The face copy.** As in the main pass, the face's pass ends after the
  sky, storing its MSAA colour and depth. The resolved layer is copied
  into `FaceTargets.copy` (the capture texture gained `COPY_SRC`), and a
  second pass loads both and draws water and the fluid composite.
- **Ownership.** `Probes.targets: Vec<FaceTargets>` holds one set per
  face size, shared by every face and due probe of that size, since faces
  are encoded one after the other. `Probes::ensure_targets` makes a set
  the first frame a due capture needs it (copy and fluid targets only
  when wanted). `Probes::prepare` drops sets whose size no slot has.

## What landed, one commit per item

| # | Item | Commit | Test (fails on the old code) |
|---|------|--------|------------------------------|
| 0 | One depth prepass for views and faces (refactor, no change) | 7d78457 | og-tests `targets` 13/13 unchanged (views-depth among its references) |
| 1 | Water in a face refracts over the face's own depth | 66f849b | og-tests probes `water_in_a_capture_shows_the_floor_below` |
| 2 | Soft particles in a face fade against its depth | 95a9bb0 | og-tests probes `soft_particles_in_a_capture_fade_against_the_floor` |
| 3 | Probe captures draw 3D fluids | cf461d3 | og-tests probes `a_capture_draws_3d_fluids` |

All three tests share one new scene in `og_tests::probe_scenes`. It is
`probe_floor_scene`: a yellow floor 10 m square, a high sun, a dark grey
background, and a baked 128² probe 1.2 m over the floor (id 7). Each test
adds its content with `add_floor_water`, `add_floor_particles` or
`add_floor_fluid`. No reference frame was recorded: every test compares
two captures of its own, read back with `read_reflection_probe` (face
means over the middle half of a face, linear RGB).

### 1. Water refracts in a face (66f849b)

0.15 m of clear water (shallow white, deep dark blue, no foam) lies over
the yellow floor.

| -y face | R | G | B |
|---|---|---|---|
| bare floor | 1.005 | 0.914 | 0.093 |
| with water | 1.007 | 0.928 | 0.190 |
| with water, old code | 0.171 | 0.202 | 0.436 |

The old capture showed the water's deep blue: it read the far plane
below, so the absorption was total, and white showed through. The test
asks for yellow (R > 3B, G > 2.5B) with R and G within 0.6 to 1.3 of the
bare floor's. The old code fails it on both counts.

Read in the dumped faces (a scratch dump, not committed):
- The -y face is the yellow floor, a little lighter, with the sun's white
  glint.
- The side faces show the water at grazing angle as a pale band at the
  horizon, the Fresnel reflection of the environment.

The existing `a_capture_draws_water` (a red pond over the meadow) still
passes: -y reads 0.198/0.035/0.035 against the meadow's 0.040/0.084/0.031.
That is now the meadow seen through red-tinted water, where it was the
deep colour before.

### 2. Soft particles fade in a face (95a9bb0)

A few green particles, 1.2 m wide, hover 0.1 to 0.2 m over the floor.
Soft particles fade over 2 m. "Off" is the sum of the channel differences
from the bare floor on the -y face.

| -y face | R | G | B | off |
|---|---|---|---|---|
| bare floor | 1.005 | 0.914 | 0.093 | 0 |
| hard | 0.540 | 0.932 | 0.122 | 0.512 |
| soft | 0.965 | 0.915 | 0.096 | 0.043 |
| soft, old code | 0.540 | 0.932 | 0.122 | 0.512 (= hard) |

The test asks for soft < 0.4 × hard. In the dumped faces, the hard
particles are distinct green dots on the -y face; the soft ones are a
faint trace.

### 3. 3D fluids in a face (cf461d3)

A red fluid ball (radius 0.7 m, about 1,360 particles on a 0.1 m grid) sits
1.5 m to the probe's +x side.

| +x face | R | G | B |
|---|---|---|---|
| bare (dark background over the yellow floor's lower half) | 0.305 | 0.281 | 0.064 |
| with the fluid | 0.272 | 0.108 | 0.047 |
| with the fluid, old code | 0.305 | 0.281 | 0.064 (= bare) |

The test asks for R > 2G, R > 2B and G < 0.6 × the bare G. In the dumped
face, the ball is a dark red disc centred in the +x face, rimmed by the
environment's reflection, over the horizon. The old capture is
bit-for-bit the bare one.

## Main frames

These scratch renders were deleted after reading, comparing the main
frame of b86bcb91 with the branch's (1280×720, pixels whose largest
channel moved by more than 12):

| Scene | Moved > 12 | Any change | Max |
|---|---|---|---|
| `probe_floor_scene` (bare) | 0 | 0 | 0 |
| `probe_effects_scene(true)` (no water, fluid or soft particle) | 0 | 0 | 0 |
| floor + soft particles | 0 | 4,627 | 1 |
| floor + water | 0 | 589,710 | 5 |
| floor + fluid | 35 | 21,278 | 34 |
| floor + all three | 13,572 | 601,899 | 66 |
| `probe_water_scene(true)` (the pond) | 21,737 | 380,180 | 34 |

- **No move where nothing changed.** A scene whose probes see no water,
  fluid or soft particle does not move. `probe-two-rooms`,
  `probe-outdoor`, the five views references and `fluid-3d` all pass on
  lavapipe.
- **The rest move only through the probe.** What moves is what samples
  the changed capture: the water's and the fluid's reflections, and the
  ambient light of terrain and grass inside the probe's box. On the pond,
  the meadow and grass inside the box are lighter in the branch's frame:
  the capture's lower hemisphere is now the meadow through red water,
  where it was the deep red. On floor + fluid, the 35 pixels are at the
  ball's rim, which now reflects itself in the capture.

## Capture-time cost

Nothing runs per frame without a due capture:
- `face_depth`, `face_copy` and `face_fluid` all require
  `!probe_frame.due.is_empty()`;
- `Probes::ensure_targets` is only called then;
- a frame with no due capture draws exactly the old draws.

Per due face, it draws:
- with water: one prepass (its opaque list, opaque skinned, terrain,
  scatter near the box), one texture copy, and a second pass begin;
- with soft particles only: the prepass;
- with 3D fluids: the prepass, four fluid passes, the copy, and one
  composite draw.

Draw calls in a frame that captures (old → new):

| Scene | Old | New |
|---|---|---|
| floor + water | 16 | 22 (+1 per face) |
| floor + soft | 16 | 22 |
| floor + fluid | 14 | 50 (+6 per face) |
| pond (terrain + grass) | 104 | 149 |
| `probe_effects_scene` | 110 | 110 |

Frames without a capture draw the same before and after.

Wall time per frame on lavapipe, a realtime probe capturing every frame
(only indicative: llvmpipe is one CPU rasteriser):

| Scene | No capture, old | No capture, new | Capture, old | Capture, new |
|---|---|---|---|---|
| pond | 913 ms | 911 ms | 2,679 ms | 3,196 ms |
| floor + fluid | 148 ms | 146 ms | 356 ms | 591 ms |
| floor + water | 122 ms | 105 ms | 395 ms | 408 ms |

Memory, kept per face size while a probe of that size exists:
- the prepass, 24 B a texel;
- the copy, 8 B;
- the fluid targets, 36 B.

That is 0.4 MB (prepass only) to 1.1 MB (all three) for a 128² probe,
and 25 MB to 71 MB for a 1024² one.

## Metal references owed (coordinator)

No reference frame was added or re-recorded, so no lavapipe copy is
new. Re-read these in the functional lane:

- `probe-two-rooms`, `probe-outdoor`, `fluid-3d`, `water-3d` and the
  views references. They pass on lavapipe and nothing in them should move
  (their probes' boxes hold no water, fluid or soft particle).
- Any fx-showcase station or template frame with a reflection probe whose
  box holds water, a 3D fluid or soft particles. Its capture now refracts,
  fades and draws fluids, and what samples it moves, as the pond above.
  The islands station's water and the probes station are the ones to look
  at.
- The browser tiers (web-reference) run the same code on WebGL2: the
  face prepass, the copy, and the fluid passes per face. No shader
  changed, and nothing here ran on WebGL2.

## Shared files touched

- **og-render `src/lib.rs`, `Renderer::render` only:**
  - the new `depth_prepass` closure, and the view pass now calling it;
  - the `face_copy`, `face_depth` and `face_fluid` flags and the
    `Probes::ensure_targets` call beside `view_depth`;
  - `ensure_pipelines`' prepass flag (`|| face_depth`, and the probe
    skinned batches' `face_depth`);
  - `face_water`, now per due capture;
  - the new `face_fluids`;
  - the face particle groups binding the face's linear depth;
  - the `FramePass::Probes` arm: the per-face prepass, the fluid encode,
    the `begin` closure that splits the pass, the copy, water and fluid
    after it.
- **og-render `src/probes.rs`:**
  - the module docs;
  - `ProbeSlot.capture` (the capture texture, now `COPY_SRC`) and
    `ProbeSlot::new`;
  - the new `FaceTargets`;
  - `Probes.targets`;
  - `Probes::prepare` (drops unused sizes);
  - the new `Probes::ensure_targets` and `Probes::targets_of`.
- **og-render `src/views.rs`:** the new `DepthPrepass::new`; `textures`
  calls it.
- **og-render `src/water.rs`:** `Slot.face` (a `WaterGpu` copy replacing
  `face_uniform`), `WaterRenderer::prepare`, `WaterRenderer::face_draws`
  (a new signature: `size`, `below`, `scene_depth`).
- **og-render `src/fluid3d.rs`:**
  - the module docs;
  - the new `Fluid3dTargets`;
  - `Fluid3dRenderer::prepare` (split);
  - the new `Fluid3dRenderer::camera_frame` and `Fluid3dRenderer::frame`;
  - the new free function `camera_uniforms`.
- **Shaders:** none.
- **og-runtime and `api.ogt`:** untouched, so no `og api gen` is needed.
- **og-tests:** `src/probe_scenes.rs` (`FLOOR_PROBE`, `probe_floor_scene`,
  `add_floor_water`, `add_floor_particles`, `add_floor_fluid`) and
  `tests/probes.rs` (three tests).
- **Docs:** `docs/design.md` §2.6 (Reflection probes) and the og-game
  skill's probes paragraph, which no longer says 3D fluids are left out.

## Left out, and why

- **Screen-space reflections on water in a face.** A face now has the
  depth and the copy they would march through, but they mirror from the
  probe's one capture point, and the planar reflection is one surface a
  frame. They stay off, as before.
- **Local lights on a 3D fluid in a face.** The fluid's `cluster_lights`
  reads the light clusters. A face's uniform has none: `cluster` is
  zeroed and every light is meant to be looped, which `pbr_fs` and
  `physical` do. Terrain and scatter in faces already read the clusters
  the same way, so this is not new to the fluid. The test scene has no
  local light, and nothing here checks it. The fix would be
  `pbr_fs.wgsl`'s every-light branch in `fluid3d.wgsl`,
  `terrain_lit.wgsl` and `scatter.wgsl` together.
- **Transparent meshes in the face copy.** As in the main pass, the copy
  is taken after the sky, so water and fluids show the opaque face, not
  the glass drawn after them.

## For the coordinator to decide

- Whether faces should loop every local light in the fluid, terrain and
  scatter shaders (see Left out). It is a shader change across three
  modules, and would move frames of scenes with local lights inside probe
  boxes.
- Whether the per-size `FaceTargets` should be freed when no capture is
  due for a while. A baked probe keeps them for its lifetime: up to
  71 MB at 1024² with water, fluids and soft particles, for one capture.
  Freeing them after the capture would make a realtime probe remake them
  each interval.
