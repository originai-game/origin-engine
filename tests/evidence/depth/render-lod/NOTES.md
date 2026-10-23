# S6 render-lod: evidence and notes for the coordinator

Branch `depth/render-lod`, rebased on main `c5aa7ff1` (depth waves 1 and 2).
Every frame here was rendered on lavapipe (`vulkan-cpu`, llvmpipe 15) in the
orb and read before it was committed. No gate, `og test`, web-reference, or
wasm/release build was run.

## What landed

| Commit | What |
|---|---|
| `8f4cc791` | IDL (appended to `module render`): `RenderSettings.lodBias`, `og.Model.lodBias`, `og.ReflectionProbe` (size, resolution 16..1024, mode baked/realtime, interval, intensity, blendDistance, visible), `ReflectionProbeMode` and its column, `MAX_REFLECTION_PROBES` 8, `PostEffects.ssr/ssrMaxDistance/ssrThickness/ssrSteps/ssrFade/ssrIntensity`, `RenderCapabilities.ssr/reflectionProbes`, `ProbeTimings.triangles` |
| `ff21345e` | Cook: `lods 1 0.5 0.25` by default with meshoptimizer, `lods`/`lodError` import settings, glTF `MSFT_lod` levels kept, and flat meshes simplified over welded positions and re-faceted (`crates/og-cook/tests/lod.rs`) |
| `48e9271a` | Runtime LOD: `og-render/src/lod.rs` (projected size, `select` with 15 % hysteresis), per-entity level in og-runtime `extract.rs`, `debugView lod` tint, `dump_entity` `render.lod`, `ProbeTimings.triangles` |
| `e0882bc9`, `0a83f610` | fx-showcase `lod` (2,000 trees) and `reflections` stations, the tree model and its generator, the `dump_render` probe dump. `debug:lod` is a variant of S1's DebugView (`VIEWS` in og-runtime `geometry.rs`). |
| `abaa0e1c` | Reflection probes (`og-render/src/probes.rs`): six 90° captures in a `reflection-probes` graph pass, copied, downsampled and GGX-prefiltered (`ibl.rs` `probe_filter`, `ibl.wgsl` `fs_probe_copy`), and sampled box-projected and blended in `pbr_fs.wgsl`/`physical.wgsl` through `frame.wgsl`'s `reflected_light`/`ambient_light` |
| `05a4c71a` | og.ReflectionProbe extraction (`og-runtime/src/probes.rs`), baked captures loaded from the cook cache by key, the `bake_probes` dump, og-cook `cook/probes.rs` (a baked probe is an `og.EnvironmentMap` blob in `cooked/probes/`, kind `probe` in the manifest, kept by the cook while its file exists), and `RenderCapabilities.ssr`/`reflectionProbes` |
| `2e8dec47` | SSR on the full tier (`og-render/src/ssr.rs`, `shaders/ssr.wgsl`): a `FramePass::Ssr` after Main into `hdr.ssr`. The prepass writes gloss and metalness into lindepth's second channel (now RG32F). |
| `e6fab3fc` | `og bake probes [project] [--commands …] [--at S] [--size WxH]` (og-cli `bake_probes.rs`, one dispatch line and USAGE in `main.rs`) |
| `f5c6b8f1` | gfx scenes and tests: `lod-levels`, `probe-interior`, `ssr-wet-floor` (native scenes) |
| `97be2f1c` | `docs/design.md` §2.6 and og-game skill §22 (old §22 is now §23) |

## The triangle counts (acceptance: at least 3× lower with LOD on)

The fx-showcase `lod` station (2,000 trees of 3,160 triangles, levels
3160/1580/790; `og cook --report` lists `"lods": [3160, 1580, 790]`) at tick
30, 1280×720, from `dump_render` (`ProbeTimings.triangles`):

| | main-pass triangles | models per level |
|---|---|---|
| LOD on (`lod-on.json`) | 1,300,812 | [0, 20, 1982] |
| `RenderSettings.lodBias = 0` (`lod-off.json`) | 5,150,788 | [2002] |

