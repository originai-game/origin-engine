# debt3/quality-presets: what high buys, and the MSAA the device gives

Base `f87c89d8` (origin/main). Branch `debt/quality-presets`. Clears the
`profile-quality` row's "Left:" in `docs/phase-1.md`: high was medium with
the shadow maps at twice their side and nothing else, and `msaaSamples` 4
was silently drawn at 1 where the device had not resolved MSAA.

## Commits

| Commit | What | Test (fails on the old code) |
|---|---|---|
| `5673bfb9` | og-render: `RenderQuality.ao_resolution`, `ssr_step_scale`, `volumetric_step_scale`; the presets' rows; og-gfx `OG_GFX_QUALITY` | og-tests `the_high_preset_refines_ao_reflections_and_fog_and_medium_moves_nothing` (fails with high's old row: "high draws the scene's AO at full resolution"); og-render `quality::tests` (2) |
| `f7883786` | og-render: `RenderInfo::msaa_samples` replaces the private `multisampled` flag and caps `RenderQuality::msaa_samples` | og-tests `the_device_msaa_caps_the_qualitys` |
| `fc28771e` | api.ogt: `RenderSettings.aoResolution` (`AoResolutionSetting`), `ssrStepScale`, `volumetricStepScale`, `RenderCapabilities.maxMsaaSamples`, the applyQuality table and docs; og-runtime reads and writes them; regenerated outputs; og-game skill; CHANGELOG | og-runtime `render_quality.rs`: the table test with the new fields, `the_capabilities_say_what_msaa_the_device_gives` |
| (this commit) | og-render README, docs/design.md, docs/phase-1.md, these notes | — |

## The preset table now

| Field | low | medium | high | run-time cost of a change |
|---|---|---|---|---|
| rectShadows | false | true | true | none |
| shadowedLights | 2 | 8 | 8 | none; capped by `maxShadowedLights` |
| shadowMapScale | 0.5 | 1 | 2 | shadow maps remade |
| volumetricFog | false | true | true | none |
| softParticles | false | true | true | none |
| screenSpaceReflections | false | true | true | none |
| ambientOcclusion | false | true | true | none |
| aoResolution (new) | half | postEffects | full | none (`ao-upsample` added or dropped by the graph) |
| ssrStepScale (new) | 0.5 | 1 | 2 | none (a uniform) |
| volumetricStepScale (new) | 0.5 | 1 | 2 | froxel volumes remade (64 → 128 slices); raymarch steps a uniform (24 → 48) |
| msaaSamples | 1 | 4 | 4 | MSAA targets and pipelines remade; capped by `maxMsaaSamples` (new) |

Medium is every field's default, and at the default every scale is the
identity (`quality::tests::medium_is_the_default_and_the_default_scales_nothing`).

### Decided

- **aoResolution's precedence.** A RenderSettings field the renderer reads
  first, never a write to PostEffects (the profile-quality row's rule for
  SSR and AO): `postEffects` (the zero value) draws
  `PostEffects.aoResolution`; `full` or `half` replaces it and PostEffects
  keeps its own. So high draws full AO even for a game that asked half,
  low half even for one that asked full, and medium restores the game's.
  A new enum `AoResolutionSetting` (resource fields cannot be optional).
- **Scales, not absolute counts,** for SSR steps and fog slices, so a
  game's `PostEffects.ssrSteps` keeps its meaning (high doubles 48 to 96,
  within the pass's 128).
- **Low's halves** only show where a game turns SSR, AO or volumetric fog
  back on after the preset (low switches all three off).
- **msaaSamples follows shadowedLights.** The field keeps the row's value
  (high writes 4 everywhere) and the capability caps it:
  `RenderCapabilities.maxMsaaSamples` is 4 where antialias resolved to
  msaa4 when the renderer was made, 1 for TAA, FXAA, off or FXAA for want
  of a multisampled HDR format, 0 without a renderer; it stays 4 while a
  quality of 1 draws FXAA (`antialias` then says `fxaa`).
