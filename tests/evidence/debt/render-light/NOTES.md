# Debt: render-light (the depth stream S7's "Left out")

Branch `debt/render-light`, cut from origin/main 89af43d4. The design is
docs/design.md §2.6; the game-facing text is the og-game skill §24. Every
check below ran in an orb on lavapipe (Mesa 22.3.6, llvmpipe LLVM 15,
`vulkan-cpu`). Per AGENTS.md, no gate, `og test`, web-reference, wasm or
release build was run.

## What landed

| # | Item | Commit | Test that fails on the old code |
|---|------|--------|---------------------------------|
| 1 | Every loaded scene's lightmap draws | 00afb450 | og-runtime `tests/baked_scenes.rs` `two_baked_scenes_both_draw_their_bakes`; og-tests light `two_baked_scenes_draw_their_own_pages` |
| 2 | Lightmap and light-volume terms in every lit batch | db55f2dc | og-tests light `every_batch_kind_draws_its_lightmap`, `baked_batches_matches_its_reference` (new scene) |
| 7 | Render snapshot/restore with a bake | 4a2ae481 (+ clippy cb0b940a, 7549386e, aeb6400d) | og-runtime `a_render_snapshot_restores_the_bake_with_its_world`; og-tests light `a_renderer_restore_keeps_the_baked_frame` |
| 4 | Cookie cap exposed | b7f7bc51 | og-render area `cookies_past_the_cap_are_counted_and_named`; og-runtime `tests/cookie_cap.rs` |
| 6 | Froxel streaks | 8729d57b | og-tests light `the_volumetric_fog_has_no_froxel_streaks` |
| 5 | Local lights in the WebGL2 raymarch | f074b0b1 | og-render shaders `the_gl_volumetric_raymarch_reads_the_local_lights`; og-tests light `the_raymarch_lights_the_fog_with_the_spot` |
| 3a | Rect lights on terrain, scatter and fluids (LTC) | fa79a1c7 | og-tests light `terrain_and_scatter_light_rect_lights_as_areas` |
| 3b | Rect-light shadows | ae0ecd35 | og-tests light `a_shadowed_rect_light_casts_a_soft_shadow` |

