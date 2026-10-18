# og-render

The Origin Engine renderer (`docs/design.md` §2.6, M3 in `docs/phase-1.md`):
wgpu, WGSL shaders in `crates/og-render/shaders/`, one code path for native
(Metal, Vulkan, headless for executors) and the browser (WebGPU, WebGL2
through wgpu's GL backend). It started as the spike's renderer; M0
reproduced the spike's frame byte for byte
(`tests/evidence/M0/spike-frame-reproduced.png`).

The crate knows nothing about the ECS. Its whole input is a `RenderScene`;
the world → scene extraction belongs to the layer above.

## Public types

| Type | What it is |
|---|---|
| `RenderDesc` | How to create a renderer: `width`, `height`, `backend` (`Auto`, `WebGpu`, `WebGl2`, `Metal`, `Vulkan`), `output_format` (`None` = `Rgba8UnormSrgb` offscreen, the surface's choice on a surface), `antialias`. Default 1280×720, `Auto`, `Antialias::Off`. |
| `Antialias` | `Off`, `Tier` (TAA on the full tier, 4× MSAA on the no-compute tier), and the explicit `Msaa4` and `Taa` (any tier). `RenderInfo::antialias` reports what was resolved; `Msaa4` falls back to `Off` where the HDR format cannot be multisampled. |
| `Renderer` | `Renderer::new(&desc).await` (headless), `Renderer::with_surface(&desc, &instance, surface).await` (presents), `render(&scene, &target) -> Result<FrameStats>`, `resize`, `info`, `reset_history` (TAA, on a camera cut), `capture().await -> RgbaImage` and `poll()` on every host. Native only: `new_blocking`, `wait_idle`, `read_offscreen()` (= `capture` blocking). |
| `render_to_rgba(&desc, &scene)` | Native, blocking: one frame headless → `RgbaImage` (RGBA8, rows top to bottom). |
| `RenderTarget` | `Offscreen` (the renderer's own texture, readable), `Surface` (acquire, draw, present), `View(&TextureView)`. |
| `RenderScene` | Resources with handles (`add_mesh`, `add_texture`, `add_material`, `add_font` → handles; `set_material` replaces one), and the per-frame public fields `instances`, `camera`, `light`, `lights`, `environment`, `post`, `sprites`, `texts`, `debug`. Resources are append-only; the renderer uploads what is new or changed. |
| `MeshData`, `Vertex`, `TextureData`, `Material` | As before: triangle lists; RGBA8 with `color_space` and `mips`; glTF metallic-roughness with `alpha_mode` (`Opaque`, `Mask`). |
| `Instance` | `(mesh, material, transform)`, submitted one by one; batched by (mesh, material). |
| `Camera` | `position`, `target`, `up`, `projection` (`Perspective { fov_y_degrees }`, `Orthographic { height }`), `near`, `far`, `exposure`. `Camera::orthographic_2d(center, height)` is the 2D camera (looking down −z at the z = 0 plane). |
| `Exposure` | `Manual { ev }` (×2^ev; `Exposure::linear(k)` from a multiplier) or `Auto { compensation, min_ev, max_ev }`: the frame's log-average luminance brought to 0.18, from the current frame only, so the same frame always gets the same exposure. |
| `DirectionalLight` | `direction_to_light`, `color`, `intensity`, `cast_shadows`, `shadow_bounds` (`Auto`: the instances' bounds; `Sphere`), `max_shadow_distance` (default 200 m; shadows fade over its last tenth). Three cascades. |
| `Light`, `LightKind` | Point and spot lights (`Point`; `Spot { direction, angle_degrees, penumbra }`) with `position`, `color`, `intensity` (radiance at 1 m), `range`, `cast_shadows`. `MAX_LIGHTS` = 8 per frame (the rest are counted in `FrameStats::lights_dropped`); the first `MAX_SHADOWED_LIGHTS` = 2 with `cast_shadows` get shadows. |
| `PostEffects`, `Bloom`, `Tonemap` | `bloom: Option<Bloom { threshold, knee, intensity }>` (threshold in exposed units: 1.0 is display white), `tonemap: Aces` (default) or `None` (clamp: authored 2D colours come out exact). |
| `Sprite` | A textured quad in the world xy plane facing +z: `texture` (`None` = solid), `position`, `size`, `anchor`, `rotation`, `color` (linear RGBA), `uv_rect` (v down), `layer`. Sorted by (layer, texture); within a layer and texture, submission order. |
| `MsdfFont`, `Glyph`, `Text` | An MSDF atlas (`TextureHandle`, linear, no mips) plus `distance_range`, `line_height` and the glyph table (`advance`, `plane_bounds` in em y up, `atlas_bounds` in pixels y down: msdf-atlas-gen's layout). `Text { font, text, position, size, rotation, color, layer }` lays out left to right with `\n`; missing characters are counted in `FrameStats::missing_glyphs`. Text sorts with the sprites. |
| `DebugView` | `None`, `Cascades` (red/green/blue per cascade, black beyond). |
| `FrameStats` | `draw_calls` (every pass), `batches`, `instances`, `culled`, `triangles` (main pass), `shadow_passes`, `shadow_casters`, `lights`, `lights_dropped`, `sprites`, `glyphs`, `sprite_draws`, `missing_glyphs`, `frame`. |
| `RenderInfo` | Adapter, backend, device type, `Tier` (`Full`, `NoCompute`), `antialias`, `backend_tier()`: the reference-frame directory name. |
| `RenderError` | `NoAdapter`, `NoDevice`, `Surface`, `InvalidHandle`, `Readback`, `Unsupported`, `InvalidData`. |
| `shapes` | `sphere`, `plane`, `cuboid(size)`, `checker`. |
| `gltf` (feature `gltf`) | `load_glb(bytes) -> GltfModel`, `GltfModel::add_to(&mut scene) -> ModelParts`, `ModelParts::instance`. A bridge until the cook (M4). |
| `wgpu` | The wgpu this crate is built on, re-exported for hosts. |

## The render graph

A fixed, feature-gated pass sequence per `render` call (not yet a graph
object; see below). Passes whose feature is off are not recorded.

1. **Sync.** New meshes, textures (with the **mip pass**: one fullscreen
   triangle per level, linear-space box filter), materials, fonts. When the
   sky changes, the **IBL passes** run (sky cube, irradiance, GGX prefilter,
   BRDF LUT; render passes only, so WebGL2 runs them).
2. **Batch and cull** (CPU). Instances stably sorted by (mesh, material);
   each sorted instance gets a world bounding sphere. Every pass gets a draw
   list per batch: the instances whose sphere touches the pass's volume. A
   batch that loses less than a tenth of its instances is drawn whole (no
   copy); otherwise its visible transforms are appended to the instance
   buffer. Draws bind a buffer slice, never `first_instance` (WebGL2).
   Sprites and glyphs are sorted by (layer, kind, texture or font) into runs.
3. **Cascaded shadows** for the directional light (`frame.rs`): splits
   between the near plane and the nearer of `max_shadow_distance` and the
   far side of `shadow_bounds`, λ = 0.8 between logarithmic and uniform.
   Each cascade is the bounding sphere of its camera slice (its size does
   not change as the camera turns), its centre snapped to the shadow map's
   texel grid in light space (texels do not swim), its depth range reaching
   towards the light to the top of the scene bounds. Casters are culled to
   the cascade's box **and** to the camera slice swept towards the light
   (the slice planes whose normal faces the light); on `gltf-reference` and
   `spheres-20k` this draws 45 % and 36 % fewer casters with identical
   frames. Three passes into a 2048² `R32Float` array (depth also written as
   colour) with a shared depth buffer, front faces culled.
4. **Local shadows**: per shadowed light (at most two), six cube faces (point)
   or one perspective view (spot), each a pass into a 512² `R32Float` array
   layer storing distance / range, casters culled to the face frustum.
5. **Main pass** into the `Rgba16Float` HDR target (4× multisampled with a
   resolve when MSAA is on): forward PBR (`pbr.wgsl`) with the directional
   light, its cascade picked by view depth (the same slices its casters were
   culled to), up to eight point/spot lights in a per-fragment loop
   (inverse square windowed to the range, smooth spot cone), split-sum IBL,
   emissive, alpha mask, derivative normal maps. Shadow lookups are bilinear
   PCF with explicit compares on `textureLoad` (the WebGL2-safe path).
   **Shadow bias works at any scale:** the normal offset and the depth bias
   are in world units proportional to the cascade's texel size (the depth
   bias then converted to that cascade's normalized depth), where the old
   single map spent a fixed bias in normalized depth over the whole scene
   (about a metre on a 400 m field). The **sky pass** draws the sky cube at
   the far plane over uncovered pixels. **Sprites and text** follow, alpha
   blended, depth-tested against the scene without writing depth: one
   instanced draw per run, quads built from the vertex index; text uses the
   MSDF median with a screen-space distance range from `fwidth`.
6. **TAA** (when on): a Halton(2,3) jitter of 8 positions is applied to the
   projection as a clip-space translation; the TAA pass reprojects the
   history through the depth buffer with the unjittered matrices (camera
   motion), clamps it to the current 3×3 neighbourhood in YCoCg and blends
   10 % of the current frame with luminance weights. History ping-pongs
   between two targets and restarts on resize, a new scene or
   `reset_history`.
7. **Metering** (auto exposure): log2 luminance into 64², then 8×8 averages
   to 8² and 1² (`textureLoad`, no float filtering), read by the bloom
   prefilter and the tonemap through `textureLoad`.
8. **Bloom**: a 13-tap prefilter with exposure and a soft-knee threshold into
   a half-resolution chain (up to 6 levels), 13-tap downsamples, then tent
   upsamples added back level by level.
9. **Tonemap**: exposure × (HDR + bloom × intensity) → ACES or clamp → the
   output. An sRGB output is encoded by the hardware; a non-sRGB one
   (WebGPU canvases are `Bgra8Unorm`) by the shader (`fs_srgb_encode`).

**Capture** (`capture.rs`): the offscreen target copied to a buffer and
mapped. One async function on both hosts: natively it polls the device; on
the web the mapping completes through WebGPU's `mapAsync` or, on WebGL2, on
the next submit or `Renderer::poll()`. It reads only the offscreen target, so
a page needs no `preserveDrawingBuffer`; a probe capture renders a frame to
`Offscreen` first. Compiled and clippy-clean for `wasm32-unknown-unknown`,
not yet run in a browser.

## Backend tiers

| Tier | Backends | Reference directory | Antialiasing (`Tier`) |
|---|---|---|---|
| `Full` | Metal, Vulkan (GPU or lavapipe), WebGPU | `metal`, `vulkan`, `vulkan-cpu`, `webgpu` | TAA |
| `NoCompute` | WebGL2 (wgpu GL), native GL | `webgl2`, `gl` | 4× MSAA |

Both tiers request `Limits::downlevel_webgl2_defaults()` at the adapter's
resolution; nothing uses compute yet, so both run the same passes. The
WebGL2 constraints stay designed in: shadow maps are `R32Float` colour
arrays read with `textureLoad` and compared explicitly (WebGL2 can neither
filter, `textureLod` nor gather a depth texture), the HDR target is
`Rgba16Float` (`EXT_color_buffer_float`), IBL, mips, metering and bloom are
render passes, there is no `first_instance`, and the uniform arrays stay far
below the 16 KB WebGL2 uniform block limit (the frame uniform is 1.7 KB).

## Testing

`crates/og-tests` is the gfx reference harness. Each named scene renders
headless at 1280×720 (after its warm-up frames, with its per-frame update)
and is compared with `crates/og-tests/references/<backend-tier>/<scene>.png`:
a pixel is an outlier when a channel differs by more than 32/255; the frame
fails above 0.5 % outliers or an RGB RMSE above 0.01. On failure the actual
frame and a diff go to `target/…/tmp/og-gfx/<tier>/`. A missing reference
fails; a scene with no reference on the adapter's tier (TAA on GL, MSAA on
the full tier) asserts the tier's antialiasing choice instead.

| Scene | What it proves | Tiers |
|---|---|---|
| `gltf-reference` | The spike's scene (trickster + dropship × 25) with CSM, sky, mips. | all |
| `spheres-20k` | 20,000 spheres submitted one by one: 2 batches, ≤ 10 draws. | all |
| `cascades-200m` | A 420 m field, pillars to 190 m, a wall at 200 m, short kerbs and blocks: shadows in every cascade. | all |
| `point-spot-lights` | Night, 7 local lights, a shadowed spot and a shadowed point light (7 shadow passes). | all |
| `taa` | Thin poles and bars after 16 TAA frames. | full |
| `msaa` | The same geometry with 4× MSAA. | no-compute |
| `bloom` | Emissive spheres with bloom. | all |
| `exposure` | A dim dusk scene metered to middle grey. | all |
| `sprites-20k` | 20,000 sprites (3 textures, one a 2×2 atlas, 2 layers), orthographic: 3 sprite draws. | all |
| `text-msdf` | Latin and CJK MSDF text from 0.32 to 4.5 world units, rotated. | all |
| `sky-only` | The sky with the sun in view: 2 draws. | all |
| `showcase` | Models, CSM, a shadowed spot and point light, bloom, auto exposure, world-space text and sprites, the tier's antialiasing. | all |

Behaviour tests in `tests/gfx.rs` read frames rather than code: the cascade
debug view puts cascade 0 at the camera's feet and cascade 2 at the horizon;
short casters (0.5 m blocks, 0.6 m kerbs) shadow pixels in every cascade of
the 400 m scene (fails with the old normalized bias); local shadows darken
the ground; MSAA and TAA turn black/white edges into partial coverage; bloom
brightens the surroundings of emitters; metered exposure gives the same
brightness for a scene lit 16× brighter (manual does not); higher sprite
layers draw on top; missing glyphs and foreign font handles are reported;
a BGRA non-sRGB output captures the same picture as RGBA sRGB; lights past
eight are counted. Each was checked by breaking the code it covers.

```sh
export CARGO_TARGET_DIR=$PWD/target/agent-render   # any target dir works
cargo test --release -p og-tests                   # gfx + the wasm32 compile check
OG_GFX_CAPTURE=1 cargo test --release -p og-tests --test gfx              # re-record this tier
OG_GFX_CAPTURE=spheres-20k cargo test --release -p og-tests --test gfx    # one scene
cargo test --release -p og-tests --test perf -- --ignored perf            # perf vs crates/og-tests/perf/
OG_PERF_RECORD=1 cargo test --release -p og-tests --test perf -- --ignored perf --test-threads=1
cargo run --release -p og-tests --bin og-gfx -- --all --frames 120        # stats, timings, RMSE
cargo run --release -p og-tests --bin og-gfx -- --costs                   # the feature-cost table
OG_GFX_DEBUG_CASCADES=1 cargo run --release -p og-tests --bin og-gfx -- cascades-200m --out c.png
```

**The `vulkan-cpu` tier** (Mesa lavapipe, what CI runs on) is recorded in the
platform's E2B probe sandbox: cross-build `og-gfx` and the gfx test binary
with `cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.31 -p
og-tests --bin og-gfx --test gfx`, then `bun
crates/og-tests/scripts/capture-vulkan-cpu.ts` (after `bun install --cwd
tools/bench` once). One sandbox run (about a minute) captures every scene,
runs the gfx tests against the captured frames (all pass, RMSE 0: lavapipe
is deterministic), times two scenes, and writes the frames to
`references/vulkan-cpu/` and the logs to `$CARGO_TARGET_DIR/vulkan-cpu/`.
The sandbox's Mesa (25.0.7, LLVM 15) may differ from CI's; the tolerance is
expected to absorb it (Metal vs lavapipe is RMSE 0.001–0.007 per scene).

**The MSDF test atlas** (`crates/og-tests/assets/fonts/noto-sans-sc-msdf.*`)
is Noto Sans SC (the platform template's OFL subset; `OFL.txt` alongside),
ASCII plus 原点引擎文字渲染, generated with
[msdf-atlas-gen](https://github.com/Chlumsky/msdf-atlas-gen) (built from
source, commit 6148900):
`msdf-atlas-gen -font NotoSansSC-Regular-subset.ttf -charset noto-sans-sc-msdf.charset -type msdf -format png -size 32 -pxrange 4 -dimensions 512 256 -yorigin top -imageout noto-sans-sc-msdf.png -csv noto-sans-sc-msdf.csv -json noto-sans-sc-msdf.json`.
`og_tests::scenes::load_font` reads the PNG, the CSV (glyphs) and two numbers
of the JSON (`distanceRange`, `lineHeight`). The cook (M4) owns producing
atlases; the renderer only takes a texture and a glyph table.

## Performance

M4 Pro, Metal, 1280×720, on a machine shared with other builds and an iOS
simulator (load average 20–60), so numbers carry ±10 % noise. Interleaved
A/B against the M0 renderer (same glam `scalar-math`), 9 rounds of 200
frames, median: `gltf-reference` 1.58 ms (M0: 1.59 ms), `spheres-20k` 6.78 ms
(M0: 8.04 ms). The new shadow pipeline costs nothing on these scenes: three
cascades draw fewer casters than M0's single map because of the swept-slice
culling, and the main pass culls to the frustum. `sprites-20k`, 20,000
sprites moved and resubmitted every frame, takes 1.0–1.3 ms (the M3 budget
is 2 ms). Per-feature costs are in `tests/evidence/M3/feature-costs.md`
(`og-gfx --costs`); perf baselines are in `crates/og-tests/perf/` per tier
and adapter, compared with 15 % tolerance.

## E1 effects

Everything a game declares on `RenderScene` as data (`src/effects.rs`,
`particles.rs`, `ribbons.rs`, `light2d.rs`, `skin.rs`); a field left at its
default records no pass and costs nothing (`effects_off_add_no_passes`).

- **Post stack**: ambient occlusion (`AoMethod::Gtao` / `Ssao`, applied to
  the ambient term), outlines (depth and normal edges, per-scene or
  per-instance colour), depth of field (`DofQuality::Low` / `High`), motion
  blur from per-object and camera motion vectors (`fx.rs` carries the
  previous view-projection), god rays, colour grading by `ColorLut`,
  vignette, chromatic aberration, film grain seeded by the tick, screen
  flash and camera shake (a deterministic offset of the camera, not a
  post pass).
- **Decals**: up to 8 projected boxes per frame, read by the lit shader
  (`fx-decals`); each is capped to its box.
- **Particles**: `ParticleEmitter` assets and `ParticleSystem`s with
  curves, bursts, noise, sprite sheets, collision against the depth buffer,
  soft fade against the scene, local or world space, 2D emitters drawn as
  sprites. Simulated by compute on the full tier and on the CPU elsewhere;
  both paths give the same particles (`gpu_and_cpu_paths_simulate_the_same_particles`)
  and a 600-tick run is deterministic on either.
- **Trails, ribbons and beams**: `Trail` keeps a per-id history for its
  `TrailStyle`'s lifetime; `Ribbon` is an explicit polyline; `Beam` joins
  two points and scrolls with the tick.
- **2D lighting**: `Lighting2d` with `Light2d`s shadowed by `Occluder2d`s
  (soft penumbrae) and an order-independent glow buffer; only the lit
  sprite layer range is affected; `SpriteFx` adds flash, glow, lit and soft
  per sprite.
- **Skinning and blendshapes**: `SkinData` + `JointPalette` (storage buffer
  on the full tier, `Rgba32Float` vertex texture on WebGL2),
  `MorphTargets` weighted per instance; og-anim writes the palettes.
- **Stylised**: toon ramp, rim light, dissolve per material.

References for every effect scene on `metal` and `vulkan-cpu`
(`og-tests/references/*/{ao-gtao,ao-ssao,decals,dof-low,dof-high,fx-2d,
god-rays,motion-blur,particles,particles-cpu,post-grade,skinning,stylised,
trails-beams}.png`); features-off references are byte-identical to the
pre-E1 ones. Costs: `tests/evidence/E1/vfx-feature-costs.md` (`og-gfx
--costs`).

## What phase 2 owes

Delivered since M3 (8d160fe, M6): the render graph object (`src/graph.rs`:
declared targets and passes, data-flow order, aliased transients; the
existing passes run on it with every reference unchanged);
`StandardMaterial` from reflected material assets with sorted transparency
(`transparent`, back to front, no depth write), double-sided and unlit
materials; pre-encoded texture uploads (BC7/BC5, ASTC 4×4, ETC2/EAC, RGBA8)
with their mips, chosen per tier and reported in
`RenderInfo::texture_formats`; `shadow_map_size`, `shadow_cascades` and
`RenderDesc::mobile()` (two 1024² cascades, ASTC/ETC2 first). Evidence:
`tests/evidence/M6/render-*`.

- **Render graph**: a depth prepass; Forward+ clustered lights (CPU tile
  table on WebGL2) beyond the 8-light uniform loop.
- **GL-tier references**: the `msaa` scene and the other ten have no `gl` /
  `webgl2` references. Native GL needs wgpu's `gles`/`angle` features
  (Cargo.lock change, ANGLE dylibs on macOS) and was not set up; a committed
  browser runner for `webgpu` and `webgl2` is still owed, and with it a run
  of `Renderer::capture` in a browser.
- **Materials**: a normal matrix for non-uniform scale (normals still go
  through the model's upper 3×3); vertex tangents (the tangent frame comes
  from screen derivatives); clearcoat. No gfx reference scene covers the M6
  material features yet (transparency, double-sided, unlit are asserted on
  pixels in `og-tests/tests/render.rs` only).
- Shadows: cascade blending at split boundaries, PCSS/soft shadows, point
  shadows by tier (dual-paraboloid on WebGL2 if cube faces cost too much),
  more than two shadowed local lights.
- TAA: Catmull-Rom history sampling, a sharpening pass; FXAA on the GL tier
  as an MSAA alternative.
- Post: AgX, temporal exposure adaptation (deterministic per tick), fog.
- 2D: per-texture filtering (nearest for pixel art), screen-space UI
  layering (sprites are world-space today; og-ui's README asks for an
  overlay pass and a per-instance clip rectangle), billboards, kerning and
  RTL shaping (rustybuzz), CJK fallback atlases.
- Instance upload is the whole buffer every frame; dirty tracking, GPU
  culling on the full tier. Resources cannot be removed. No anisotropy.
  The runtime loaders do not upload the tier's compressed format yet (the
  cook's native runtime transcodes Basis to RGBA8; see og-cook's README).
  Shaders are WGSL compiled by naga at pipeline creation; cooking per
  backend at build time is still to do.
