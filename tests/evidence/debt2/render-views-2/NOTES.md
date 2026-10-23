# Debt round 2 `render-views-2`: evidence

Branch `debt2/render-views-2`, cut from origin/main ad903b18. It pays the
`render-views-2` row of docs/phase-1.md ("Plan 2026-10-23, debt round 2"),
what render-targets' and render-light's NOTES left
(`tests/evidence/debt/render-targets/NOTES.md` §Left out,
`tests/evidence/debt/render-light/NOTES.md` §Left out).

Every check ran in an orb on lavapipe (Mesa 22.3.6, llvmpipe LLVM 15,
`vulkan-cpu`, debug build). Following AGENTS.md, nothing here ran
`scripts/check.sh`, `og test`, web-reference, a wasm or a release build.
The loop was `cargo check -p og-render`, `cargo clippy -p og-render -p
og-tests --all-targets -D warnings`, `cargo fmt --check`, og-render's
`--lib`, `--test shaders` and `--test webgl2`, and og-tests' `--test
targets`, `--test probes`, `--test light`, `--test atmosphere` (water),
`--test sprites2d`, `--test effects` and `--test gfx`, one at a time. Each
new test was run on the old code by stashing `crates/og-render` and
rerunning it (numbers below). Frames were read by region and face means
and moved-pixel counts, printed by the tests; the test frames were saved
with `OG_VIEW_FX_OUT`, `OG_PROBE_OUT` and `OG_RECT_SHADOW_OUT` to a
scratch directory and looked at, not committed. No new reference frame
was recorded: every new test compares two renders of its own.

og-tests `gfx` and `light` under nextest: 69 of 74 pass. The other five
fail identically on ad903b18 on this lavapipe, with the same numbers:
- `sprites-array`: RMSE 0.01255.
- `tilemap-lit`: RMSE 0.02653, 1.917 % outliers.
- `stylise_paint_matches_its_reference`, `baked_batches_matches_its_reference`
  and `every_batch_kind_draws_its_lightmap` SIGSEGV in the driver (the
  physical-pipeline crash render-light reported).

## What landed, one commit per item

| # | Item | Commit | Test (fails on the old code) |
|---|------|--------|------------------------------|
| 1 | `custom_lit.wgsl` lights rect lights through `surface_local_light` with their shadow | 212e1f01 | og-tests light `a_full_custom_shader_lights_rect_lights_with_their_shadow` |
| 2 | Skinned instances pick their probe from their posed bounds | a1179789 | og-tests probes `a_skinned_instance_takes_the_probe_of_its_pose` |
| 3 | Views draw trails, ribbons and beams | 5bf3a475 | og-tests targets `a_view_draws_the_main_passs_effects` (Ribbon) |
| 4 | Views 2D-light their sprites | 5f5b3cf1 | the same test (Lit2d) |
| 5 | Views draw 2D fluids | 2a8a9702 | the same test (Fluid2d) |
| 6 | Views' soft particles | 22da963d | the same test (SoftParticles) |
| 7 | Views' sprite glow | f04477cf | the same test (Glow) |
| 8 | Probe captures draw particles, lines and points, world sprites | dadc0779 | og-tests probes `a_capture_draws_particles_sprites_and_lines` |
| 9 | Probe captures draw water | 94d3479a | og-tests probes `a_capture_draws_water` |

### 1. Rect lights in a full custom lit shader (212e1f01)

- Root cause: `custom_lit.wgsl` had its own local-light loop
  (`custom_local`, pbr.wgsl's point and spot without the toon ramp). A
  rect light went through the point branch: lit as a point at the panel's
  centre, with no LTC, no `rect_shadow`, no cookies.
- Change: the full shader's lit module includes `area.wgsl` (custom.rs
  `lit_source`) and the loop calls `surface_local_light`, the function
  terrain, scatter and the 3D fluid already share. `custom_local` is gone.
- Test: `rect_shadow_custom_scene` is render-light's `rect_shadow_scene`
  with the floor and the post drawn by a full lit shader that keeps the
  standard entries. Floor luma at the post's foot and toward the shadow's
  tip:

  | Run | Foot | Tip |
  |---|---|---|
  | standard material, unshadowed | 114.0 | 58.0 |
  | custom, unshadowed | 114.0 | 58.0 |
  | custom, shadowed | 8.2 | 30.8 |
  | custom on the old code, unshadowed | 66.5 | 37.3 |
  | custom on the old code, shadowed | 66.5 | 7.1 |

  The old shader fails the first assertion (66.5 against 114.0 for the
  standard floor): a point light, and no shadow at the foot.

### 2. Skinned probes from the posed bounds (a1179789)

- Root cause: `build_skinned` picked an instance's probe by its mesh's
  bind-pose box through its transform. A figure moved by its joints (root
  motion, a walk) kept the probe of where its transform stands.
- Change: each skinned mesh keeps the bind-pose box of the vertices each
  joint moves (`skin::joint_boxes`, made at upload and again on a mesh's
  revision). The instance's centre is those boxes moved by its palette
  and its transform (`skin::posed_centre`). A morphed mesh without a skin
  keeps the bind pose's. The CPU cost is 8 corner transforms per joint per
  instance per frame.
- Test: `posed_figure_scene` puts a chrome skinned cylinder into
  probe-two-rooms. Posed: its instance stands in the red room and every
  joint moves it 7.2 m into the blue room. Placed: the same cylinder put
  there by its transform. The mean of the cylinder's 11,764 pixels:

  | Run | Mean RGB |
  |---|---|
  | placed | 158.0 161.3 170.0 |
  | posed | 158.1 161.4 170.1 |
  | posed, old code | 167.0 164.9 163.9 (the red room's probe) |

  Read in the saved frames: the placed and posed cylinders reflect the
  blue walls on their sides; the old posed one reflects warm.

### 3 to 7. What views draw of the main pass (5bf3a475 .. f04477cf)

One scene and one test cover all five, `target_scenes::view_fx_scene`
and `a_view_draws_the_main_passs_effects`. The scene is a dark wall; a
viewport view draws the frame's right half looking at it; the main
camera looks the other way. Each effect is drawn on and off. The view's
half must move by at least 400 pixels (> 12 on a channel), the main half
by 0.

| Effect | View pixels moved | Main | Old code |
|---|---|---|---|
| Ribbon | 36,006 | 0 | 0 |
| Lit2d | 148,339 | 0 | 0 |
| Fluid2d | 93,997 | 0 | 0 |
| SoftParticles | 14,777 (14,113 differ from the same particles drawn hard) | 0 | 0 |
| Glow | 47,275 | 0 | 0 |

Each item was checked failing on the old code before the next one landed.

- **3. Trails, ribbons and beams.**
  - Root cause: the view pass drew no 3D strip, and its layered 2D
    trails reused the vertices built facing the main eye.
  - Change: `Trails::build` keeps the frame's strips, and `Trails::emit`
    lays them out for any eye. Each view's strips, and its 2D water
    strips, follow the main camera's in the strip buffer. They are drawn
    after its particles (`draw_strips`, shared with the main pass) and
    with its sprites.
  - Frame: the ribbon band at 960,330 reads 177/152/130 and at 960,360
    232/200/99, over the wall's 129/131/136.
- **4. 2D lighting.**
  - Root cause: the light map is drawn through the main camera at the
    frame's size, so `sprites::build` ran unlit for views.
  - Change: each view keeps a light map of its size × `Lighting2d.resolution`
    and a sprite-fx uniform of its image size. The frame's lights and
    occluders are drawn into the map through its camera
    (`Light2dRenderer::write_view`/`encode_view`) before its pass, and the
    lit layers are lit as on the main camera.
  - Frame: the sprite reads 55/55/55 at its lower left (ambient 0.05) and
    244/237/223 toward the light. Unlit it is 229 everywhere.
- **5. 2D fluids.**
  - Root cause: the metaball fields are main-frame-sized and the
    composite loads them by pixel.
  - Change: each view keeps field targets of its size, as many as the
    frame's. The kernels are drawn into them through its camera
    (`encode_field` with its frame group), and its surfaces are composited
    in its sprite order from them.
  - Frame: one blue blob with a light rim, 97/181/213 at its centre.
- **6. Soft particles.**
  - Root cause: a view bound the 1×1 far-depth placeholder. The particle
    shader loads it at its pixel, which is out of bounds past (0, 0); with
    robust access that reads 0, so `scene - depth` < 0 and every soft
    particle of a view vanished, not just its softening.
  - Change: a frame with soft particles (`ParticleFrame.soft`) runs each
    view's depth prepass. This is the one a target's depth texture
    already had, now moved before the view's pass. Its particles fade
    against its linear depth.
- **7. Sprite glow.**
  - Root cause: a view dropped its glow instances, since the glow buffer
    feeds the main frame's bloom chain.
  - Change: with `PostEffects.glow`, each view draws its glowing sprites
    into a glow buffer at half its size and fills its box-filtered mip
    chain (`MipGenerator`). `views_glow.wgsl` adds the levels onto its
    HDR image before the resolve: as many levels as `bloom_levels`, each
    spread by a 3×3 tent. The new module is on the shader lists, so
    `shaders.rs` validates and translates it for both tiers.
  - Compared with the same sprite glowing in the main frame (a scratch
    test, deleted), at 60/80/100/140/200 px from the centre: view
    251/239/226/176/132, main 251/240/225/190/150. The far halo is
    weaker: box mips spread less than the bloom chain's 13-tap
    downsample. Without the tent it was 231/211/131 at 80/100/140.

### 8. Probe captures: particles, lines and points, sprites (dadc0779)

- Root cause: the probe pass drew opaque, skinned and transparent meshes,
  terrain, scatter and the sky only.
- Change: each due face draws the following after its transparents:
  - the particle systems, through a billboard uniform of its own;
  - the scene lines and points, through a line uniform of its own;
  - the world sprites and text. These are built once for all faces, on
    every layer, unlit by 2D lights and without glow, after the views' in
    the sprite buffer.

  `views::camera_gpu` makes the two uniforms for views and faces alike.
  The particle shader's soft-depth load is clamped to its texture, so a
  pass binding the 1×1 far plane reads it instead of 0. A face's soft
  particles draw at full strength.
- Test: `probe_effects_scene` puts a red sprite on the probe's -z side, a
  yellow line loop on +x and a green fountain on -x. Face means of the
  capture (linear RGB):

  | Face | With | Bare meadow | Old code |
  |---|---|---|---|
  | -z (sprite) | 0.96 0.10 0.09 | 0.35 0.43 0.53 | = bare |
  | +x (lines) | 0.62 0.63 0.36 | 0.37 0.45 0.55 | = bare |
  | -x (particles) | 0.31 0.61 0.49 | 0.36 0.44 0.54 | = bare |

### 9. Probe captures: water (94d3479a)

- Root cause: the probe pass left water out. The water shader reads the
  frame's scene copy and depth below, and a face has neither.
- Change: `WaterRenderer::face_draws` makes, per surface, a group over a
  face uniform (no SSR, no planar reflection, screen 1×1). It binds the
  far plane as the depth below and white as what shows through. A face
  therefore draws the deep colour (the shallow colour mixed to the deep
  by opacity) over the reflected environment and the sun's highlight,
  without refraction or edge foam. Faces draw it after their sky.
  water.wgsl clamps its depth loads to the texture (`depth_texel`), which
  changes nothing for the main frame's screen-sized depth: water-3d,
  water-2d and fluid-2d still match their references (rmse 0.00167,
  0.00206, 0.00087).
- Test: `probe_water_scene` puts a red-tinted 6 m pond 0.4 m under the
  probe. The -y face reads 0.53/0.06/0.06 against the meadow's
  0.04/0.08/0.03; the old capture reads the meadow's.

## Metal references owed (coordinator)

No reference frame was added or re-recorded. Re-read these in the
functional lane, since the passes they draw changed:
- og-tests `views-monitor`, `views-mirror`, `views-split`, `views-layers`
  and `views-depth`. They pass on lavapipe; their scenes have no trails,
  2D lighting, fluids, soft particles or glow, so they should not move.
- `probe-two-rooms` and `probe-outdoor`. They pass on lavapipe; neither has
  particles, lines, sprites or water in a box.
- `shader-full-dissolve` and every station with a full custom lit shader
  under a point or spot: those lights now go through
  `surface_local_light`, which adds the light's cookie. Only a rect light
  changes the result.
- fx-showcase stations that show a view or a probe next to trails,
  sprites, particles, lines or water (the views station, the probes
  station); any game whose skinned figures cross probe boxes.
- The browser tiers (web-reference): views_glow.wgsl and the per-view
  light maps and fields run on WebGL2 too. They were checked through
  naga translation only (`--test webgl2`, `--test shaders`).

## Shared files touched

- og-render `src/lib.rs`: the view pass (strips, light map, fields,
  depth prepass moved before the pass, glow), the probe pass (particles,
  lines, sprites, water), `build_skinned`'s probe pick, the `draw_strips`
  and `draw_glow` closures and the `draw_sprites` fields argument.
- og-render `src/probes.rs` (module doc only). `src/post.rs` is untouched;
  `bloom_levels` is read from views.rs.
- Shaders: `custom_lit.wgsl`, `particles.wgsl` (the clamped depth load),
  `water.wgsl` (`depth_texel`), `views_glow.wgsl` (new).
- og-render `src/views.rs`, `ribbons.rs`, `light2d.rs`, `water.rs`,
  `skin.rs`, `custom.rs`, `particles_gpu.rs`, `probe_select.rs` (doc),
  `shaders.rs`.
- og-runtime `extract.rs` and `crates/og-api/api.ogt`: untouched, so no
  `og api gen`.
- og-tests `src/light_scenes.rs`, `src/probe_scenes.rs`,
  `src/target_scenes.rs`, `tests/light.rs`, `tests/probes.rs`,
  `tests/targets.rs`.
- docs/design.md §2.6 (Views, Reflection probes, Area lights) and the
  og-game skill §23 and the probes section.

## Left out, and why

- **3D fluids in probe captures.** The screen-space fluid is four passes
  over screen-sized targets (`fluid.depth`, `fluid.zbuf`, `fluid.thick`,
  two blurs) plus a composite that reads the scene copy and depth. A face
  would need all five targets at face size, six times per capture, and a
  face depth prepass. The water's shortcut (deep colour over the
  environment) does not apply: the fluid's surface is the depth targets
  themselves. It is the next probe item if wanted; the skill says 3D
  fluids are left out of captures.
- **Faces have no depth of their own.** Water there has no refraction or
  edge foam, and soft particles there are hard. Giving each face a depth
  prepass, as views now have, would pay for both.
- **The view glow is the glow alone,** not the main frame's bloom chain:
  its far halo is weaker (176 against 190 at 140 px).
- **The low quality preset and rect shadows.** The row's decision says the
  low preset turns rect shadows off. The engine has no quality preset to
  hang that on. `GraphicsQuality` is a field of the UI kit's
  `GameSettings`, which the settings screen answers to the game, and
  nothing in og-runtime or og-render reads it. A game turns
  `RectLight.castShadow` off itself. Nothing was changed for it.

## For the coordinator to decide

- Whether the engine should own what `GraphicsQuality` does (a
  `RenderSettings.quality` the runtime maps to rect shadows, volumetric
  method, shadow sizes and the like), or whether the skill should tell
  games to map `low` to `castShadow: false` themselves. The second is a
  skill line; the first is an api.ogt change and its own stream.
- Whether probe captures should draw 3D fluids (see Left out).
