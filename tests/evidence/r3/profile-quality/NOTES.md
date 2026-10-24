# Round 3 `profile-quality`: evidence

Branch `r3/profile-quality`, cut from origin/main 5cc1275d. It pays the
`profile-quality` row of docs/phase-1.md ("Plan 2026-10-24, round 3"),
which joins `publish-2d-refusal` (from the size stream,
`tests/evidence/debt2/size/NOTES.md` §Left out) and `quality-stream`
(from render-views-2, `tests/evidence/debt2/render-views-2/NOTES.md`: the
low preset promised to turn rect shadows off and nothing read it).

Everything ran in an orb on lavapipe (Mesa llvmpipe, `vulkan-cpu`, debug
build). Following AGENTS.md, nothing here ran `scripts/check.sh`,
`og test` or web-reference. The loop was `cargo check`, `cargo clippy
--all-targets -D warnings` and `cargo fmt` per crate, and the tests of
the crates edited, one at a time. `OG_GATE=1 sh
packages/engine/scripts/build.sh` was run once for the dev addon. With no
gate wasm in the orb it also built every web module (`wasm` profile).
The 2D publish module was then built three more times for its size
(§Size).

## What landed

| # | Item | Commits | Test that fails on the old code |
|---|------|---------|------------------------------|
| 1 | 2D publish refusal | 86d1185 og-cli, 1d2113c og-runtime/og-physics, fddcd6f docs | og-cli `cli.rs` `a_2d_publish_build_refuses_the_3d_world_by_scene_and_line`; og-runtime `tests/profile2d.rs` `the_terrain_and_its_height_fields_answer_the_profile` |
| 2 | GraphicsQuality read by the engine | eaf5c1a og-render, cf9ba9f api + og-runtime, 0bd7c33 UI kit + docs | og-tests `light.rs` `the_low_quality_preset_turns_rect_light_shadows_off`; og-runtime `tests/render_quality.rs`; packages/engine `ui-levels.test.ts` "picking a graphics preset applies it" |

### Item 1: the 2D publish refusal

- `og build --publish` and `og run --device`, for a project of
  `profile "2d"`, read every scene under `assets/` (nested entities
  included) before anything is cooked or exported. They refuse
  `Terrain`, `TerrainScatter`, `Water`, `WaterSurface` and `Fluid3D`,
  one line per hit: `  <scene file>:<line>: <Component> (<what>)`. The
  refusal names `RenderCapabilities.world3d`
  (`crates/og-cli/src/publish.rs` `refuse_3d_world`). A 3D project, a 2D
  dev build, and a 2D project without those components build as before.
- og-runtime under `profile2d` builds `terrain_absent.rs` in place of
  `terrain.rs`, and does not build `terrain_stream.rs`. Every
  `og.terrain.*` member answers "terrain is not built into this module
  ... RenderCapabilities.world3d is false". og-physics gains
  `PhysicsSystem::refuse_heightfields`, so a `SHAPE_HEIGHTFIELD` collider
  and `set_heightfield` are refused with the same reason under either
  physics profile.
- Old code: the og-cli test fails at cli.rs:479. The build exits 0
  instead of 1: it cooked and exported. The profile2d test fails because
  `og_terrain_load` answers ok. Both were run against origin/main's src
  files checked out in place.

### Item 2: GraphicsQuality

- og-render `RenderQuality` (`crates/og-render/src/quality.rs`,
  `RenderScene.quality`) holds what a frame may cost: `rect_shadows`,
  `shadowed_lights`, `shadow_map_scale`, `volumetric_fog`,
  `soft_particles`, `ssr`, `ambient_occlusion`, `msaa_samples`.
  `QualityPreset::{Low, Medium, High}` gives a row of it. The renderer
  reads them every frame. A new shadow-map scale remakes the cascade and
  local maps. Where the device resolved MSAA, a new `msaa_samples`
  switches MSAA and FXAA: every pipeline drawing into the scene target is
  a `Sampled` (one pipeline per sample count), and the pipeline caches
  are cleared and remade at the new count. The default is the full frame,
  as before.