- The settings screen (`og.ui.settingsScreen`) shows only the preset
  names (Low, Medium, High, overridable labels), no descriptions, so no
  template changed.

### Left, and why

- **Shadow filtering.** The PCF kernel (4 bilinear taps) and PCSS's 16 +
  16 taps are WGSL constants (`lighting.wgsl` `PCSS_TAPS`); a wider
  kernel for high needs the taps through a uniform or a pipeline variant:
  shader work and a web-reference pass.
- **Cascade count.** Made with the renderer (`RenderDesc::shadow_cascades`,
  3 = `MAX_CASCADES` already on the full tier), so high has none to add.
  Low taking fewer at run time would need the cascade split and frame
  uniforms re-planned per frame; not needed for this row.
- **GTAO slices and steps.** Constants in `fxpost.wgsl` (r4/ao-cost
  measured 2 slices × 4 steps with the constants patched and rejected it
  for half resolution).
- **Bloom chain depth.** `post::bloom_levels` follows the target's size
  (6 at 720p already); a deeper chain widens the glow, which changes the
  look rather than its quality.
- **Soft particles' depth.** Already a switch (`softParticles`); their
  fade distance is the emitter's own (`softDistance`), not a quality.

## Frames read on lavapipe (llvmpipe, vulkan-cpu, 1280×720)

No frame moves at the default quality:

- og-gfx captures of ao-gtao, ao-ssao, ssr-wet-floor, volumetric-fog, msaa,
  taa, showcase, gltf-reference, rect-lights and light-cookies on this
  branch against the base (stash, rebuild, recapture): **byte-identical**
  (`cmp`), all ten.
- `volumetric_fog_matches_its_reference`, `ao_gtao_…`, `ao_ssao_…`,
  `taa_matches_its_reference_on_the_full_tier`, `antialiasing_smooths_edges`,
  `the_low_quality_preset_turns_rect_light_shadows_off`: pass.
- In the new test, medium is byte-identical to the default for all three
  scenes.

High against "shadows only" (the old high: `shadowMapScale` 2, the rest
default), from `the_high_preset_refines_…` (pixels moved by more than 2 in
a channel):

| Scene | Region (x0..x1, y0..y1) | Pixels moved there | Mean luma | Also |
|---|---|---|---|---|
| ao-gtao, the scene asking half | contacts under the larger sphere and the box beside it, 600..760, 440..530 | 2,488 (22,805 in the frame) | 145.53 → 145.16 | high byte-identical to the scene asking full itself |
| ssr-wet-floor, 48 steps | the emissive slab's reflection in the floor, 745..885, 405..665 | 642 (1,457 in the frame) | 171.13 → 172.37 | high byte-identical to the scene asking 96 steps itself |
| volumetric-fog | the fog in front of the dark pier between two shafts, 640..720, 120..520 | 17,777 (74,350 in the frame) | 23.73 → 21.07 | medium → high: 19,978 there, 24.13 → 21.07 |

Read: AO's contacts sharpen (the half-resolution upsample's softness at
the sphere's rim and the box's foot goes; the full-res noise is finer);
the slab's reflected edges close where 48 steps stepped over them (the
reflection's bottom edge and sides fill in); the fog stops leaking the
shafts' light across the dark pier (with slices half as deep, a
shaft's edge in depth no longer smears into the slice beside it), the
pier reading darker. `fog-slot-medium-high.png` is that
crop, medium left, high right.

### og run --commands

fx-showcase's volumetric station (the room with the window, the rect
panel, the cookie spot and lantern), native host on lavapipe, dev addon:

```
og run templates/fx-showcase --size 1280x720 --commands "0.3:get:og.RenderSettings;0.5:station:volumetric;1.50:screenshot:medium.png;1.7:quit"
og run templates/fx-showcase --size 1280x720 --commands "0.3:set:og.RenderSettings.shadowMapScale=2;0.31:set:og.RenderSettings.aoResolution=full;0.32:set:og.RenderSettings.ssrStepScale=2;0.33:set:og.RenderSettings.volumetricStepScale=2;0.4:get:og.RenderSettings;0.5:station:volumetric;1.50:screenshot:high.png;1.7:quit"
```