That is 3.96× fewer triangles with LOD on. `dump_entity` of `tree-1990`
reported `render.lod` 2 with LOD on (earlier session) and 0 after
`lodBias = 0` (`tree-1990.json`).

## Frames read

- `lod-on.png`: the forest field under the sun, with shadows and no holes or
  popping artefacts in the coarse trees.
- `lod-debug.png` (`debug:lod`): the nearest rows are yellow (level 1) and
  every row behind them orange (level 2). None is green, because no tree
  reaches 1.2× of the view height.
- `lod-off.png`: the same field at full detail, which looks the same at
  this distance.
- `reflections-probe-off.png`: the probe is invisible. The chrome sphere
  reflects the sky (blue above, the sky's brown ground below) and the wet
  floor reflects a sky gradient: the wrong reflection for a closed room.
- `reflections-probe-on.png`: with the probe, the chrome sphere shows the
  room: the blue left wall, the white ceiling, the gold sphere and the
  green block. The floor reflects the red back wall. **The chrome sphere
  shows the room, not the sky** (acceptance).
- `reflections-ssr-on.png` / `reflections-ssr-off.png`: with SSR the wet
  floor also mirrors the chrome sphere, the gold sphere and the lamp-lit
  panel at their places. Without SSR the floor keeps the probe's
  reflection.
- `reflections-baked-from-cache.png` (after `og bake probes templates/fx-showcase
  --commands "0:station:reflections"` wrote `cooked/probes/556ec96cab9f2c9b.ogb`):
  matches the lazily drawn frame, and `reflections-baked-dump-render.json`
  reports `bakedFromCache: true`.
- `gfx-lod-levels-lavapipe.png`: three trees at 6, 16 and 45 m tinted
  green, yellow and orange. 5,532 triangles drawn; three full trees would
  be 9,480.
- `gfx-probe-interior-lavapipe.png` (and `…-without-probe-…`): the chrome
  sphere reflects the blue wall, the ceiling and the green block with the
  probe, and the sky without it.
- `gfx-ssr-wet-floor-lavapipe.png`: the red sphere and the white panel
  mirrored in the floor under a black sky.

## gfx tests (`crates/og-tests/tests/gfx.rs`) and the Metal references owed

These checks pass on lavapipe (the second check of each scene renders a
control frame):

- `lod_levels_pick_coarser_meshes_with_distance`: levels [0, 1, 2], and
  under 65 % of three full trees' triangles.
- `a_reflection_probe_puts_the_room_in_the_chrome_sphere`: the sphere's
  lower half measures (134, 140, 156) with the probe and (128, 119, 108)
  without it.
- `ssr_reflects_the_sphere_in_the_floor`: the floor under the sphere
  measures (55, 31, 30) with SSR and (31, 31, 31) without.

These reference tests are owed on Metal, to be recorded by the coordinator
with `OG_GFX_CAPTURE=<scene>`:

- `lod_levels_matches_its_reference`: `lod-levels.png`
- `probe_interior_matches_its_reference`: `probe-interior.png`
- `ssr_wet_floor_matches_its_reference`: `ssr-wet-floor.png` (full tier
  only)

All three scenes are in `NATIVE_SCENES`: they use og-cook, and the scene
module is native-only.

## Shared files touched (to reconcile with S8 and S7)

- `crates/og-api/api.ogt`: appended to `module render` (one block after
  `RenderCapabilities`), plus fields on `RenderSettings`, `og.Model`,
  `PostEffects`, `ProbeTimings` and S1's `DebugView` (variant `lod`). The
  generated outputs (`packages/engine/types`, `schema`, wiki) were
  regenerated. Most of the wiki diff is `api.ogt line N` references
  shifting, and those pages will conflict with any other IDL stream:
  regenerate them after the merge rather than merging them.
- og-render:
  - `lib.rs`: `mod probes`, `mod ssr` and their re-exports; a
    `probes: probes::Probes` field; the probes' prepare, assignment and
    culling; `FramePass::Probes` and its encode arm; and the frame groups
    per probe. `draw_list` takes one more argument (a probe binding), and
    each of its four call sites changed. The `ssr` decision is in the
    prepass trigger and the pass list. `Renderer::read_reflection_probe`
    is new.
  - `post.rs` (frame graph): `FramePass::Probes` and `FramePass::Ssr`, the
    `hdr.ssr` transient, `LitNames.base` (depth bands read after SSR), the
    `fs_ssr` effect, and `FxPostUniform.ssr`. **`LINDEPTH_FORMAT` is now
    `Rg32Float`**: its second channel is the reflectivity. Every pipeline
    writing lindepth (prepass, terrain, scatter, fluid3d) takes the format
    from that constant.
  - `scene.rs`: `RenderScene::reflection_probes` and `PostEffects::ssr`.
  - `frame.rs`/`frame.wgsl`: `FrameUniform.probe` (three rows at the end),
    plus `probe_weight`, `probe_direction`, `ambient_light` and
    `reflected_light`.
  - Lit WGSL: `pbr_fs.wgsl` and `physical.wgsl` sample IBL through those
    helpers. Their cluster loop takes an every-light path when
    `frame.cluster.x == 0` (a probe face has no clusters).
  - `pbr_prepass.wgsl` (depth.y), `fxpost.wgsl` (the `ssr` rows),
    `ibl.wgsl` (`fs_probe_copy`), `ibl.rs` (`probe_filter`,
    `record_probe_filter`; `cube_texture` is `pub(crate)`), and `lod.rs`.
- og-cook: `cook/model.rs` (levels), `cook/mod.rs` (`MeshOptions.lods`,
  `for_model`, `pub mod probes`, probe assets kept through a cook),
  `cook/environment.rs` (`environment_blob` split out for reuse),
  `cook/probes.rs`, `cook/vox.rs`, and `runtime.rs` (`mesh_compact`,
  `add_lod_levels`).
- og-runtime: `extract.rs`, `engine.rs`, `probe.rs`, `probes.rs`,
  `geometry.rs`, `assets.rs`, `fx.rs`, `api.rs`, `lib.rs`.
- og-probe: `probe.rs` and `hooks.rs` (`timings.triangles` and the
  `entity_render` hook).
- og-host-node and og-host-web `lib.rs` (`triangles`, plus
  `read_reflection_probe` on node).
- `packages/engine/src/index.ts` (`ReflectionProbeMode`,
  `MAX_REFLECTION_PROBES`).
- og-cli `main.rs` (one `mod`, one dispatch line, the USAGE text).
- og-tests `scenes.rs` (`NATIVE_SCENES` and three `build` arms) and
  `lib.rs`.

## Left out, or different from the letter of §6

- **Probes skip transparent and skinned meshes and effects.** A capture
  draws the opaque lit meshes and the sky. Skinned meshes, terrain,
  scatter and transparent meshes are not in it. Reflections
  inside a probe are the environment's, one bounce.
- **One probe per batch.** A batch (one mesh and material) takes the
  smallest probe holding the centre of all its instances' bounds, not one
  probe per instance. Two rooms that share a mesh and material and each
  have a probe sample whichever box holds the batch's centre. Probe boxes
  are axis-aligned (the entity's rotation is ignored, as the IDL says).
- **Not probe-sampled.** Skinned, terrain, scatter, water and fluid draws
  keep the environment.
- **SSR reads the material factors only.** The prepass writes the
  roughness and metalness factors, not their maps. Terrain, scatter and
  unlit surfaces write no reflectivity and reflect nothing.
- **The bake key** hashes position (to the mm), box and face size, not
  the room's content. Re-run `og bake probes` after changing a room (or
  delete `cooked/probes/`).
- **No station playtest with `expect` steps was added.** The acceptance
  checks are the gfx tests and the scripted runs above.
- **The WebGL2 path was not run in a browser** (web-reference is not this
  stream's to run). Probes use no texture slot, and SSR is gated on
  `Tier::Full` in the renderer and in `RenderCapabilities`.
  `og-render/tests/webgl2.rs` and `shaders.rs` (naga validation of every
  module) pass.