- api.ogt: RenderSettings gains `rectShadows`, `shadowedLights`,
  `shadowMapScale`, `volumetricFog`, `softParticles`,
  `screenSpaceReflections`, `ambientOcclusion` and `msaaSamples`, plus
  `quality: QualityPreset` (none, low, medium, high), which records the
  preset last applied. `og.render.applyQuality(world, GraphicsQuality)`
  writes the preset's row through the same declared-write path as a
  game's write, so inside a system RenderSettings must be among its
  writes. The table is in applyQuality's doc. og-idl's wiki now keeps
  consecutive `|` doc lines together as one table, and the wiki renders
  it. `GraphicsQuality` moved from the ui module to render.
- og-runtime's extract maps the fields one to one onto
  `RenderScene.quality`. `quality` is only a record: the renderer never
  reads it, so a field set after a preset is drawn as set. The probe's
  `get:og.RenderSettings` is what the renderer is handed (render_quality
  test).
- The switches only take away and live in RenderSettings, not
  PostEffects. Low turns SSR and AO off without destroying the game's
  PostEffects.ssr/.ao settings, so applying medium afterwards restores
  them. This is a reading of the row's "RenderSettings and PostEffects
  fields": the coordinator may want it the other way (§Decisions).
- UI kit: `SettingsDesc.world` (new, optional). When the player picks
  another preset, the settings screen calls
  `og.render.applyQuality(world, preset)` before `onChange`, and throws
  the refusal if there is one. Without `world` the game applies it.
- Tests and how each fails on the old code:
  - og-tests light `the_low_quality_preset_turns_rect_light_shadows_off`
    reads the post's foot in `rect_shadow_scene(true)`: unshadowed 114.0,
    medium 8.2, low 114.0. On the old code the field does not exist. With
    the renderer's read removed (frame.rs filter forced to cast), low
    reads 10.2 and the test fails at light.rs:360.
  - og-runtime `render_quality.rs`: the rows of the table written out
    independently, compared with the probe's `get` and with the
    `RenderQuality` a recording backend is handed, plus a field set after
    the preset and an unknown preset refused. On the old code there is no
    `og_render_apply_quality` and no such fields (the test does not
    build).
  - packages/engine `ui-levels.test.ts` "picking a graphics preset
    applies it to the world's RenderSettings": taps Low and expects
    `["low", false, 2, 1, false]`. With the screen's call removed it reads
    `["none", true, 8, 4, true]` and fails.

## Frames read

- **No frame moves at the default quality.** 27 og-tests scenes (msaa,
  fxaa, taa, gltf-reference, sprites-array, text-msdf, particles,
  trails-beams, water-3d, water-2d, fluid-2d, fluid-3d, shader-hooks,
  ground-grid, sky-only, shadows-local-8, point-shadows-paraboloid,
  sprites-shadow-terrain, materials, ao-gtao, atmosphere-fog, decals,
  skinning, lights-256, stylised, fx-2d, baked-cornell) were each
  rendered with and without `Antialias::Msaa4`: 54 frames at this
  branch's og-render and at 5cc1275's on lavapipe. Every pair compared
  with `magick compare -metric AE` at 0 pixels. This was a scratch test,
  not committed.
- **Switching at run time.** 19 of those scenes on one renderer with
  MSAA4: medium, low, high, medium. `RenderInfo.antialias` went
  Msaa4 → Fxaa → Msaa4 → Msaa4 and the shadow map 2048 → 1024 → 4096 →
  2048. The last medium frame matched the first at 0 differing pixels,
  with no wgpu validation error. Scratch test, not committed.
- **`og run --commands`** in templates/fx-showcase (the dev addon via
  OG_NATIVE_ADDON), volumetric station, screenshot at 1.5 s, 1280x720:
  - `set:og.RenderSettings.volumetricFog=false` moves 99 % of the pixels
    (mean grey 178.9 → 228.0). Read: the lit medium and its shafts are
    replaced by flat pale plain fog, and nothing breaks.
  - The whole low row set field by field: the same, plus 5.7 % of pixels
    against fog-off alone (half-size shadow maps, FXAA).
  - `rectShadows=false` moves nothing: the station's panel casts no
    shadow. With `castShadow` set on the panel it still moves nothing in
    that view, so the rect-shadow frame is read in the og-tests gfx test
    above.
  - The probe's `get:og.RenderSettings` in each run showed the fields
    set.
  The side-by-side of medium and low was looked at in the orb and not
  committed.