(applyQuality is a game call, so the commands write high's row field by
field; the get reads `aoResolution "full", ssrStepScale 2,
volumetricStepScale 2, shadowMapScale 2, msaaSamples 4` back, medium's
`aoResolution "postEffects"`.) 27,254 pixels differ by more than 1 %
(of 921,600). Read: the window's patch on the floor has crisper bar
edges (the cascades at 4096, AE 1,220 in 660..940, 540..600, luma 214.8 →
215.1); the fog on the back wall loses the faint vertical banding of 64
slices (200×220 at 760, 200: luma 198.5 → 198.4); the lantern's glow
on the left wall tightens (120×80 at 40, 170: 182.1 → 181.5).
`volumetric-station-medium-high.png` is the room's middle, medium left,
high right. The world hash differs between the runs only because
RenderSettings differs.

## GPU cost of high (lavapipe, relative only)

Per-pass GPU timestamps (`Renderer::set_clock`, `gpu_timings`), mean of
10 frames after 3, 1280×720, ms (a scratch test, not committed):

| Scene | medium | shadows only (old high) | high | the passes that grow |
|---|---|---|---|---|
| ao-gtao (asking half) | 38.1 | 55.8 | 63.5 | `ao` + `ao-blur` + `ao-upsample` 12.7 → `ao` + `ao-blur` 23.6; cascades 6.7 → 20.3 |
| ssr-wet-floor | 33.3 | 44.9 | 66.8 | `ssr` 13.3 → 28.4; cascades 8.5 → 23.7 |
| volumetric-fog | 66.4 | 85.3 | 126.2 | `volumetric` 40.2 → 82.1; cascades 7.3 → 23.6 |
| showcase | 141.8 | 154.6 | 169.9 | cascades 18.5 → 30.2; `main` 28.1 → 33.1 |

So each refined effect costs about twice its pass, as the step counts
say; a scene without AO, SSR or volumetric fog pays only the shadow
maps. llvmpipe has a large fixed cost per pass; a GPU's ratios should be
close for the step-bound passes (SSR, the froxel inject and integrate).

## Owed to the Mac lane

- No shader changed (the froxel shaders already read the grid's depth
  and the raymarch's steps from the uniform; SSR's steps were a uniform),
  and no frame moves at the default, so **no reference frame is owed**,
  Metal or web-reference.
- Worth reading there: `OG_GFX_QUALITY=high og-gfx volumetric-fog
  ssr-wet-floor` against medium with `OG_GFX_SERIAL=1`, for the real GPU
  cost of high's froxel grid (128 slices: 2 × 14.7 MB of Rgba16Float
  volumes instead of 2 × 7.4 MB) and SSR's 96 steps.
- World hashes: RenderSettings has a new layout, so every world hashes
  differently once (simulation untouched), as the ao-resolution round's
  PostEffects did.

## Checks run here

- `cargo check -p og-render`; `cargo nextest run -p og-render` (99 passed);
  `cargo nextest run -p og-runtime` (81 passed, 5 skipped).
- og-tests, one at a time: the two new tests and the six above.
- `og api gen`, `og api check`: the generated files match.
- packages/knowledge: `bun scripts/validate.ts`, `bun test tests/` (49 pass).
- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -D
  warnings`; `cargo clippy -p og-render -p og-runtime -p og-host-web
  --target wasm32-unknown-unknown -D warnings`; og-render
  `--features profile2d` native and `profile2d,publish` wasm32: clean.
  og-runtime's `publish` feature (native, and og-host-web's publish on
  wasm32) stops on two `dead_code` errors that are on the base too
  (`AnimAssets::replay` in anim.rs, `FeelAssets::replay` in feel.rs; not
  touched here).
