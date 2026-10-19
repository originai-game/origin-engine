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
| `RenderScene` | Resources with handles (`add_mesh`, `add_texture`, `add_material`, `add_font` → handles; `set_material` replaces one), and the per-frame public fields `instances`, `camera`, `light`, `lights`, `environment`, `post`, `sprites`, `texts`, `overlay`, `debug`. `mesh_handle(index)` turns a mesh index back into its handle (the runtime's skinned copies). Resources are append-only; the renderer uploads what is new or changed. |
| `MeshData`, `Vertex`, `TextureData`, `Material` | As before: triangle lists (a vertex's `tangent` in glTF's frame, generated at upload when `w` is 0: `MeshData::generate_tangents`); RGBA8 with `color_space` and `mips`; glTF metallic-roughness with `alpha_mode` (`Opaque`, `Mask`). |
| `Instance` | `(mesh, material, transform)`, submitted one by one; batched by (mesh, material). |
| `Camera` | `position`, `target`, `up`, `projection` (`Perspective { fov_y_degrees }`, `Orthographic { height }`), `near`, `far`, `exposure`. `Camera::orthographic_2d(center, height)` is the 2D camera (looking down −z at the z = 0 plane). |
| `Exposure` | `Manual { ev }` (×2^ev; `Exposure::linear(k)` from a multiplier) or `Auto { compensation, min_ev, max_ev }`: the frame's log-average luminance brought to 0.18, from the current frame only, so the same frame always gets the same exposure. |
| `DirectionalLight` | `direction_to_light`, `color`, `intensity`, `cast_shadows`, `shadow_bounds` (`Auto`: the instances' bounds; `Sphere`), `max_shadow_distance` (default 200 m; shadows fade over its last tenth). Three cascades. |
| `Light`, `LightKind` | Point and spot lights (`Point`; `Spot { direction, angle_degrees, penumbra }`) with `position`, `color`, `intensity` (radiance at 1 m), `range`, `cast_shadows`. Clustered (see [Forward+](#forward-phase-2)): lights whose range misses the view are culled (`FrameStats::lights_culled`), up to `MAX_LIGHTS` = 256 in view are drawn (past it the brightest at the camera are kept, the rest counted in `lights_dropped`); the first `MAX_SHADOWED_LIGHTS` = 2 in view with `cast_shadows` get shadows. |
| `PostEffects`, `Bloom`, `Tonemap` | `bloom: Option<Bloom { threshold, knee, intensity }>` (threshold in exposed units: 1.0 is display white), `tonemap: Aces` (default) or `None` (clamp: authored 2D colours come out exact). |
| `Sprite` | A textured quad in the world xy plane facing +z: `texture` (`None` = solid), `position`, `size`, `anchor`, `rotation`, `color` (linear RGBA), `uv_rect` (v down; a flipped region swaps u0/u1 or v0/v1), `layer`, `clip` (an index into the overlay's clip rectangles; ignored in the world). Sorted by (layer, sort key, texture); within those, submission order. |
| `SpriteOrder`, `TextOutline`, `TextureFilter` | Per-frame side lists by index, so `Sprite` and `Text` keep their fields: `sprite_order` gives a sprite a sort key within its layer (top-down y sorting: `-y` of the feet; default 0), `text_outlines` an outline (width as a fraction of half the font's distance range, linear colour) to a world text; `set_texture_filter(texture, Nearest)` samples a texture by its nearest texel wherever sprites, tiles and particles draw it (pixel art). |
| `TileLayer`, `TileChunk`, `TileQuad`, `TileAnimation` | `tile_layers`: chunked tile quads (shared `Arc<Vec<TileChunk>>`, so an unchanged layer copies no tiles) drawn through the sprite batch before the sprites of their layer, chunks culled to an orthographic view, with `repeat` per axis (scrolling backgrounds) and animated quads cycling uvs by `time`. `FrameStats::tiles` / `tile_chunks` count what was drawn. |
| `nine_slice(sprite, insets, texture_size, world_per_pixel)` | The nine sprites of a nine-slice panel: corners keep their size, edges stretch along one axis, the centre along both; insets clamp to half the sprite. |
| `MsdfFont`, `Glyph`, `Text` | An MSDF atlas (`TextureHandle`, linear, no mips) plus `distance_range`, `line_height` and the glyph table (`advance`, `plane_bounds` in em y up, `atlas_bounds` in pixels y down: msdf-atlas-gen's layout). `Text { font, text, position, size, rotation, color, layer, clip }` lays out left to right with `\n`; missing characters are counted in `FrameStats::missing_glyphs`. Text sorts with the sprites. |
| `Overlay` | The screen-space layer drawn last (`RenderScene::overlay: Option<Overlay>`): `sprites` and `texts` in CSS pixels, origin top left, y down; `width`, `height` (the CSS viewport) and `scale` (device pixels per CSS pixel; 0 fits the height); `clips` (`add_clip([x, y, w, h])` → the index a sprite or text names in `clip`). See [The overlay](#the-overlay). |
| `DebugView` | `None`, `Cascades` (red/green/blue per cascade, black beyond). |
| `FrameStats` | `draw_calls` (every pass), `batches`, `instances`, `culled`, `triangles` (main pass), `shadow_passes`, `shadow_casters`, `lights`, `lights_dropped`, `lights_culled`, `cluster_lights`, `cluster_lights_dropped`, `sprites`, `glyphs`, `sprite_draws`, `missing_glyphs` (overlay ones included), `overlay_sprites`, `overlay_glyphs`, `overlay_draws`, `frame`. |
| `RenderInfo` | Adapter, backend, device type, `Tier` (`Full`, `NoCompute`), `antialias`, `backend_tier()`: the reference-frame directory name and the tier the probe's hello, `og.render.capabilities` and `og polish` report (`tier_name(backend, device_type, adapter)`: `vulkan-cpu` for a software rasterizer, `is_software_adapter`: a CPU device type, or an adapter named llvmpipe, lavapipe or SwiftShader). |
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
   culled to), the point and spot lights of the fragment's cluster
   (Forward+, below; inverse square windowed to the range, smooth spot
   cone), split-sum IBL,
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
   (WebGPU canvases are `Bgra8Unorm`) by the shader (`fs_srgb_encode`). A
   surface takes an sRGB format where one is offered (WebGL2, native
   windows).
10. **Overlay** (when `scene.overlay` is set; `overlay.rs`, `overlay.wgsl`):
    after the graph, straight into the output or its sRGB view (see below).

## Forward+ (phase 2)

Point and spot lights are clustered (`src/clusters.rs`, owed row 30 of
`docs/phase-1.md`). The view is cut into `CLUSTER_GRID` = 16 × 9 screen
tiles × 24 depth slices (logarithmic between the camera's near and far
planes; even for an orthographic camera). Each frame the CPU culls the
lights whose sphere (`range`) misses the view frustum, keeps up to
`MAX_LIGHTS` = 256 (past it the ones with the most radiance at the camera:
intensity times the colour's brightest channel over the squared distance,
at least 1 m), and assigns each to the clusters its sphere can touch: the
view-space box of the sphere, clipped to the near plane and projected,
widened by a pixel for the TAA jitter, and the slices of its depth range.
A cluster holds at most `MAX_CLUSTER_LIGHTS` = 64; past it (and on WebGL2
past `UNIFORM_LIGHT_REFS` = 16,384 references in all) the busiest
clusters keep their most important lights. The lit shader finds its
cluster from the pixel and the view depth and shades that cluster's lights
in scene order, so every frame of up to eight lights is byte for byte the
frame of the old per-fragment loop (every reference passes at RMSE 0), and
a frame of 243 lights is the frame of the same lights brute-forced over
every pixel (RMSE 0) in a tenth of the time.

The tables are three buffers in the frame group (bindings 7 to 9: the light
records, one word per cluster with its first reference and count, and the
references as bytes): storage buffers on the full tier, uniform buffers of
at most 16 KiB on WebGL2. A tile table texture, as first planned for
WebGL2, would need a seventeenth texture slot in the skinned lit pipeline
(below). The build runs on the CPU on both tiers: tens of microseconds for
256 lights (`tests/evidence/P2/render/forward-plus.md`), less than a
compute dispatch would cost. Stats: `lights` (drawn), `lights_culled`,
`lights_dropped`, `cluster_lights` (references stored),
`cluster_lights_dropped`.

## The overlay

The screen-space layer for HUDs and og-ui. It is drawn after every post
effect (tone mapping, the LUT, vignette, aberration, grain, bloom, TAA), so
an authored colour comes out exactly (linear in, sRGB out), with no depth
test and straight alpha blending in linear light on every tier: an sRGB
output encodes in hardware, and a non-sRGB 8-bit one (the WebGPU canvas's
`Bgra8Unorm`, a `Bgra8Unorm` offscreen target) is drawn through its sRGB
view (`view_formats`, where `DownlevelFlags::VIEW_FORMATS` or
`SURFACE_VIEW_FORMATS` allow it), so translucent fills blend as on Metal.
Only a caller's non-sRGB `RenderTarget::View`, or a backend without view
reinterpretation, falls back to the shader encoding, which blends encoded
values. Positions are CSS pixels from the top-left corner, scaled by
`Overlay::scale` (0: the output's height over `Overlay::height`). Sprites and
text are sorted by (layer, texture or font) into instanced runs (stable
within a run); each instance carries an optional clip rectangle tested per
pixel, so a scrolled list cuts a glyph in half instead of dropping it. Every
tier runs it (render passes only); screenshots, `capture` and
`read_offscreen` include it; `FrameStats` counts `overlay_sprites`,
`overlay_glyphs` and `overlay_draws`. Tests: `tests/gfx.rs`
`the_overlay_is_drawn_after_post_untonemapped_and_clipped`,
`the_overlay_blends_in_linear_light_on_a_non_srgb_output`, the `overlay`
reference scene (the showcase under TAA with a HUD; `metal` and
`vulkan-cpu`), and fx-showcase's HUD and og.ui badge on `webgpu` and
`webgl2` (`make web-reference`).

## From TypeScript

Every E1 effect is declared from a game through the IDL (`crates/og-api/api.ogt`,
module `render`), on both hosts, off by default:

| og-render | TypeScript |
|---|---|
| AO, outlines, depth of field, motion blur, god rays, glow, LUT grading, vignette, aberration, grain, flash, shake | resource `og.PostEffects` (zero = off; `lut` is a handle from `og.load.lut`) |
| `Lighting2d`, `Light2d`, `Occluder2d` | resource `og.Lighting2D`; components `og.Light2D`, `og.Occluder2D`; `SpriteDesc.castShadow` |
| `Decal` | component `og.Decal` with the atlas in `og.RenderSettings.decalAtlas` |
| `ParticleSystem` | component `og.ParticleSystem` naming an emitter from `og.load.emitter` (`.ogt` of type `particles`, cooked by og-cook) |
| `Trail`, `Beam`, `Ribbon` | components `og.Trail`, `og.Beam`; `og.render.ribbon(world, desc)` per tick |
| `SpriteFx` | `SpriteDesc` `flash`, `flashColor`, `glow`, `blend` |
| Toon, rim, dissolve, flash, outline per material | `StandardMaterialDesc` fields, set with `og.render.standardMaterial` / `updateMaterial` |
| Skinning and blendshapes | `og.Skeleton` + `og.AnimationPlayer`, `og.Blendshapes` (the runtime keeps skinned copies of a model's meshes) |
| `Overlay` | `SpriteDesc.screen` / `TextDesc.screen` with an optional `clip` `[x, y, w, h]`; og-ui draws into it |

`templates/fx-showcase` shows each of them on five stations with references
on `metal`, `vulkan-cpu`, `webgpu` and `webgl2` (the last two from headless
Chrome, `make web-reference`; each matches `metal` within og-tests'
tolerance; `tests/evidence/E2/`).

**Capture** (`capture.rs`): the offscreen target copied to a buffer and
mapped. One async function on both hosts: natively it polls the device; on
the web the mapping completes through WebGPU's `mapAsync` or, on WebGL2, on
the next submit or `Renderer::poll()`. It reads only the offscreen target, so
a page needs no `preserveDrawingBuffer`; a probe capture renders a frame to
`Offscreen` first. It runs in Chrome on WebGPU and WebGL2: every frame
`make web-reference` compares is a probe capture.

## Backend tiers

| Tier | Backends | Reference directory | Antialiasing (`Tier`) |
|---|---|---|---|
| `Full` | Metal, Vulkan (GPU or lavapipe), WebGPU | `metal`, `vulkan`, `vulkan-cpu`, `webgpu` | TAA |
| `NoCompute` | WebGL2 (wgpu GL), native GL | `webgl2`, `gl` | 4× MSAA |

Both tiers request `Limits::downlevel_webgl2_defaults()` at the adapter's
resolution; both tiers run the same passes (the full tier adds GPU
particles), and the frame group's light tables are storage buffers on the
full tier and uniform buffers on the no-compute one. The
WebGL2 constraints stay designed in: shadow maps are `R32Float` colour
arrays read with `textureLoad` and compared explicitly (WebGL2 can neither
filter, `textureLod` nor gather a depth texture), the HDR target is
`Rgba16Float` (`EXT_color_buffer_float`), IBL, mips, metering and bloom are
render passes, there is no `first_instance`, and the uniform arrays stay far
below the 16 KB WebGL2 uniform block limit (the frame uniform is 1.7 KB).
Three constraints of wgpu's GL backend, each of which once broke
fx-showcase on WebGL2:

- GLSL has no `textureLoad` from a depth texture: TAA binds the depth
  buffer as an unfilterable float texture (WebGPU allows a depth format
  there) and reads `.r`, on every backend.
- A pipeline has 16 texture slots across all its bind groups, not per
  stage: the lit skinned pipeline uses all 16 (frame 5, material 8, fx 2,
  skin 1), so the sky cube is the sky pass's own group and on the texture
  path the morph deltas and the joint palette share one texture.
- The texture target is chosen at creation, and a square 2D texture with a
  multiple of six layers becomes a cube map (array): the local shadow map
  (twelve layers) gets a spare thirteenth to stay a 2D array.

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
| `lights-256` | Night over a plaza lit by 256 point and spot lights (Forward+; 243 in view). | all |
| `atmosphere-moon` | The night colonnade with a waxing gibbous moon in view. | all |
| `materials` | Every M6 material feature: metalness and roughness rows, emissive, normal maps, clearcoat, alpha mask and blend, double-sided, unlit, a non-uniformly scaled ellipsoid. | all |
| `skinned-hooks` | Skinned cylinders drawn with a material shader that ripples, bands and lifts them; their shadows and AO follow. | all |

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

## The 2D batch (E4)

Everything world-space 2D goes through one sorted list per frame: sprites,
2D particles and glyphs keyed by (layer, sort key, source), then tile
layers and layered trails (`trail_layers: (Trail::id, layer)`) placed
before the sprites of their layer (tiles first), adjacent runs of one
source merged into one instanced draw. A layered trail is drawn with the
ribbon pipeline inside the sprite order (unlayered trails, ribbons and
beams still draw with the 3D scene), so a 2D sprite's trail shows over a
tile background. Glyph instances carry an outline in their otherwise
unused flash colour and glow strength; `fs_msdf` grows the shape by the
width and mixes the fill over the outline (width 0 is the old shader's
output exactly). With every new list empty the frame is the pre-E4 frame:
every gfx and effects reference passes unchanged.

Tests: `tests/sprites2d.rs` (sort keys across textures within a layer,
layers still win; tile culling, tiles under their layer's sprites, repeat
filling the view's edge, an animated quad on its second frame; nearest
against linear sampling; nine-slice borders at their texel size; an
outline only when asked, the fill kept; a trail hidden under a layer-0
sprite without a layer and shown over it on layer 1), each broken once
(`tests/evidence/E4/break-and-restore.md`).

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
  on the full tier; on WebGL2 one `Rgba32Float` vertex texture, the morph
  deltas in its first rows and the palette after them),
  `MorphTargets` weighted per instance; og-anim writes the palettes.
- **Stylised**: toon ramp, rim light, dissolve per material. Every
  fragment entry point has one `discard` (`dissolve_at` returns a flag the
  caller folds in): on Metal a second discard in an entry point did not take
  effect, so dissolved pixels drew black
  (`dissolved_pixels_leave_no_trace_in_the_prepass`).
- **Graph declarations**: an effect pass declares every prepass texture it
  binds (`lindepth`, `gbuffer`), not only its own input; with depth of
  field, motion blur and TAA together the graph had aliased a target a pass
  wrote with one it read (`every_effect_pass_together_under_taa`).

References for every effect scene on `metal` and `vulkan-cpu`
(`og-tests/references/*/{ao-gtao,ao-ssao,decals,dof-low,dof-high,fx-2d,
god-rays,motion-blur,particles,particles-cpu,post-grade,skinning,stylised,
trails-beams}.png`); features-off references are byte-identical to the
pre-E1 ones. Costs: `tests/evidence/E1/vfx-feature-costs.md` (`og-gfx
--costs`).

## E5: atmosphere, water, fluids, material shaders

All off by default: a scene that sets none of them draws exactly the frame
it drew before (every earlier reference is unchanged).

- **Fog** (`RenderScene::fog`, `src/atmosphere.rs`): distance fog
  (exponential from `start`, or a linear ramp to `end`), integrated
  exponential height fog, forward scattering towards `sun_color`, applied
  in the lit, sky, particle, strip and world-sprite shaders (blended
  effects fade into it by `fog_keep`); the overlay is never fogged.
- **Sky** (`SkyLook` on `ProceduralSky::look`, `Background::Procedural`):
  zenith, horizon and ground colours, sun disc and glow, hashed stars, a
  moon disc (phase 2, owed row 28: `moon_size`, `moon_phase` 0 new to 0.5
  full, `moon_intensity`, `moon_color`, `moon_direction`, zero meaning
  opposite the sun; lit from the right while waxing, darker maria, a faint
  earthshine and halo, hiding the stars behind it, set below the horizon;
  off at size 0, so every earlier frame is unchanged), drawn per pixel so a
  moving sun moves smoothly; the image-based light is
  generated from the same look (`IblGenerator` keeps its pipelines and
  textures, so regenerating it for a day-night step allocates nothing).
  `day_night(hours, tilt, azimuth)` gives the sun or moon light, the sky,
  the ambient and a fog tint as a pure function of the clock.
- **Weather** (`weather(kind, …)`, `src/weather.rs`): rain, snow, dust and
  mist as an emitter placed over the camera (`WeatherLook::origin`), a fog
  and a dimming of the light.
- **Water** (`RenderScene::water`, `src/water.rs`, `shaders/water.wgsl`): a
  grid displaced by an optional height field, travelling detail normals,
  colour by the water's depth over the scene (the depth prepass, which a
  frame with water always runs), foam where it meets geometry and on steep
  crests, refraction of a copy of the scene, reflections by a march through
  the depth prepass on the full tier and the prefiltered sky cubemap where
  that misses and on WebGL2. The main pass splits after the sky: the
  resolved scene is copied to `scene-copy` (`hdr` gains `COPY_SRC` only in
  frames with water), then water, transparents, particles, strips and
  sprites draw over it.
- **2D water** (`RenderScene::water_2d`): a side-on body and its lip as
  strips in the sprite order.
- **Metaballs** (`RenderScene::metaballs`, `src/metaballs.rs`): a fluid's
  particles added as kernels into one channel of a field target before the
  main pass (up to `MAX_FLUIDS`), composited at the fluid's sprite layer by a
  threshold with an antialiased edge and a rim.
- **Material shaders** (`MaterialShader`, `src/hooks.rs`): a WGSL snippet
  defining any of `vertex_offset`, `surface_color`, `surface_normal`,
  `surface_emissive`, with up to eight declared parameters read from the
  material uniform; `RenderScene::add_shader`, `set_material_shader`,
  `set_shader_param`. A material with a shader draws with its own lit,
  shadow and prepass pipelines, static and skinned (phase 2, owed row 23:
  shadows and the depth prepass are displaced by `vertex_offset` and cut
  by `surface_color`'s alpha; the shadow pass's uniform carries the clock
  for `h.time`), every other material with the built-in identity hooks.
  `lit_module_source` is the module og-cook compiles with naga for every
  back end.
- **Mesh replacement** (`RenderScene::set_mesh`): cloth and other generated
  geometry, uploaded again in place when the sizes match.

References on `metal` and `vulkan-cpu`: `og-tests/references/*/{atmosphere-fog,
atmosphere-dusk,atmosphere-night,water-3d,water-2d,fluid-2d,shader-hooks,
weather-rain,weather-snow}.png`, with behaviour tests in
`og-tests/tests/atmosphere.rs`; the moon's `atmosphere-moon` (a waxing
gibbous moon over the night colonnade) with `og-tests/tests/moon.rs`
(phases, size, the horizon). Costs: `tests/evidence/E5/feature-costs.md`.

## Phase 2 materials

Owed row 31 (`og-tests/tests/materials.rs`, the `materials` reference):

- **Normals under a non-uniform scale** go through the inverse transpose of
  the model's upper 3×3 (its cofactor matrix, flipped for a mirroring
  transform), for static and skinned meshes and the shadow hooks alike.
- **Vertex tangents.** `Vertex::tangent` (`xyz` along +u, `w` the
  bitangent's sign) in glTF's frame: the bitangent cross(normal, tangent) ×
  w points up the image (towards −v), so a normal map's red tilts towards
  +u and its green up the image. It comes from the file (the glTF bridge
  reads `TANGENT`), from the cook (MikkTSpace on v flipped, so cooked and
  exported tangents agree), or is generated at upload for a mesh without
  (`MeshData::generate_tangents`, Lengyel's method). The old frame from
  screen derivatives is gone: in a y-down framebuffer it had turned both
  tangent axes round, so every normal map read inverted.
- **Clearcoat** (`StandardMaterial::clearcoat`, `clearcoatRoughness`, three.js
  `MeshPhysicalMaterial` names): Filament's clear coat, a GGX lobe with 4 %
  reflectance and Kelemen's visibility on the surface's normal before the
  normal map, over the base it dims by its Fresnel, for the lights and the
  image-based light. No clearcoat maps: WebGL2's skinned lit pipeline has no
  texture slot left.

## Compressed textures at runtime

Owed row 32. The renderer still does not transcode: hosts hand it a
`TextureSource` (`RenderScene::add_texture_source`), RGBA8 level 0 with mips
built on the GPU, or every level pre-encoded in one of
`RenderInfo::texture_formats`. og-runtime's loaders pass the attached
backend's formats to the host's texture hook, so cooked Basis textures now
reach the GPU as BC7/BC5 on desktop GPUs, ASTC 4x4 on Apple and most Android,
ETC2/EAC RG11 elsewhere (og-cook's README §KTX2 says which kind gets which).
`FrameStats::texture_bytes` is the GPU memory of the scene's textures, every
level as allocated. Measured on the template's textures:
`tests/evidence/P2/render/compressed-textures.md`.

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

- **Render graph**: a depth prepass for every frame (it runs only when an
  effect reads it).
- **GL-tier references**: the og-tests scenes (`msaa` among them) have no
  `gl` / `webgl2` references: native GL needs wgpu's
  `gles`/`angle` features (Cargo.lock change, ANGLE dylibs on macOS) and was
  not set up, and og-tests' scenes load assets from disk and read back
  synchronously, so a browser run would need a wasm harness of its own.
  The browser runner (`make web-reference`) covers `templates/fx-showcase`
  on `webgpu` and `webgl2`, which draws every E1 effect, skinning and the
  overlay.
- **Materials**: clearcoat, sheen, transmission and iridescence maps.
- Shadows: cascade blending at split boundaries, PCSS/soft shadows, point
  shadows by tier (dual-paraboloid on WebGL2 if cube faces cost too much),
  more than two shadowed local lights.
- TAA: Catmull-Rom history sampling, a sharpening pass; FXAA on the GL tier
  as an MSAA alternative.
- Post: AgX, temporal exposure adaptation (deterministic per tick), fog.
- 2D: per-texture filtering (nearest for pixel art), billboards, kerning and
  RTL shaping (rustybuzz), CJK fallback atlases.
- Instance upload is the whole buffer every frame; dirty tracking, GPU
  culling on the full tier. Resources cannot be removed. No anisotropy.
  Shaders are WGSL compiled by naga at pipeline creation; cooking per
  backend at build time is still to do.