- No reference frame was re-recorded. No new reference is needed: the new
  gfx test compares renders of its own.

## Size (2D publish module, `wasm` profile, wasm-bindgen + scripts/wasm-opt.sh)

Built in this orb at each revision's `crates/`, with the same three
steps build.sh runs for `web-publish-2d`: cargo, `--profile wasm`,
features `publish,physics2d,profile2d,shaping`, `--target-dir
target/wasm-publish2d`. Brotli is `brotli -q 11`.

| Revision | Raw bytes | Brotli bytes | Change |
|---|---:|---:|---|
| 5cc1275 (origin/main) | 12,070,097 | 3,263,472 | |
| fddcd6f (item 1) | 11,856,043 | 3,213,016 | −214,054 raw, −50,456 brotli |
| 0bd7c33 (items 1 and 2) | 11,869,203 | 3,215,219 | +13,160 raw, +2,203 brotli (RenderQuality, `Sampled`, applyQuality) |

The 2D module is 200,894 B raw and 48,253 B brotli smaller in all. The
terrain's share is larger than the size NOTES' estimate of 56 kB raw. That
estimate counted og-runtime's terrain alone. Here terrain_stream, the
terrain's extract and draw layouts, and whatever they kept reachable also
go. No per-crate (twiggy) split was made, so how the drop divides between
crates is not measured. These are
the shipping profile's numbers, not the size gate's `wasm-gate`
profile's.

## Shared files touched

- `crates/og-api/api.ogt` (RenderSettings' fields, GraphicsQuality moved
  to render, QualityPreset, applyQuality, SettingsDesc.world, the
  world3d and terrain docs). Generated outputs from `og api gen` and
  `og api gen --wiki`, `og api check` OK: og-api `generated/host.rs`,
  both hosts' `generated/glue.rs`, `packages/engine/src/generated/abi.ts`,
  `types/index.d.ts`, `types/bindings.json`, `schema/api.schema.json` and
  every wiki page. Most wiki pages moved only their quoted api.ogt line
  numbers.
- `crates/og-idl/src/gen/wiki.rs` (table rows in docs).
- og-render: deferred.rs (`Sampled`), lib.rs, frame.rs, particles_gpu.rs,
  scene.rs, quality.rs (new), and `set_samples` in water, metaballs,
  fluid3d, lines, terrain, scatter, sprite_array and lightmap.
- og-runtime: api.rs, extract.rs, terrain_absent.rs, modules.rs,
  engine.rs, lib.rs and Cargo.toml. og-physics: lib.rs and sim.rs. og-cli:
  publish.rs, bundle.rs, device.rs. og-host-node and og-host-web
  host.rs.
- packages/engine: src/runtime.ts (`render.applyQuality`), src/index.ts
  (the `QualityPreset` column factory, which api-surface.test.ts asks
  for), src/ui-kit/settings.ts, tests/ui-levels.test.ts.
- Skills: og-game SKILL §2, §11 (terrain) and §16 (quality); og-build
  SKILL §6; og-ui SKILL. docs/design.md §7, docs/web-export.md §Variants.

## Left out, and for the coordinator

- **Decision: where the switches live.** SSR and AO are allowed or
  forbidden by RenderSettings fields, not by writing PostEffects.ssr/.ao.
  Writing PostEffects would lose the game's settings at low, and medium
  could not restore them.
- **Decision: SettingsDesc.world.** The settings screen applies the preset
  only when given a world, since the UI kit has no world of its own. A
  saved preset is applied by the game at start (og-game §16 shows it).
- High is medium with the shadow maps at twice their side. The renderer
  offers nothing else of cost above medium today: TAA's history and the
  volumetric resolution are fixed at attach (`GameDesc.render`), and
  making them run-time fields is a stream of its own.
- `msaaSamples` switches only where the device resolved MSAA (WebGL2, or
  `antialias: "msaa4"`). On TAA, FXAA or off it is carried but changes
  nothing. Its doc says so.
- Metal references owed: none. The new gfx test reads pixels against its
  own renders, and no committed reference moved on lavapipe.
- `cargo clippy -p og-render --features profile2d --all-targets` fails on
  `tests/profile2d.rs:32` (`assert!(!WORLD_3D)`,
  clippy::assertions_on_constants). That file comes from the size stream
  and this branch does not touch it.