1. **Many scenes' bakes.** og-render's `BakedLighting` is now
   `{ meshes: HashMap<usize, BakedMesh { pages, uvs }>, volumes }`. Each
   mesh carries its own scene's page set, and lightmap.rs keeps one bind
   group per page set (one per `Arc`). The main pass binds each batch's own
   pages at group 3, so batches no longer share one global atlas. In
   og-runtime, baked.rs keeps every loaded scene's `DrawnBake` and merges
   them each frame. Light volumes from all scenes are concatenated. A bake
   whose static models are all despawned is dropped, and
   `og.render.bakedLighting` says so ("its static models were
   despawned"). Test: `baked_cornell_pair_scene` loads two Cornell boxes,
   the right one mirrored with its own bake. The right box's block reads
   39,73,30 (the green wall's bounce on its side). Given the left box's
   pages instead (the old single-scene behaviour), it reads 93,28,25.
2. **Every batch kind draws its lightmap.** `lightmap::Family` covers
   Standard, Compact, Array, Physical, Shader and PhysicalShader, through
   new shaders `lightmap_pages.wgsl` and `lightmap_physical.wgsl`, plus
   `fs_lightmap_array` and `vs_lightmap_compact`. The physical `fs` became
   `physical_fragment`, so the lightmap variant wraps it. Light volumes are
   sampled in the same paths. The WebGL2 test now counts texture units per
   pipeline: 13 shared, plus 1 palette or 1 page set. New native-only gfx
   scene `baked-batches`: the Cornell box with a physical ceiling, a
   compact back wall, an array-map tall block and a material-shader short
   block. Each part must match `baked-cornell` within 3/255. On lavapipe
   the three non-physical parts are byte-identical. With main's renderer
   they miss:
   - the back wall reads 84,94,113 against 88,90,76;
   - the tall block's side reads 103,54,55 against 102,44,43;
   - the shader block reads 75,70,72 against 67,60,58.
7. **Snapshot/restore.** The renderer test draws a baked frame, saves the
   state, draws unbaked frames from elsewhere, then restores. The frame
   after restore is byte-identical. The runtime test snapshots a world
   with a baked scene, despawns it, and restores; the bake is drawn again.
   Main's runtime reports "despawned, no bake drawn".
4. **Cookie cap.** `og_render::cookie_layers` / `CookieLayers` counts the
   layers the cookies take and which lights are left out past
   `MAX_COOKIE_LAYERS` (24). The runtime's extract reports through the
   existing warning channel, naming the layers asked for and the lights
   drawn without cookies. The IDL gains `RenderCapabilities.maxLightCookies`
   (24). Design §2.6 and skill §24 document it.
6. **Froxel streaks: root cause.** The `inject` pass sampled one depth per
   froxel. With slices growing with the square of their index, far slices
   are metres deep, so a shadow edge inside a slice is either all in or all
   out per column. That is the vertical streaking. Confirmation: raising
   the grid to 512 slices removes the streaks
   (`frames/volumetric-fog-512-slices.png`). Fix: each froxel integrates 8
   stratified, jittered samples over its slice's depth, using interleaved
   gradient noise seeded by `vol.grid.w`, at the same 64 slices. Measured
   streak residual: columns x 60..440, y 200..500, mean absolute
   difference between neighbouring column means minus their trend, limit
   0.1. Main reads 0.346; now it reads 0.017.
5. **WebGL2 raymarch lights.** `locals_in` takes a per-method bound:
   `RAYMARCH_LIGHTS` = 8 (the first eight of the pixel's cluster, the
   brightest first), and 64 for the froxel grid. The IDL gains
   `RenderCapabilities.volumetricLights`. The `VolumetricFog` docs say
   which lights each method uses. There is a new
   `RenderDesc.volumetric: Option<VolumetricMethod>`, reported through
   `RenderInfo.volumetric` and `volumetric_lights`; it forces the raymarch
   on the full tier. In og-tests, `base_desc` and og-host-node select it
   with `OG_GFX_VOLUMETRIC=raymarch`, so the WebGL2 path is read on
   lavapipe. The raymarch module is also validated on the full tier's
   shader list. Spot cone region 1040,120,1090,250 reads:
   - raymarch now: 99,89,78;
   - froxel grid: 100,89,79;
   - raymarch on main: 76,73,69 (sun only).
3a. **Rect lights on terrain, scatter, fluids.** `surface_local_light` in
   area.wgsl (LTC rect lights, point and spot cookies, rect shadows) is
   now the one local-light function of `terrain_lit.wgsl`, `scatter.wgsl`
   and `fluid3d.wgsl`. Test: `rect_meadow_scene` places a panel facing
   left over the island terrain, with and without grass. Lit side against
   behind:
   - terrain: 31.4 against 0.7;
   - grass: 26.5 against 1.1;
   - main's point-light treatment: 11.8 against 10.4.
3b. **Rect-light shadows: implemented.**
   - `RectLight.castShadow` (IDL, read in og-runtime `baked.rs`
     `rect_light`) gives the light one layer of the local shadow array, or
     two when two-sided. The layers come from the shadowed lights' budget
     (both layer counts in lib.rs).
   - The shadow view is a perspective from the rect's centre along its
     normal, `RECT_SHADOW_FOV` = 150°, with the half-height as up. It
     stores distance / range, like every local shadow.
   - area.wgsl `rect_shadow` reads it as PCSS. A 16-tap blocker search
     covers the light's footprint; then 24 bilinear PCF taps cover a disc
     of radius half-size × (receiver − blocker) / blocker. Each pixel's
     golden-angle spiral is turned by a PCG hash of its world position.
   - It applies in pbr.wgsl's rect branch (respecting `receives`),
     `area_light` (terrain, scatter, fluids), physical.wgsl
     `physical_local`, and volumetric.wgsl `locals_in` (fog shafts behind a
     panel's casters).
   - The emissive fixture slab at the light's centre lies inside the
     near plane (`local_near` ≥ 0.02 m) and does not shadow its own light.
   - Test: `rect_shadow_scene` has a 1.6 m panel 3 m up, 1.2 m left of a
     2 m post. Floor luma at the post's foot and near the shadow's tip:

     | Run | Foot | Tip |
     |-----|------|-----|
     | unshadowed | 114.0 | 58.0 |
     | shadowed | 8.2 | 30.8 |

     The test asserts foot < 0.35× unshadowed (main ignores `cast_shadows`
     on rect lights). It also asserts the tip's ratio is above the foot's
     by more than 0.2, so a hard shadow from the centre fails as well.

## Frames read on lavapipe

- `frames/volumetric-fog-before.png` (main): the froxel fog shows tall
  vertical light/dark bands through the slot shafts and the spot cone,
  each a froxel column wide; column residual 0.346.
- `frames/volumetric-fog-after.png`: the shafts are continuous wedges with
  smooth edges, and the spot cone is a smooth gradient; residual 0.017.
  Against the Metal reference: RMSE 0.0138, max diff 191 at 0.028% of
  pixels.
- `frames/volumetric-fog-512-slices.png`: the diagnostic run. Streaks are
  gone with one sample per froxel at 512 slices, which confirmed the
  depth-sampling cause.
- `frames/volumetric-fog-raymarch-before.png` (main, raymarch forced): the
  slot shafts are there, the spot cone has no fog (76,73,69).
- `frames/volumetric-fog-raymarch-after.png`: the spot's cone glows in the
  fog as on the froxel tier (99,89,78 against 100,89,79).
- `frames/rect-shadow.png` / `rect-shadow-off.png`: the warm panel's pool
  on the grey floor and the post.
  - With the shadow, the post throws a shadow to the right: near black and
    sharp at its foot, widening into a soft cone that fades out by x ≈ 2.4.
  - The grain from the rotated taps is visible only at 1:1 zoom.
  - A first version with an unrotated 12-tap spiral drew the post as
    streaky copies across the penumbra; per-pixel rotation fixed it.
  - A second version used the full size as the disc radius and drew a
    shadow too pale at contact (73 against 114); the half-size radius
    fixed it.
- the Cornell pair (`baked_cornell_pair_scene`, test-only), `baked-batches`, and the meadow scenes
  were read through their tests' region means, quoted above.

## Shared files touched (sibling streams beware)

- og-render: `src/lib.rs`, `src/frame.rs`, `src/scene.rs`,
  `src/shaders.rs`, `src/terrain.rs`, `shaders/pbr.wgsl`,
  `shaders/pbr_array.wgsl`, `shaders/physical.wgsl`,
  `shaders/terrain_lit.wgsl`, `shaders/scatter.wgsl`,
  `shaders/fluid3d.wgsl`, `tests/webgl2.rs`, `tests/shaders.rs`. The
  lighting files are lightmap.rs, area.rs, volumetric.rs, area.wgsl,
  lightmap*.wgsl and volumetric*.wgsl. `pbr_fs.wgsl` and `material*` are
  untouched.
- og-runtime: `src/extract.rs`, `src/api.rs`, `src/baked.rs`.
- og-host-node: `src/lib.rs` (`OG_GFX_VOLUMETRIC`).
- og-api: `api.ogt`, which adds `RenderCapabilities.maxLightCookies`,
  `RenderCapabilities.volumetricLights` and `RectLight.castShadow` and
  updates the VolumetricFog and RectLight docs. The generated outputs
  (`host.rs`, `index.d.ts`, `bindings.json`, `api.schema.json`) and the
  whole wiki were regenerated; line numbers moved in every wiki page.
- og-tests: `src/scenes.rs` (`baked-batches` registered, `base_desc`
  volumetric override), `src/light_scenes.rs`, `tests/light.rs`.
- docs/design.md §2.6; packages/knowledge/skills/og-game/SKILL.md §24.

## Metal references the coordinator must re-record

- `volumetric-fog` (froxel; item 6 changes every fog pixel slightly):
  `volumetric_fog_matches_its_reference` fails on lavapipe (RMSE 0.0138)
  until it is re-recorded.
- `baked-batches` (new, native only): it has no reference yet. Read its
  physical ceiling on Metal; `every_batch_kind_draws_its_lightmap` covers
  it there.
- `baked-cornell` is unchanged in intent; re-check it, since the bind path
  moved to per-batch page groups.
- The WebGL2 tier references of `volumetric-fog` (the raymarch now
  includes the spot): web-reference.
- Every fx-showcase station frame, in particular the baked and volumetric
  stations (both bake paths and fog changed), plus any station with a
  terrain, scatter or fluid under a rect light.
- `rect-lights` is unchanged (its lights do not cast). The rect-shadow
  scene is test-only (region means), with no reference.

## Left out / for the coordinator

- `custom_lit.wgsl` (a full custom shader, S8's file) still lights rect
  lights as a point and without their shadow. It would need its local
  light loop to call `surface_local_light`. This is left out because that
  file belongs to the material stream.
- Rect shadows are a centre approximation (PCSS from one view). A caster
  close to a large panel is shadowed as from its centre, with the
  penumbra widened by the panel's half-size rather than by its true view
  of each part of the panel. Receivers past 75° from the normal are
  unshadowed. At 512 texels over 150°, a texel is about 5 cm at 3.3 m, so
  thin casters under a high panel lose detail. The exact answer is the
  bake (`og bake lightmaps` traces the rectangle). An exact real-time
  shadow would need several shadow views across the panel (e.g. 4 corner
  views, so 4× the layers) or ray-traced visibility.
- On lavapipe (Mesa 22.3.6), every physical-material pipeline crashes the
  driver's draw thread (SIGSEGV). Main's physical tests crash the same
  way. So `baked_batches_matches_its_reference` and
  `every_batch_kind_draws_its_lightmap` SIGSEGV here and must be read on
  Metal. `water_phase_2_scenes_match_their_references` (`water-underwater`)
  also fails on main here, which is unrelated.
- Decide: whether `RAYMARCH_LIGHTS` = 8 is the right WebGL2 bound (cost
  per pixel: 24 steps × 8 lights), and whether rect shadows should be
  allowed on WebGL2 (they run there now: 16 + 24×4 texel loads per
  shadowed rect per pixel).
