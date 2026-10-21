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
| `RenderDesc` | How to create a renderer: `width`, `height`, `backend` (`Auto`, `WebGpu`, `WebGl2`, `Metal`, `Vulkan`), `output_format` (`None` = `Rgba8UnormSrgb` offscreen, the surface's choice on a surface), `antialias`, `shadow_map_size`, `shadow_cascades`, `texture_compression`, `particles`, `culling`, `point_shadows` (`PointShadows::{Auto, Cube, Paraboloid}`), `taa_history` (`TaaHistory::{Bilinear, CatmullRom}`), `taa_sharpen` (0 to 1). Default 1280×720, `Auto`, `Antialias::Off`, point shadows `Auto`, a bilinear TAA history, no sharpening. |
| `Antialias` | `Off`, `Tier` (TAA on the full tier, 4× MSAA on the no-compute tier), and the explicit `Msaa4`, `Taa` and `Fxaa` (any tier). `RenderInfo::antialias` reports what was resolved; `Tier` and `Msaa4` fall back to `Fxaa` where the HDR format cannot be multisampled ([Shadows and antialiasing by tier](#shadows-and-antialiasing-by-tier)). |
| `Renderer` | `Renderer::new(&desc).await` (headless), `Renderer::with_surface(&desc, &instance, surface).await` (presents), `render(&scene, &target) -> Result<FrameStats>`, `resize`, `info`, `reset_history` (TAA, on a camera cut), `save_state() -> RendererState` and `restore_state(state)` (the adapted exposure and its tick, kept on the GPU for a world snapshot; `RendererState::id`/`from_id` carry it as a u64), `capture().await -> RgbaImage` and `poll()` on every host. Native only: `new_blocking`, `wait_idle`, `read_offscreen()` (= `capture` blocking). |
| `render_to_rgba(&desc, &scene)` | Native, blocking: one frame headless → `RgbaImage` (RGBA8, rows top to bottom). |
| `RenderTarget` | `Offscreen` (the renderer's own texture, readable), `Surface` (acquire, draw, present), `View(&TextureView)`. |
| `RenderScene` | Resources with handles (`add_mesh`, `add_texture`, `add_texture_source`, `add_material`, `add_font`, `add_terrain`, `add_terrain_stream`, `add_environment_map`, `add_texture_array`, `add_palette`, `add_lut` → handles; `set_material`, `set_terrain`, `set_terrain_stream` replace one, `set_terrain_tile` hands a streamed terrain a tile; `remove_mesh`, `remove_texture`, `remove_material`, `remove_font`, `remove_environment_map`, `remove_texture_array`, `remove_palette`, `remove_lut` free one: [Resources and removal](#resources-and-removal)), and the per-frame public fields `instances`, `camera`, `light`, `lights`, `environment`, `environment_map`, `post`, `stylise`, `sprites`, `sprite_arrays`, `texts`, `overlay`, `debug`, `terrains`, `water`, `particles`, `particle_colliders` (`ParticleCollider::{Plane, Box, Sphere, HeightField}`: what CPU-simulated particles bounce off), `tick`, `tick_rate`. `mesh_handle(index)` turns a mesh index back into its handle (the runtime's skinned copies). The renderer uploads what is new or changed and frees what was removed before the next frame. |
| `MeshData`, `Vertex`, `TextureData`, `Material` | As before: triangle lists (a vertex's `tangent` in glTF's frame, generated at upload when `w` is 0: `MeshData::generate_tangents`); RGBA8 with `color_space` and `mips`; glTF metallic-roughness with `alpha_mode` (`Opaque`, `Mask`). |
| `VertexEncoding` | How a mesh's vertices are held on the GPU: `Full` (default, `Vertex` as given, 64 bytes) or `Compact` (16 bytes where a plain material draws it; quantised): `RenderScene::set_vertex_encoding(mesh, encoding)`, `vertex_encoding(mesh)`. [Instance and vertex encodings](#instance-and-vertex-encodings). |
| `StandardMaterial`, `PhysicalExtensions` | The reflected three.js material (`MeshStandardMaterial` fields, `MeshPhysicalMaterial`'s clearcoat, sheen, transmission, iridescence, anisotropy and `ior`: [Physical materials](#physical-materials-p4)); glTF's `KHR_materials_*` extensions of one material with textures as handles or indices, `apply` lays them over a `StandardMaterial` as three.js's `GLTFLoader` does. `mapArray`/`mapLayer` and `normalMapArray`/`normalMapLayer` read the base colour and normal from texture-array layers ([Array maps](#array-maps-materials-and-terrain-layers)). |
| `Instance` | `(mesh, material, transform, material_layer, fx, cast_shadows, receive_shadows)`, submitted one by one; batched by (mesh, material, cast_shadows). `material_layer` (`None`: the material's) is the texture-array layer an array-map material samples for this instance alone: [Array maps](#array-maps-materials-and-terrain-layers). `fx` (`InstanceFx { flash, flash_color, tint }`, `InstanceFx::NONE` the material's own look; `SkinnedInstance::fx` the same) is this instance's own hit flash and tint: [Instance looks](#instance-looks-the-hit-flash-of-one-instance). `cast_shadows` (drawn into the shadow passes; a (mesh, material)'s non-casters are a batch of their own) and `receive_shadows` (darkened by the shadow maps; carried in the record, no batch of its own), both on from `add_instance` and `SkinnedInstance::new`: [Which instances receive](#shadows-and-antialiasing-by-tier). The transform is affine: the GPU holds its columns' xyz, the layer and the look, 64 bytes ([Instance and vertex encodings](#instance-and-vertex-encodings)). |
| `Camera` | `position`, `target`, `up`, `projection` (`Perspective { fov_y_degrees }`, `Orthographic { height }`), `near`, `far`, `exposure`. `Camera::orthographic_2d(center, height)` is the 2D camera (looking down −z at the z = 0 plane). |
| `Exposure` | `Manual { ev }` (×2^ev; `Exposure::linear(k)` from a multiplier) or `Auto { compensation, min_ev, max_ev, adapt_speed_up, adapt_speed_down }`: the frame's log-average luminance brought to 0.18. With both speeds 0 (`Exposure::auto()`) from the current frame only, so the same frame always gets the same exposure; otherwise adapted over the scene's ticks (see [Phase 4](#phase-4-environment-maps-agx-exposure-adaptation)). |
| `DirectionalLight` | `direction_to_light`, `color`, `intensity`, `cast_shadows`, `shadow_bounds` (`Auto`: the instances' bounds; `Sphere`), `max_shadow_distance` (default 200 m; shadows fade over its last tenth), `shadow_softness` (PCSS: the penumbra's width per metre between caster and receiver; 0, the default, is the hard edge) and `cascade_blend` (the fraction of each cascade's depth over which it fades into the next; 0 by default). Three cascades. |
| `Light`, `LightKind` | Point and spot lights (`Point`; `Spot { direction, angle_degrees, penumbra }`) with `position`, `color`, `intensity` (radiance at 1 m), `range`, `cast_shadows`. Clustered (see [Forward+](#forward-phase-2)): lights whose range misses the view are culled (`FrameStats::lights_culled`), up to `MAX_LIGHTS` = 256 in view are drawn (past it the brightest at the camera are kept, the rest counted in `lights_dropped`); of those in view with `cast_shadows`, `RenderInfo::shadowed_lights` (`MAX_SHADOWED_LIGHTS` = 8 on the full tier, `NO_COMPUTE_SHADOWED_LIGHTS` = 4 on WebGL2) get shadows, the most radiant at the camera first (the rest counted in `shadowed_lights_dropped`). |
| `PostEffects`, `Bloom`, `Tonemap` | `bloom: Option<Bloom { threshold, knee, intensity }>` (threshold in exposed units: 1.0 is display white), `tonemap: Aces` (default), `Agx` (Blender 4's and three.js's AgX fit, base look) or `None` (clamp: authored 2D colours come out exact). |
| `EnvironmentMapData`, `EnvironmentMap`, `bake` | A prefiltered environment map (level 0's face `size`, `ENVIRONMENT_MAP_LEVELS` = 5 levels of RGB9E5 `texels`, level m at roughness m/4, `irradiance` as nine harmonics) and how a frame uses one (`map`, `rotation` about +y in radians, `background_intensity`); `rgb9e5`, `rgb9e5_decode`, `sh9`, `face_direction`; feature `bake`: `bake::bake(&Equirect, &BakeDesc)` and `bake::bake_cube`, the cook's prefilter, and `bake::from_prefiltered(&PrefilteredCube, &BakeDesc)`, a cube's own prefiltered mip chain taken as it is. See [Phase 4](#phase-4-environment-maps-agx-exposure-adaptation). |
| `Sprite` | A textured quad in the world xy plane facing +z: `texture` (`None` = solid), `position`, `size`, `anchor`, `rotation`, `color` (linear RGBA), `uv_rect` (v down; a flipped region swaps u0/u1 or v0/v1), `layer`, `clip` (an index into the overlay's clip rectangles; ignored in the world). Sorted by (layer, sort key, texture); within those, submission order. |
| `SpriteArray`, `TextureArrayData`, `TextureArrayInfo` | `sprite_arrays` (and `Overlay::sprite_arrays` for screen sprites): a sprite drawn from layer `layer` of a texture array (`add_texture_array`: same-sized layers uploaded as one `D2Array`; `set_texture_array_filter` for pixel art) instead of its texture, with glow, flash, 2D lighting and blend; `ParticleEmitter::array` draws a 2D emitter's particles from one. See [The 2D batch](#the-2d-batch-e4). |
| `SpriteShadow`, `ShadowGround`, `ShadowBend`, `SPRITE_SHADOW_LIFT`, `SHADOW_BEND_ROWS`, `SHADOW_BEND_COLUMNS`, `SHADOW_BEND_TOLERANCE` | `sprite_shadows`: a world sprite's drop shadow along the sun onto its `ground` (a plane through `height` under the anchor with a `normal`; the plane y = 0 by default; `ShadowGround::fit` fits one to a terrain under the shadow, and bends it, `bend`, over a hollow or ridge the plane misses), faded where the cascades already shadow that ground, drawn before the sprites of its layer (see [The 2D batch](#the-2d-batch-e4)). |
| `SpriteOrder`, `TextOutline`, `TextureFilter` | Per-frame side lists by index, so `Sprite` and `Text` keep their fields: `sprite_order` gives a sprite a sort key within its layer (top-down y sorting: `-y` of the feet; default 0), `text_outlines` an outline (width as a fraction of half the font's distance range, linear colour) to a world text; `set_texture_filter(texture, Nearest)` samples a texture by its nearest texel wherever sprites, tiles and particles draw it (pixel art). |
| `TileLayer`, `TileChunk`, `TileQuad`, `TileAnimation` | `tile_layers`: chunked tile quads (shared `Arc<Vec<TileChunk>>`, so an unchanged layer copies no tiles) drawn through the sprite batch before the sprites of their layer, chunks culled to an orthographic view, with `repeat` per axis (scrolling backgrounds) and animated quads cycling uvs by `time`. `FrameStats::tiles` / `tile_chunks` count what was drawn. |
| `nine_slice(sprite, insets, texture_size, world_per_pixel)` | The nine sprites of a nine-slice panel: corners keep their size, edges stretch along one axis, the centre along both; insets clamp to half the sprite. |
| `MsdfFont`, `Glyph`, `Kern`, `Text`, `GlyphPlacement` | An MSDF atlas (`TextureHandle`, linear, no mips) plus `distance_range`, `line_height`, the glyph table (`advance`, `plane_bounds` in em y up, `atlas_bounds` in pixels y down: msdf-atlas-gen's layout) and `kerning` (`Kern { left, right, adjust }` em: the cook's pair table; empty for a font that does not kern). `Text { font, text, position, size, rotation, color, layer, clip, tracking, weight, glow, placements }` lays out left to right with `\n`, each glyph advancing by its advance, the kerning with the character before it and `tracking` em (CSS `letter-spacing`), world and overlay alike (`FontLayout::walk`); a shaper's `placements` (one `GlyphPlacement { advance, x, y }` em per character, as og-ui hands a bidi or shaped line) replace those and offset each glyph (marks on their bases). Missing characters take no space, break the kerning pair and are counted in `FrameStats::missing_glyphs`. `MsdfFont::measure` is a line's width as laid out. `RenderScene::set_font` replaces a font's glyph table (glyphs rendered at run time joining a page): fonts carry a revision and the renderer re-lays them out. Text sorts with the sprites. `weight` (a faux bold, CSS px of the distance field) and `glow` (`TextGlow { color, radius }`) are the overlay's (below). |
| `Overlay` | The screen-space layer drawn last (`RenderScene::overlay: Option<Overlay>`): `sprites` and `texts` in CSS pixels, origin top left, y down; `width`, `height` (the CSS viewport) and `scale` (device pixels per CSS pixel; 0 fits the height); `clips` (`add_clip([x, y, w, h])` → the index a sprite or text names in `clip`). See [The overlay](#the-overlay). |
| `TerrainData`, `TerrainControl`, `TerrainHandle` | A terrain's geometry: `columns` × `rows` heights (m) over a `size` rectangle centred on its origin, row-major from -z and -x, and an optional RGBA8 control map (one channel per layer). `height(local)` (on the triangles the renderer and physics use), `normal(local)`, `range()`, `cell()`, `check()`. See [Terrain](#phase-2-terrain). |
| `Terrain`, `TerrainLayer`, `TerrainLod` | A terrain drawn this frame (`RenderScene::terrains`): its data, its `transform` (rotation, scale and translation from the terrain's own space, centred on the height field at height 0, to the world), up to `MAX_TERRAIN_LAYERS` = 4 layers (albedo and normal maps, tint, roughness, `tile`, `triplanar`, slope and altitude rules with fades), `height_blend`, `cast_shadows`, and the clipmap's `lod` (finest `spacing`, `ring` quads per side). |
| `DebugView` | `None`, `Cascades` (red/green/blue per cascade, black beyond). |
| `FrameStats` | `draw_calls` (every pass), `batches`, `instances`, `culled`, `triangles` (main pass), `shadow_passes`, `shadow_casters`, `lights`, `lights_dropped`, `lights_culled`, `cluster_lights`, `cluster_lights_dropped`, `sprites`, `glyphs`, `sprite_draws`, `missing_glyphs` (overlay ones included), `overlay_sprites`, `overlay_glyphs`, `overlay_draws`, `texture_bytes` (textures, texture arrays, environment maps, palettes and colour LUTs on the GPU; removed ones count nothing), `frame`, `taa_history` (whether TAA blended history this frame), `shadowed_lights` and `shadowed_lights_dropped`, `particle_collision_skipped`, and phase 2's `terrain_draws` (clipmap levels in the main pass), `terrain_triangles`, `terrain_shadow_draws`, `reflection_draws` (the planar reflection pass). |
| `RenderInfo` | Adapter, backend, device type, `Tier` (`Full`, `NoCompute`), `antialias`, `shadow_map_size`, `shadow_cascades`, `shadowed_lights`, `point_shadows` (resolved: `Cube` or `Paraboloid`), `texture_formats`, `backend_tier()`: the reference-frame directory name and the tier the probe's hello, `og.render.capabilities` and `og polish` report (`tier_name(backend, device_type, adapter)`: `vulkan-cpu` for a software rasterizer, `is_software_adapter`: a CPU device type, or an adapter named llvmpipe, lavapipe or SwiftShader). |
| `RenderError` | `NoAdapter`, `NoDevice`, `Surface`, `InvalidHandle`, `Readback`, `Unsupported`, `InvalidData`. |
| `shapes` | `sphere`, `plane`, `cuboid(size)`, `checker`. |
| `gltf` (feature `gltf`) | `load_glb(bytes) -> GltfModel`, `GltfModel::add_to(&mut scene) -> ModelParts`, `ModelParts::instance`. A bridge until the cook (M4). |
| `wgpu` | The wgpu this crate is built on, re-exported for hosts. |

## Resources and removal

Every resource kind a game streams has a generation-tagged handle: a
handle's low 24 bits are its slot, the top 8 the slot's generation.
`remove_mesh`, `remove_texture`, `remove_material`, `remove_font` (the P4
perf round, db105bd), `remove_environment_map`, `remove_texture_array`,
`remove_palette` and `remove_lut` (the sprites-arrays round: a streaming
group's colour LUTs) empty a slot and move its generation on; the renderer
frees the GPU copy before the next frame (by slot revision: a refilled slot
is uploaded again, an emptied one dropped), and the next add of the kind
fills the slot again under the new generation, so an old handle never
reaches the new resource. A stale handle is refused everywhere: the
setters (`set_mesh`, `set_material`, `set_font`, `set_texture_filter`,
`set_texture_array_filter`, ...) return false, the getters (`mesh`,
`font`, `environment_map_data`, `texture_array`, `palette`, `lut`) `None`,
`contains_*` false, and `render` refuses a frame that still names one
(`RenderError::InvalidHandle`, naming it: an instance's mesh or material, a
sprite's or an overlay sprite's texture or array, a 2D emitter's array, a
text's font or a live font's atlas, `environment_map`, the colour
grading's LUT, the stylisation's palette, a terrain's layer array). A
removed texture is also cleared from every material slot and the decal
atlas, a removed texture array from every material's `mapArray` and
`normalMapArray` (they draw without it). Removing what the frame still uses is the caller's to avoid:
og-runtime's streaming keeps such a resource, with a warning, until the
frame stops using it. `FrameStats::texture_bytes` counts textures, arrays
(padding layers included), environment maps (every level of the six RGB9E5
faces), palettes (256 × 1 RGBA8) and LUTs (32³ RGBA8), so a frame after a
removal reports the freed bytes gone. The image-based light is keyed by the map slot's
revision, the stylisation's bind group by the palette's, so a refilled
slot never draws the old resource. Tests: og-tests `tests/removal.rs`.

## The render graph

A feature-gated pass sequence per `render` call, run on the render graph
object (`src/graph.rs`, M6: declared targets and passes, data-flow order,
aliased transients; §What phase 2 owes). Passes whose feature is off are
not recorded.

1. **Sync.** New meshes, textures (with the **mip pass**: one fullscreen
   triangle per level, linear-space box filter), materials, fonts. When the
   sky changes, the **IBL passes** run (sky cube, irradiance, GGX prefilter,
   BRDF LUT; render passes only, so WebGL2 runs them); with an environment
   map they copy the cooked map into the same cubes instead.
2. **Batch and cull** (CPU). Instances stably sorted by (mesh, material);
   each sorted instance gets a world bounding sphere. Both are kept across
   frames (`instances.rs`): the sort is redone only when the submission's
   (mesh, material) sequence or a drawn mesh's bounds changed, and an
   instance's sphere and GPU transform only when its transform changed
   (bit for bit), so a still field costs one pass over the submission and
   no sort. Every pass gets a draw
   list per batch: the instances whose sphere touches the pass's volume. A
   batch that loses less than a tenth of its instances is drawn whole (no
   copy); otherwise its visible transforms are appended to the instance
   buffer. Draws bind a buffer slice, never `first_instance` (WebGL2).
   Sprites and glyphs are sorted by (layer, kind, texture or font) into runs.
   **GPU culling** (`cull_gpu.rs`; `RenderDesc::culling`: `Culling::Auto`
   by default, which takes it from `GPU_CULLING_MIN_INSTANCES` (16,384)
   instances, `Culling::Gpu` at any size) replaces the per-pass lists on an
   adapter with compute and indirect draws (native Metal, Vulkan, DX12 and
   the browser's WebGPU):
   the sorted instances are uploaded once, a compute stage tests them
   against every shadow face, the planar reflection and the camera, last
   frame's visible opaque occluders are drawn at this frame's camera into
   a depth buffer of their own, a depth pyramid (farthest depth per texel,
   each level's size taken from the screen's, not `textureDimensions`,
   which llvmpipe answers wrongly) is built from it, and a second stage
   drops camera instances whose box lies behind it. Each pass's visible
   instances are compacted in sorted order by a deterministic prefix sum
   and every batch is one `draw_indexed_indirect`, so both paths draw the
   same frame. A frame with no occluder material, or no earlier camera
   visibility, skips the occluders, their depth and the pyramid. WebGL2,
   `Culling::Cpu`, object motion blur and frames over the storage-binding
   limit keep the CPU path. The counts come back through a readback ring
   for frames after `Renderer::request_culling_counts`:
   `FrameStats::gpu_culling`, `occluded`, `occluded_triangles`,
   `in_view_triangles`, `occluder_draws` and `counts_lag` (the GPU totals
   lag by that many frames; `finish_stats` waits for the frame's own
   natively). `og-gfx` takes `OG_GFX_CULLING=gpu|cpu|auto`;
   `og-tests/tests/culling.rs` renders eight scenes both ways
   (`tests/evidence/P3/culling/`).
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
4. **Local shadows**: per shadowed light (at most `RenderInfo::shadowed_lights`:
   8 on the full tier, 4 on WebGL2, since the P4 shadows round), six cube
   faces or two paraboloid hemispheres (point, by `PointShadows`) or one
   perspective view (spot), each a pass into a 512² `R32Float` array
   layer storing distance / range, casters culled to the face frustum.
   With terrain, each of its clipmap levels that reaches into the face's
   or cascade's volume is drawn too; with a planar water reflection the
   **reflection pass** follows the shadows (§Phase 2: water).
   While a 3D fluid has particles the **fluid-3d pass** follows
   (§Phase 2: 3D fluids).
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
   prefilter and the tonemap through `textureLoad`. With adapt speeds the
   **exposure-adapt** pass follows: the 1×1 `R32Float` adapted EV walks
   towards the metered one, and the bloom prefilter and the tonemap read it
   instead.
8. **Bloom**: a 13-tap prefilter with exposure and a soft-knee threshold into
   a half-resolution chain (up to 6 levels), 13-tap downsamples, then tent
   upsamples added back level by level.
9. **Tonemap**: exposure × (HDR + bloom × intensity) → ACES, AgX or clamp → the
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
(below). The build runs on the CPU on both tiers: 38 to 53 microseconds for
256 lights, less than a compute build costs to record and submit alone
(`tests/evidence/P4/perf/clusters-compute.md`; the P2 numbers in
`tests/evidence/P2/render/forward-plus.md`). Stats: `lights` (drawn), `lights_culled`,
`lights_dropped`, `cluster_lights` (references stored),
`cluster_lights_dropped`.

The terrain's lit pass reads the same tables. The planar reflection pass
(water) shades the main view's lights through clusters of its own, built
for the mirrored camera over the same light records
(`ClusterBuilder::build_kept`; the oblique projection moves only depth, so
the tiles follow the mirrored camera's x and y).

## Shadows and antialiasing by tier

The shadows-aa round (P4, `tests/evidence/P4/shadows-aa/`) closed the
shadow and antialiasing lines phase 2 owed (`src/shadows.rs`,
`src/frame.rs`, `shaders/lighting.wgsl`, `shaders/shadow_pass.wgsl`,
`src/post.rs`, `shaders/fxaa.wgsl`, `shaders/taa.wgsl`). Everything new is
off by default or tier-dependent off the full tier's defaults: every
earlier reference frame is byte for byte what it was.

**Shadowed local lights.** Up to `RenderInfo::shadowed_lights` point and
spot lights cast shadows in a frame: 8 on the full tier, 4 on WebGL2
(`og.render.capabilities().maxShadowedLights`). When more in view cast,
the ones with the most radiance at the camera keep theirs (the clusters'
score: intensity × the colour's brightest channel over the squared
distance; ties in scene order) and the rest are counted in
`FrameStats::shadowed_lights_dropped`; `shadowed_lights` counts the drawn
ones. Each gets consecutive layers of one 512² `R32Float` array, handed out
in scene order: a spot one view, a point light six cube faces or two
paraboloid hemispheres. The array starts at twelve layers and grows once,
to the tier's capacity (48 layers with cube faces), the first frame whose
scene could need more.

**Point shadows by tier** (`RenderDesc::point_shadows`, the IDL's
`RenderOptions.pointShadows`): `Cube` draws six 90° faces, `Paraboloid` the
hemisphere below the light and the one above, each in one pass (the
projection scaled into 0.9 of the disc, so filtering at the seam reads up to
96° from the axis). A paraboloid projection is not linear: vertices are
projected and triangles stay straight between them, so a large caster's
silhouette is approximate; the distance each texel stores is measured along
its own ray to the triangle's plane (from the world position's screen
derivatives), exact for flat triangles of any size, so a one-quad ground
does not shadow itself. The texels are about 2.2 times coarser at a
hemisphere's centre than a cube face's, and the normal offset and depth
bias grow with them. On `point-spot-lights` the paraboloid frame is within
RMSE 0.0023 of the cube one (102 outlier pixels, on shadow edges and the
ring at the light's height) in 3 passes instead of 7. `Auto` picks cube
faces on the full tier and paraboloids on WebGL2: measured in Chrome on the M4 Pro (`reference-gfx.ts --time 60`), WebGL2
draws `point-spot-lights` in 0.50 ms per frame with paraboloids against
0.70 ms with cube faces, and four shadowed lights of `shadows-local-8` in
0.60 against 1.09 ms; on WebGPU one point light costs the same either way
(0.44 against 0.45 ms) and eight lights (six point) 0.60 against 1.33 ms;
natively on metal 0.4–0.5 against 0.6–0.9 ms and 0.8–1.0 against
2.2–2.7 ms, most of it the CPU encoding 38 shadow passes instead of 14
(`tests/evidence/P4/shadows-aa/`). A full-tier frame with a few shadowed
point lights costs nearly nothing more with cube faces, whose texels are
finer, so `Auto` keeps them there; a game with many shadowed point lights
saves about 0.7 ms a frame with `Paraboloid`.

**Which faces a pass stores, and which instances cast** (web findings
#4, wave 13). Every shadow pass culls a front-sided caster's front
faces, so a closed caster stores its far side and its lit side never
compares against itself. The paraboloid projection looked down its axis
with x as is, a mirror image of a perspective camera's, so its triangles
reached the rasteriser with the opposite winding and the hemispheres
stored the faces turned towards the light: a light inside a closed
caster (a lamp's glass box around its own light) was lit by paraboloids
and dark by cube faces, and a box's top edge drew a curved shadow band
down its own lit side (the cube frame of `shadows-crypt` against the
paraboloid one: RMSE 0.0148 before, 0.0036 after). The hemispheres
mirror x now (`shadow_pass.wgsl` `shadow_project` and `local_distance`,
`lighting.wgsl` `local_shadow`) and store the far side as the cube
faces do; `point-shadows-paraboloid` moved by RMSE 0.0021 (contact
shadows at the spheres' and pillars' feet) and was re-recorded.
[`Instance::cast_shadows`] keeps an instance out of every shadow pass
(the sun's cascades and the local lights' views) while it still
receives: its instances are runs of their own (`instances.rs`), and the
shadow lists skip those batches. og-runtime sets it from
`og.Mesh.castShadow` (off by default, as three.js) and
`og.Model.castShadow` (on by default: a loaded model casts, as it always
has). Before, every mesh cast whatever its flag: The
Kestrel Stair's crypt lamp, a 0.22 m glass box with its point light at
the centre, shadowed the whole crypt from its own light on the full
tier (cube faces) and not on WebGL2 (paraboloids), which the web round
read as the sun's shadow missing under the roof (crypt RMSE 0.046
against `metal`). `shadows-crypt` is that room: a roof and walls under a
low sun, a non-casting glass box around a shadowed point light, which
lights the altar and the pillar and throws a post's shadow.

**Which instances receive** (owed row 87, after wave 13).
[`Instance::receive_shadows`] (and `SkinnedInstance::receive_shadows`,
whose `cast_shadows` keeps a skinned (mesh, material)'s non-casters a
batch of their own as the static runs do) lights an instance as if
nothing stood between it and any light: the lit shaders (`pbr_fs.wgsl`,
`physical.wgsl`, and `local_light`/`physical_local` for the point and
spot lights) read the sun's cascades and the local lights' maps only
where `look_receives`. The flag rides in the instance record's layer lane
(`instances::layer_bits`: the array-map layer plus one as before for an
instance that receives, negated less one for one that does not;
`look_layer` reads the layer back), so it never splits a batch and a
receiving instance's record is the bytes it was: every og-gfx scene's
Metal frame stayed byte-identical. og-runtime sets it from
`og.Mesh.receiveShadow` and `og.Model.receiveShadow`, both on by default
(every mesh and model received before), and `og.Model.castShadow` from a
model's own field, which world.instantiate and a scene's `model` line
take from the load's `GltfDesc` (on when omitted). `shadow-flags` holds
it: a model that casts as added beside one that casts none, and in a
wall's shadow a box and a model that receive beside a box and a model
that do not.

**The sun's soft shadows and cascade blending** (`DirectionalLight::
shadow_softness`, `cascade_blend`, the IDL's `shadowSoftness` and
`cascadeBlend`). PCSS: 16 Vogel-disc reads search the cascade for blockers
over the widest penumbra the cascade allows (24 texels at most), then 16
bilinear PCF taps (64 compares) filter over the penumbra their mean depth
gives, softness × the distance from blocker to receiver; a slope term keeps
the wide kernel off receivers tilted from the light. Contact shadows stay
sharp and penumbrae widen with height (`shadows-soft`: penumbra pixels on a
plain ground 759 hard, 7,073 at softness 0.03). Blending: each cascade's
last `cascade_blend` of its depth fades into the next, whose fit reaches
back over that band so its casters are there. Both read with
`textureLoad` compares, so WebGL2 draws them too. PCSS costs 0.22 ms on WebGPU (`shadows-soft` 0.64 against
0.41 ms hard), 0.39 ms on WebGL2 (1.01 against 0.62) and about 0.4 ms on
metal at 1280×720.

**FXAA** (`Antialias::Fxaa`): FXAA 3.11's quality path (eight edge-search
steps) over the tonemapped image, which the tonemap writes into an sRGB
8-bit target the FXAA pass reads and writes to the output before the
overlay, so the HUD stays sharp. One pass, no history, on any tier; `Tier`
and `Msaa4` resolve to it where the HDR target cannot be multisampled
(they resolved to `Off` before). It costs 0.08 ms on WebGPU and WebGL2 (`fxaa`
0.50 and 0.75 ms against 0.41 and 0.67 ms without antialiasing; WebGL2's
4× MSAA takes 1.11 ms on the same scene) and about 0.1 ms on metal.

**TAA's history and sharpening** (`RenderDesc::taa_history`,
`taa_sharpen`; the IDL's `taaHistory` and `taaSharpen`):
`TaaHistory::CatmullRom` resamples the history with a 9-tap Catmull-Rom
(Jimenez's bilinear-tap form) before the neighbourhood clamp, which keeps
it sharp when the camera moves; `taa_sharpen` adds a pass after TAA into
its own target (the history stays unsharpened): an unsharp mask over the
four neighbours clamped to their range. On the thin geometry under a
camera sliding 2 cm a frame, the mean absolute Laplacian of the luma is
7.07 bilinear, 7.38 Catmull-Rom, 8.61 with 0.5 sharpening. Catmull-Rom and 0.5 sharpening together cost 0.06 ms on
WebGPU (`taa-sharp` 0.55 against `taa` 0.49 ms) and about 0.1 ms on
metal.

**Where TAA reads its history.** The pixel's centre in uv is
`in.pos.xy / size` (the window position, exact on every backend), not
the interpolated `in.uv`, and a pixel nothing moved under (a reprojection
of less than a thousandth of a pixel) reads its own history texel with
`textureLoad`, unfiltered, so no sampler or varying precision can move
the history of a still camera. On metal the still path is byte for byte
the old frames (the 37 looks frames, `taa`); the moving path moved
`taa-sharp` (a sliding camera) by RMSE 0.00029, max 4 levels. It did
not change the WebGL2 looks station's frames (still 30 of 37 apart from
metal, to four decimals). The browser diagnostics then put the
divergence in TAA's history loop on WebGL2: with the jitter zeroed and
the camera still, WebGL2 settles 0.5 to 0.75 px to the side of WebGPU,
blurred, as a history read with a small sideways offset every frame
would; with no history (every frame its own) the tiers agree. A still
camera is therefore decided on the CPU (`still_camera`: the history's
view-projection and the current one equal to a millionth of each entry)
and sent in the uniform (`TaaUniform::still`); then every pixel reads
its own history texel with no reprojection computed on the GPU at all.
Metal is byte for byte unchanged (the looks frames, every og-gfx scene).

**What differed on GL (row 68): the depth read as 0.** TAA read the
depth buffer with `textureLoad` (GLSL `texelFetch`). wgpu's GL backend
binds no sampler object to a texture only `texelFetch` reads, so the
texture's own filtering applies, and wgpu sets it to nearest only for
unfilterable colour formats: the `Depth32Float` buffer kept GL's default
linear filtering, which makes a `DEPTH_COMPONENT32F` texture without a
compare mode incomplete under GLSL ES 3.0, and `texelFetch` of an
incomplete texture returns 0. Every pixel was therefore reprojected from
the near plane. Natively, through ANGLE's Metal renderer (the one Chrome's
WebGL2 uses on macOS, loaded by wgpu's `angle` feature for the
diagnosis), a motion visualisation read 0 for the depth everywhere and
`taa-sharp` (the sliding camera) came out at RMSE 0.074 against metal,
its poles broken into dashes. For a still camera the motion is the
difference of two equal products, but at the near plane the products of
a world point 2800 m out (fx-showcase's looks station) cancel to a few
ulps of 2800 against a `w` of 0.1 m, and ANGLE's compiled shader rounds
the current and the previous product differently: a TAA harness on
ANGLE measured a still camera's motion up to 0.025 px with the depth at
0 (3.8e-5 px on Metal, 5.5e-4 px on ANGLE with the real depth), above
the own-texel test's 1e-3 px, so the history was resampled bilinearly
at a small offset every frame: the sideways drift the browser showed.
naga's GLSL itself was not the cause: it declares `precision highp`
for floats and ints, and the shader alone, given the depth, matched
Metal on ANGLE to within one 16-bit step for still and moving cameras.
The resolve now reads the depth through a nearest sampler of its own
(`depth_sampler`, binding 5) at the pixel's centre, which reads the
texel `textureLoad` read on every backend and makes the GL backend bind
a sampler to that unit. Metal: every og-gfx scene and the 37 looks
frames byte for byte unchanged. ANGLE: `taa-sharp` RMSE 0.00028 and
`taa-motion` 0.00010 against metal. The CPU's still decision stays (it
spares the reprojection where nothing moved); `taa-sharp` now applies
on every tier, so the browser tiers judge a moving camera through TAA
on WebGL2 against metal.

**Ambient occlusion by tier.** `AoMethod::Auto` (the default, and what
`PostEffects.ao` asks for) is GTAO on the full tier and SSAO on WebGL2 by
design (GTAO's horizon search is too heavy there), so any frame with AO
differs between the tiers: fx-showcase's paper-craft, voxel-worlds and
sculpted-clay looks frames (RMSE 0.006 to 0.019 against WebGPU with
FXAA) differ for this reason, and natively forcing the no-compute tier
on metal reproduces exactly those three.

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
pixel, so a scrolled list cuts a glyph in half instead of dropping it.
Overlay text also takes `weight` (the fill's edge moved out by that many CSS
pixels of the distance field: a faux bold) and `glow`: a soft halo read from
the font's distance field in the same MSDF pipeline, drawn as one extra
instance per glyph, every glyph's glow before every fill so no halo lies
over ink. The distance is the MTSDF's true field in alpha (round past
corners), or the median for a plain MSDF; it falls off quadratically to
`radius`, capped where the field ends (half the font's `distance_range` in
atlas pixels at the size drawn, so a font cooked with `px_range 12` glows
three times as far as one with 4); og-ui's skins use it for `textGlow`
(og-ui's `tests/gfx.rs` `a_text_glow_is_a_halo_from_the_distance_field`
and `references/<tier>/text-glow.png`; the cost is
`tests/evidence/P2/ui/text-glow-cost.md`). The weight stays within the
field's reach too, so a thin field never fills the glyph's quad.
*The UI's layer* (`UI_LAYER` 1000, `UI_OVERLAY_LAYER`, `screen_layer`):
og-runtime emits og.ui's painter levels from `UI_OVERLAY_LAYER` (2^20) up
and a game's screen sprites and screen text at `screen_layer(layer)`:
their own layer up to `UI_LAYER`, under every UI element, and lifted by
2^21 above it, over every one (a HUD icon of sprites on a UI panel), the
order among screen layers kept. They all drew under the UI whatever their
layer before (hollowbell FINDINGS #8). og-ui's `tests/screen_layers.rs`
and `references/<tier>/screen-layers.png`: a panel, a sprite at 999
under it, one at 999 half past its edge, one at 1001 on it and one at
1002 over that one.
`Overlay::sprite_arrays` draws overlay sprites from a layer of a texture
array, one draw per array (see [The 2D batch](#the-2d-batch-e4)).
`Overlay::glass` holds glass panels (`Glass`: a rounded rectangle, a blur
radius, a tint and its share, an opacity, a layer and a clip): a frame with
any renders its tonemap into a texture of its own (`src/glass.rs`), copies
it into the output exactly, and downsamples it into a six-level
`Rgba16Float` pyramid of 13-tap blurs (decoding an sRGB-encoded source);
each panel samples the level its blur asks for, trilinear between levels,
tints it and masks it to its rounded rectangle, sorted with the overlay's
other runs at its layer. It blurs the scene under the overlay, not other
overlay content; the pyramid is a render target of `Rgba16Float`, which WebGL2 renders to only with `EXT_color_buffer_float` (WebGPU, Metal, Vulkan and DX12 always); a frame without glass pays nothing (og-ui's `tests/gfx.rs`
`a_glass_panel_blurs_the_frame_under_it_and_nothing_else` and
`references/<tier>/glass.png`; the cost is
`tests/evidence/P2/ui/glass-cost.md`). Every
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
| Tier | Backends | Reference directory | Antialiasing (`Tier`) | Shadowed local lights | Point shadows (`Auto`) | Particle collision |
|---|---|---|---|---|---|---|
| `Full` | Metal, Vulkan (GPU or lavapipe), WebGPU | `metal`, `vulkan`, `vulkan-cpu`, `webgpu` | TAA | 8 | cube faces | the depth buffer (compute) |
| `NoCompute` | WebGL2 (wgpu GL), native GL | `webgl2`, `gl` | 4× MSAA (FXAA without a multisampled HDR target) | 4 | paraboloids | the scene's colliders (CPU) |

Both tiers request `Limits::downlevel_webgl2_defaults()` at the adapter's
resolution; both tiers run the same passes (the full tier adds GPU
particles), and the frame group's light tables are storage buffers on the
full tier and uniform buffers on the no-compute one. The
WebGL2 constraints stay designed in: shadow maps are `R32Float` colour
arrays read with `textureLoad` and compared explicitly (WebGL2 can neither
filter, `textureLod` nor gather a depth texture), the HDR target is
`Rgba16Float` (`EXT_color_buffer_float`), IBL, mips, metering and bloom are
render passes, there is no `first_instance`, and the uniform arrays stay far
below the 16 KB WebGL2 uniform block limit (the frame uniform is about 4 KB:
48 local shadow matrices since the shadows-aa round).
Four constraints of wgpu's GL backend, each of which once broke
fx-showcase on WebGL2:

- GLSL has no `textureLoad` from a depth texture: TAA binds the depth
  buffer as an unfilterable float texture (WebGPU allows a depth format
  there) and reads `.r`, on every backend.
- A texture read only with `textureLoad` gets no sampler object, and a
  depth texture keeps GL's default linear filtering, under which it is
  incomplete and reads 0: TAA samples the depth with a nearest sampler
  of its own at the pixel's centre ([Where TAA reads its
  history](#shadows-and-antialiasing-by-tier), row 68; `tests/shaders.rs` checks its GLSL).
- A pipeline has 16 texture slots across all its bind groups, not per
  stage: the lit skinned pipeline uses all 16 (frame 5, material 8, fx 2,
  skin 1), so the sky cube is the sky pass's own group and on the texture
  path the morph deltas and the joint palette share one texture. The
  physical pipelines pack their maps into one array texture and use 14
  ([Physical materials](#physical-materials-p4)).
- The texture target is chosen at creation, and a square 2D texture with a
  multiple of six layers becomes a cube map (array): the local shadow map
  (twelve layers, 48 once grown) gets a spare layer to stay a 2D array.

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
| `point-spot-lights` | Night, 7 local lights, a shadowed spot and a shadowed point light (7 shadow passes; 3 on WebGL2, whose point shadows are paraboloids). | all |
| `point-shadows-paraboloid` | The same with the point light's shadow in two paraboloid hemispheres: 3 passes, RMSE 0.0023 against the cube frame. | all |
| `shadows-local-8` | Night plaza: eight shadowed lights (six point, two spot; 38 passes) and two dim ones far behind whose shadows are left out. | full |
| `shadows-crypt` | A roofed room under a low sun with a shadowed point light inside a glass box that casts no shadow (`Instance::cast_shadows`): the light reaches the altar and the pillar, a post's shadow falls beside them; paraboloids store the same faces as cube faces. | all |
| `shadow-flags` | A low sun over a field: a trickster added as is (casting) beside one with `cast_shadows` off (no shadow), and in a 5 m wall's shadow a red box and a trickster that receive (dark) beside a box and a trickster with `receive_shadows` off (lit as if the wall were not there). | all |
| `shadows-soft` | A low sun with PCSS (softness 0.03) and cascade blending (0.2): a cube's contact shadow sharp, a raised plank's and a floating ball's penumbrae wide, poles across the cascade split. | all |
| `taa` | Thin poles and bars after 16 TAA frames. | full |
| `msaa` | The same geometry with 4× MSAA. | no-compute |
| `fxaa` | The same geometry with FXAA. | all |
| `taa-sharp` | The same under TAA with the camera sliding 2 cm a frame, Catmull-Rom history and 0.5 sharpening (`Antialias::Taa`, so TAA on WebGL2 too: the browser's moving-camera TAA frame). | every |
| `bloom` | Emissive spheres with bloom. | all |
| `exposure` | A dim dusk scene metered to middle grey. | all |
| `post-agx` | The `bloom` scene through AgX: lower contrast, the orange and magenta halos paler than under ACES, the cores rolling to white. | all |
| `environment-map` | Chrome and white spheres from mirror to rough lit only by a cooked 64×32 HDR studio (baked here as the cook bakes it), turned 0.6 rad, drawn as the background. | all |
| `sprites-20k` | 20,000 sprites (3 textures, one a 2×2 atlas, 2 layers), orthographic: 3 sprite draws. | all |
| `sprites-array` | Sixteen sprites of an eight-layer pixel-art flipbook array (nearest; the second row flipped and turned), six of a three-layer smooth array added over a plain textured strip: one draw per array and blend, three in all. | all |
| `sprites-array-fx` | The same two arrays through the other paths: four glowing flipbook sprites over a strip (glow pass), a fountain of round, glowing, added particles on random layers of the smooth array, a flipbook emitter stepping each particle through the layers by age, and four overlay sprites of the flipbook's even layers beside a plain one; after 60 ticks. 5 sprite draws (three world, two overlay). | all |
| `text-msdf` | Latin and CJK MSDF text from 0.32 to 4.5 world units, rotated. | all |
| `sky-only` | The sky with the sun in view: 2 draws. | all |
| `showcase` | Models, CSM, a shadowed spot and point light, bloom, auto exposure, world-space text and sprites, the tier's antialiasing. | all |
| `lights-256` | Night over a plaza lit by 256 point and spot lights (Forward+; 243 in view). | all |
| `atmosphere-moon` | The night colonnade with a waxing gibbous moon in view. | all |
| `materials` | Every M6 material feature: metalness and roughness rows, emissive, normal maps, clearcoat, alpha mask and blend, double-sided, unlit, a non-uniformly scaled ellipsoid. | all |
| `materials-array` | Texture-array maps (round of the cook's owed lines; references to record on metal and vulkan-cpu): eight cubes of one tile material (a four-layer sRGB colour array and a four-layer normal array), seven on their own layers and one on the material's, one batch; a cube of layer 1 as plain 2D textures beside them; two blended panes of one array material (layer 0, and the material's 3); a sphere with only a normal array (orange over the white stand-in). 5 batches, 13 instances. | all |
| `materials-physical` | Every `MeshPhysicalMaterial` lobe: clearcoat with its three maps, sheen plain and mapped, iridescence on black and on metal, anisotropy plain and swirled by its map, then clear, frosted and amber glass, a transmission-mapped cube and a high-`ior` ball in front of a plinth of coloured bars. | all |
| `skinned-hooks` | Skinned cylinders drawn with a material shader that ripples, bands and lifts them; their shadows and AO follow. | all |
| `terrain-near`, `terrain-mid`, `terrain-far` | Phase 2: the 2 km island (clipmap, four layers, shadows, fog) from 2 m, 150 m and 700 m. | all |
| `water-planar`, `water-shadows`, `water-underwater` | Phase 2: the lake's planar reflection, shadows on the water, the view from under it with caustics. | all |

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

### Pipelines made on first use

A pipeline is a program to compile, and on WebGL2 wgpu compiles and links
each one synchronously when it is made (ANGLE's compile, then the status
read that waits for it). `Renderer::new` used to make every fixed pipeline
up front, 52 programs on the WebGL2 page of the dogfood games, most of them
for passes the first frame does not draw (water, metaballs, 3D fluids,
terrain, 2D lights, glass, ribbons, the sprite variants, bloom, TAA and the
eleven effect passes): 400 to 550 ms before the first frame. Each is now a
`Deferred` (`src/deferred.rs`), made from the same descriptor when a pass
first sets it, and the shader modules only those passes use are parsed
then too (`deferred::module`), so a frame compiles only what it draws and
every frame is the frame it was (the pipelines are the same objects, made
later). `Renderer::warm()` makes the rest one per call, the likeliest
first, and answers false when none is left: the web Worker calls it in
idle time behind the play gate (`og_web_warm`), so a pass met later does
not stop a frame to compile. Measured in headless Chrome 150 on the three
dogfood games' publish builds (`packages/engine/tests/web/dogfood.ts
--boot`, three runs each, median, 2026-10-21): the WebGL2 renderer's
creation went from 507 / 546 / 461 ms to 129 / 155 / 127 ms (Lantern Vale,
Hollowbell, Kestrel Stair) and the first frame from 1,405 / 1,118 / 1,197
ms to 703 / 701 / 705 ms; WebGPU's creation from 22 / 13 / 21 ms to 10 /
10 / 9 ms.

### Lit pipeline variants

The standard lit shader (`pbr.wgsl`, `pbr_fs.wgsl`, `pbr_prepass.wgsl`)
serves every material: a vertex reads the whole 64-byte `Vertex`
(position, normal, uv, colour, tangent) and hands on 23 interpolated
floats for maps, normal maps, the dissolve, retro 3D's affine uv and the
material shader hooks. Its pipelines are keyed (side, blended, skinned,
effects); `EFFECTS` false compiles the toon, rim, flash, dissolve, decal
and AO code out. Physical materials take `physical.wgsl`'s.

A **plain** material takes the lean pipelines (`pbr_lean.wgsl`;
`GpuMaterial::lean`): opaque, not physical, no texture of any kind, no
toon, rim, flash, dissolve or outline, no alpha cut and no material
shader (the material style counts: a styled material is judged as
styled). Its unskinned instances draw with `vs_lean` / `fs_lean`, which
read position, normal and colour and interpolate 10 floats (world
position, normal, colour), and in the prepass with `vs_prepass_lean` /
`fs_prepass_lean` (position and normal in; world position, normal and the
two motion clips out, 14 floats). Positions come from the same
`lit_place` as the full path's (retro 3D's snap included) and the
fragments run the full path's shading (`lit_fragment`,
`prepass_targets`; the white default maps are still sampled, at uv 0),
so the pixels are the full pipeline's byte for byte. The `DISCARD`
override drops the fragment discard a plain material never takes, which
keeps the early depth test (hidden surface removal on Apple GPUs), except
where the fragments take screen derivatives (flat shading, or decals in
the frame; `Batch::derivatives`): on Metal a shader with a discard
computes derivatives differently, so there the discard stays. The lean
pipelines are keyed (side, effects, derivatives), the lean prepass (side,
derivatives), made at first use: one of each for a typical scene, at most
12 and 6. Materials with texture-array maps draw through a variant of the
full lit, prepass and shadow pipelines ([Array
maps](#array-maps-materials-and-terrain-layers)), never the lean ones. Shadow casters that are not skinned, dissolving or hooked read
their position alone whatever their material (`shadow.wgsl` `vs_lean` /
`fs_lean`: the world position the only varying; no extra pipeline).
spheres-20k on an M4 Pro (dev og-gfx): 7.1 to 4.6 ms a frame, every
Metal `--all` frame byte-identical to the full path's
(`tests/evidence/P4/lean/`).

### Instance looks: the hit flash of one instance

A glTF model's instances share its materials, so a material's flash
(`StandardMaterial::flash`) lights every copy. `Instance::fx` and
`SkinnedInstance::fx` (`InstanceFx`) are one instance's own: `flash` (0
to 1) mixes its lit colour towards `flash_color` after every other term,
as the material's flash does, and `tint` multiplies its base colour. They
ride in the instance record beside the transform (`instances::fx_bits`:
the flash amount, then each colour 8:8:8 in an integer below 2^24, exact
in an f32, so GPU culling's compaction copies them unchanged; the tint as
its distance from white, so a record of zeros is no look), reach the
fragment as a flat varying, and are applied by `pbr_fs.wgsl`'s
`lit_fragment` (the full and the lean paths, the array-map variant) and
`physical.wgsl` on every tier, each behind a branch, so an instance
without a look shades bit for bit as before. One (mesh, material) is one
draw whatever its instances show. The runtime fills it from `og.Model`'s
`tint`, `flash` and `flashColor` (`og.tween.flash` takes an og.Model
entity: kestrel-stair #11), and og.Mesh's `flash` and `flashColor` the
same way since row 85 (it had drawn a flashing mesh through a copy of its
material at one of 16 flash levels: a draw and a material per level, each
copy kept as its material was when made, so a material updated during the
flash, a dissolve, flashed stale; the amount is continuous now, the colour
8 bits a channel). The og-tests scene `model-flash` (three instances of one glTF
model, one flashing white and one tinted, and two skinned cylinders of one
material, one flashing red) holds it on metal.

### Instance and vertex encodings

**Instances: 64 bytes.** Every instance buffer (the CPU path's, GPU
culling's regions, the previous transforms of object motion, the
scattered copies) holds an instance's transform as the x, y and z of its
four columns and, after them, an array-map instance's layer with whether
it receives shadows (`instances::layer_bits`) and the
instance's look (`instances::GpuTransform`: `Float32x3` attributes 4 to 7
and a `Float32x4` at 8, pbr.wgsl's `InstanceFxIn`; 16 floats a slot in
`cull.wgsl`); a skinned instance holds the columns before its palette and
its look after its morph weights (`skin::SkinnedInstanceGpu`, the look at
attribute 13). The record was 52 bytes until the look joined it (wave 11):
every Metal `og-gfx --all` frame stayed byte-identical. `instance.wgsl`'s
`instance_matrix` rebuilds the 4x4 with the fourth row (0, 0, 0, 1), so
an `Instance::transform` is affine. The points such a matrix moves have a
w of 1 the shader compiler can see, and Metal then reorders the sum of the
projection that follows (the w term first), which moved vertices by an ulp
and flipped edge pixels in 45 of the 68 `og-gfx --all` frames. Those
projections (the view, the cascades and spots, last frame's view, the
occluders', GPU culling's screen boxes) go through `mul_point`, the column
order fused chain the compiler used when w came from the buffer, and a
point moved by a material's `vertex_offset` keeps the plain product
(`hooks.rs` `moved_clip`): every Metal frame is the 64-byte encoding's,
byte for byte. The GL and SPIR-V tiers compile the same arithmetic (naga
writes `fma` as `a * b + c` for GLSL ES 3.0); their references are
compared under the gate's tolerance.

**Vertices: 16 bytes, opt-in.** `VertexEncoding::Compact`
(`RenderScene::set_vertex_encoding`; `compact.rs`) uploads a mesh as the
position in `Unorm16x4` over its bounds, the normal octahedral in
`Snorm16x2` and the colour in `Unorm8x4`; no uv and no tangent, which no
plain material reads. The bounds' corner and size are not a shader input:
a (mesh, material) run that draws the compact vertices folds them into
its instances' GPU transforms (`Run::dequant`), so the vertex stage reads
a point in the unit box; the normal is stored through the bounds' scale,
so the normal matrix of (instance x dequantisation) turns it back. The
lean lit, lean prepass, lean shadow caster and occluder-depth pipelines
have compact variants (`vs_lean_compact`, `vs_prepass_lean_compact`,
shadow `vs_lean_compact`, `cull_depth.wgsl` `vs_compact`), GPU culling
reads the unit box's bounds with a radius that never shrinks the
sphere, and CPU culling keeps the mesh's own. The f32 vertices of a
compact mesh are uploaded only when something else draws it (a material
that is not plain, a scatter, a skinned instance: `Renderer::ensure_full`),
so a mesh that only plain materials draw never holds them on the GPU.
Skinned and morphed meshes ignore it. It is quantised (the position to
1/65535 of the bounds, the normal to about 1e-4 rad, the colour to 8
bits), so its pixels are near `Full`'s, not equal: the default stays
`Full` and every reference frame with it. `og-gfx` takes
`OG_GFX_COMPACT=1` (every mesh compact): all 68 `--all` scenes draw, the
largest difference against the `Full` frames rmse 0.011 (`showcase`).
Natively the sphere field's GPU time does not move with it
(`tests/evidence/P4/parity/`): its 221 vertices sit in cache, and the
pass is bound by its 7.3 million triangles (its time follows the
instance count, not the pixels, the varyings or the vertex fetch).

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

**Texture arrays** (`sprite_arrays`, `SpriteArray { sprite, array, layer
}`): a world sprite draws from a layer of a texture array instead of its
texture, through pipeline variants (`shaders/sprite_array.wgsl`, made on
the first frame that draws one) that sample a `texture_2d_array`: alpha,
additive and the glow buffer's, one per sprite pipeline. The instance
carries its layer (location 7 of the sprite instance, which the other
sprite pipelines do not read), so the run key is (blend, array): every
sprite of one array is one instanced draw whichever layer it shows, where a
sheet of separate textures would break the run at each frame. The group's
view is an explicit `D2Array`; on WebGL2 the GL backend would take an array
of one layer, or of a multiple of six, for a 2D or cube texture, so
`add_texture_array` pads it (`gl_safe_array_layers`) and callers still
address only the layers they gave. A layer past the array's count is
refused (`InvalidData`). Array sprites take the hit flash, 2D lighting,
fog, the additive blend, the soft dot and the glow pass (a glowing array
sprite joins the glow runs by array, sorted after the plain textures, so
every existing glow run keeps its order). Screen sprites draw from arrays
too (`Overlay::sprite_arrays`, indexing `Overlay::sprites`): the overlay's
variant (`shaders/overlay_array.wgsl`, in the output's format and its sRGB
view's, made on the first frame with one) reads the layer from the overlay
instance's otherwise unused `effect.w`, one draw per array. 2D particles
draw from an array when their emitter names one (`ParticleEmitter::array`,
sprite mode only: a billboard emitter with an array is refused): the
layers are the sheet's frames, picked by the sheet's mode (`OverLife` and
`Fps` by age, `Random` per particle; `sheet.frames` limits them, 0 every
layer), every particle of the array one draw; `ParticleEmitter::round`
gives sprite particles the soft dot's round edge over their texture or
layer. MSDF text never samples an array. og-runtime's `og.Sprite { array,
arrayLayer }`, `SpriteDesc.array` (world or screen), an emitter's `array`
line and a flipbook sheet with `array true` (its frames cooked into an
array, frame n drawn as layer n) all come here. Tests: og-tests
`sprites-array` (an eight-frame pixel-art flipbook in two rows and a
three-layer smooth array added over a plain strip: three draws; unchanged
byte for byte by the round that added the other paths), `sprites-array-fx`
(glowing array sprites, a round glowing emitter on random layers and a
flipbook emitter by age, four overlay array sprites beside a plain one),
`a_sprite_draws_the_layer_of_its_texture_array`,
`array_sprites_glow_and_draw_in_the_overlay` and
`array_particles_take_their_layers_by_age_or_at_random`.

**Drop shadows** (`sprite_shadows`, `SpriteShadow { sprite, opacity,
ground }`; the IDL's `SpriteDesc.dropShadow` and `Sprite.dropShadow`): a world
sprite's shadow is one more instance of its quad, region and texture (or
array layer) in black at `opacity` times its alpha, keyed `f32::MIN` so it
draws before every sprite of its layer, whose `ground` (location 6: on,
the x and z moved per metre of height, the ground's height) makes the
vertex shader carry each corner of the standing quad along the scene's
sun (`RenderScene::light`) down to the plane y = 0, lifted
`SPRITE_SHADOW_LIFT` (1 cm) over a ground mesh there. Depth-tested as the
sprites are, so the set's geometry in front hides it. None with the sun at
or below the horizon; a low sun is raised to about 11° so a shadow is at
most five times the sprite's height (`shadow_ground`). Pixel sprites in a
3D set (a perspective camera over the xz ground) cast shadows that agree
with the meshes' cascades; a 2D camera looking down -z sees the plane edge
on. Shadows are not counted in `FrameStats::sprites`. The instance grew
by those 16 bytes (100 a sprite or glyph). Tests: `sprites.rs`'s
`a_drop_shadow_draws_first_in_its_layer_projected_along_the_sun` and
`a_low_sun_casts_at_most_five_heights` (CPU), og-tests `sprites-shadow`.

*Off the plane y = 0* (stylisation round three): a shadow's `ground`
(`ShadowGround { height, normal }`) puts it on the plane through `height`
under the sprite's anchor, tilted to `normal`: `ground.w` carries the
height (plus the lift), and the shadow instance's `fx.yz` (glow, lighting
and the soft dot are never a shadow's, and the vertex shader zeroes them
for one) the rise per metre along x and z, so the instance layout is
unchanged; the vertex shader (`sloped_shadow`, sprite.wgsl and
sprite_array.wgsl) moves each corner down the sun until it meets that
plane, a slope steeper than about 70° taken as that steep and a sun
grazing the slope held to five heights. A level ground keeps the old
projection, byte for byte. `ShadowGround::fit(height_at, sprite, sun)`
fits the plane to uneven ground: through the ground at the anchor, along
the slope to where the shadow's tip falls and across it over the sprite's
width, raised where the ground between bulges above it, so the shadow
never sinks into a hill. og-runtime gives every shadow the level ground
of `dropShadowHeight` (0 by default) and, where an og.Terrain drawn under
the anchor is higher, the plane fitted to that terrain
(`terrain::ground_drop_shadows`, sampling as `og.terrain.sample` does,
streamed tiles included). *In the cascades*: the fragment shader reads
the sun's cascaded shadow at the shadow's point (lighting.wgsl
`cascade_shadow`, now prepended to the sprite shaders) and multiplies the
shadow's alpha by the light there, so where a wall or tree already
shadows the ground the drop shadow fades out instead of darkening it
twice. Tests: `sprites.rs`'s
`a_drop_shadow_lies_on_its_ground_at_its_height_and_slope` and
`a_fitted_ground_follows_the_terrain_under_the_shadow`, og-runtime
`tests/drop_shadows.rs` (a cooked ramp terrain, a deck above it, a sprite
off it and an immediate sprite), og-tests `sprites-shadow-terrain`
(figures on hills, one on a deck, one whose shadow runs out of a wall's
shadow). The sprite vertex shader's slope path moves `sprites-20k` on
Metal by at most 2/255 on 261 pixels (the compiler's code for the
standing quad changes with it; no shadow in that scene); every other
reference is byte-identical.

*Bent over hollows* (stylisation round four): one plane cannot follow a
hollow or a ridge narrower than the shadow (it floated over the one and
was lifted over the other). `ShadowGround::bend` (`ShadowBend`) holds the
ground's heights under a 5 × 3 grid of the sprite's quad (rows up the
quad, so along the shadow; columns across it), each point carried down
the sun to the first ground its ray meets (`landing`: marched over three
sprite heights, then bisected). `ShadowGround::fit` keeps the plane where
it lies within `SHADOW_BEND_TOLERANCE` (3 cm, plus 3 % of the sprite's
height) of the ground under every grid point's landing on it, and bends
otherwise, so og-runtime's terrain shadows bend with no change there. A
bent shadow is drawn as 4 × 2 instances of the sprite, one per cell
(`ground.x` 2): the cell's column and row in the two flash bytes a shadow
never uses, its corners' heights in `ground.w` and `fx`, so the instance
layout is unchanged again. The vertex shaders (`bent_vertex`, sprite.wgsl;
the same in sprite_array.wgsl) compute each corner from the whole quad,
so cells meeting at a corner compute it alike and the shadow has no
seams, and carry it down the sun to its height; the plane path is the
round-three code behind an early return (`sprites-20k` byte-identical).
*Limit*: between grid points the ground is taken as straight, so a
feature narrower than a cell (a quarter of the shadow's length, half the
sprite's width) is still bridged, a bump between two points can cover
the shadow there, and where the ground falls away steeper than the sun
(the far wall of a deep pit) the cells stretch across the part in the
ground's own shadow. Tests: `sprites.rs`'s
`a_shadow_bends_over_a_hollow_narrower_than_itself` (a pit and a ridge,
the eight cells and their corners), og-tests `sprites-shadow-terrain`,
which gains a hollow 0.4 m deep and about 1.4 m across and a figure whose
shadow dips into it (its reference moves).

Tests: `tests/sprites2d.rs` (sort keys across textures within a layer,
layers still win; tile culling, tiles under their layer's sprites, repeat
filling the view's edge, an animated quad on its second frame; nearest
against linear sampling; nine-slice borders at their texel size; an
outline only when asked, the fill kept; a trail hidden under a layer-0
sprite without a layer and shown over it on layer 1), each broken once
(`tests/evidence/E4/break-and-restore.md`).

### Untonemapped sprites

`SpriteFx::tone_mapped` false (the IDL's `SpriteDesc.toneMapped` and
`Sprite.toneMapped`, three.js `Material.toneMapped`): an alpha-blended
world sprite that keeps its colours as given while the scene around it
rides exposure, bloom and the tone curve: a cream speech bubble that
auto exposure at dusk had blown to white and haloed (lantern-vale
FINDINGS #17). *Chosen: drawn in place, flagged in the HDR alpha*, not
in a pass after the tonemap. The sprite stays in the main pass, so it is
depth tested against the scene's depth buffer on every tier (a pass
after the tonemap would need that depth at the output's sample count,
which WebGL2's 4x MSAA cannot give a single-sampled target) and sorted
with the other sprites. Before the frame's first such run a cover
(`sprite.wgsl` `vs_cover`/`fs_cover`, a pipeline with no groups) sets the
HDR target's alpha to 1 and keeps its colour; the sprite's pipeline
(`UNTONEMAPPED_BLEND`, `fs_untonemapped`: no fog) blends its colour as
alpha blending does and takes the alpha towards 0 by its coverage, a
tone-mapped sprite or glyph over it takes it back towards 1. The alpha is
then the tone-mapped fraction of each pixel: the MSAA resolve averages
it, TAA blends it with the colour's weights (its history clamped to the
neighbourhood's range, so a moving bubble leaves no trail), outlines,
depth bands, depth of field and motion blur pass it on, the bloom
prefilter weights its taps by it (`bloom_tap`: such a sprite neither
blooms nor dims the bloom round it) and the tonemap mixes the clamped,
unexposed colour with the curve by it (post.wgsl `tonemapped`; the
post uniform's `extra.y` says the frame has one, so every other frame's
pixels are unchanged). Grading, vignette, grain, the screen flash and
the stylise chain after the tonemap still apply. An additive sprite adds
light to the scene and is always tone mapped. og-tests
`sprites-untonemapped` (TAA, `metal`): a tone-mapped bubble blown
towards white with a halo beside untonemapped ones in their cream and
red, one half behind a crate, one from a texture array, a tone-mapped
marker over a corner of one.

## E1 effects

Everything a game declares on `RenderScene` as data (`src/effects.rs`,
`particles.rs`, `ribbons.rs`, `light2d.rs`, `skin.rs`); a field left at its
default records no pass and costs nothing (`effects_off_add_no_passes`).

- **Post stack**: ambient occlusion (`AoMethod::Gtao` / `Ssao`, applied to
  the ambient term), outlines (depth and normal edges, per-scene or
  per-material colour and width; below), depth of field (`DofQuality::Low` / `High`), motion
  blur from per-object and camera motion vectors (`fx.rs` carries the
  previous view-projection), god rays, colour grading by `ColorLut`,
  vignette, chromatic aberration, film grain seeded by the tick, screen
  flash and camera shake (a deterministic offset of the camera, not a
  post pass).
- **Decals**: up to 8 projected boxes per frame, read by the lit shader
  (`fx-decals`); each is capped to its box.
- **Particles**: `ParticleEmitter` assets and `ParticleSystem`s with
  curves, bursts, noise, sprite sheets, collision against the depth buffer
  (compute) or against `RenderScene::particle_colliders` (the CPU path:
  planes, boxes, spheres and terrain height fields, which og-runtime fills
  from its og.Collider components; a step from outside a shape to inside
  bounces, as the compute shader's response), soft fade against the scene,
  local or world space, 2D emitters drawn as sprites (from a texture or a
  texture array's layers, `ParticleEmitter::array`, with `round` for the
  soft dot's edge). Simulated by compute
  on the full tier and on the CPU elsewhere; without collision both paths
  give the same particles (`gpu_and_cpu_paths_simulate_the_same_particles`)
  and a 600-tick run is deterministic on either.
- **Trails, ribbons and beams**: `Trail` keeps a per-id history for its
  `TrailStyle`'s lifetime; `Ribbon` is an explicit polyline; `Beam` joins
  two points and scrolls with the tick.
- **2D lighting**: `Lighting2d` with `Light2d`s shadowed by `Occluder2d`s
  (soft penumbrae) and an order-independent glow buffer; only the lit
  sprite layer range is affected; `SpriteFx` adds flash, glow, lit and soft
  per sprite. Tile layers in the range are lit as its sprites are
  (`TileLayer::lit`, true unless the map's layer opts out: og-tilemap's
  `lit false`, a Tiled layer's bool property `lit`, the blob's
  `layer_flag::UNLIT`): every tile instance carries the lit flag the
  sprite shader reads (`fx.y`), so the ambient, the lights and the
  occluders' shadows reach a tilemap on every tier, the same pipeline.
  Before, tiles were never lit (hollowbell FINDINGS #9: a dark cave of
  tiles drew as authored). Tests: `tests/sprites2d.rs`
  `tile_layers_in_the_lit_range_take_the_2d_light` (near, far, behind a
  wall, opted out, out of the range; broken once by an unlit flag) and
  the og-tests scene `tilemap-lit` (a dark cave of three tile layers, a
  torch and a crystal, a pillar's and a ledge's shadows on the back wall,
  a window's sky layer kept as authored).
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
  main pass, composited at the fluid's sprite layer by a threshold with an
  antialiased edge and a rim. Phase 2: fluids sharing a `group` are one
  surface: the group takes a field's run of four channels (one per `lane`),
  a particle adds its kernel into its lanes' channels by its `mix`, and the
  surface is thresholded on the sum and coloured by the channels' shares,
  so blended particles draw blended colours. Lone fluids take one channel.
  Surfaces are packed into up to `FIELDS` (4) field targets
  (`metaballs`, `metaballs.1`…; one field pass each, all read by the main
  pass), so up to `MAX_FLUIDS` (16) fluids draw a frame; a lone fluid draws
  exactly as before (`fluid-2d` at RMSE 0). og-tests' `fluid-groups` scene:
  six lone pools, a group blending water into oil, a group layered.
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
  image-based light. Its maps came in P4 ([Physical
  materials](#physical-materials-p4)).

## Physical materials (P4)

`StandardMaterial` carries three.js `MeshPhysicalMaterial`'s fields under
its names, with glTF `KHR_materials_*` meanings (`src/physical.rs`,
`shaders/physical.wgsl`; `og-tests/tests/physical.rs` and the
`materials-physical` reference):

- **Clearcoat maps**: `clearcoatMap` (red, times `clearcoat`),
  `clearcoatRoughnessMap` (green), `clearcoatNormalMap` with
  `clearcoatNormalScale` (the coat's own normal in the vertex tangent
  frame; the base keeps `normalMap`).
- **Sheen** (`sheen`, `sheenColor`, `sheenRoughness`, `sheenColorMap` in
  RGB, `sheenRoughnessMap` in alpha): the Charlie distribution with
  Neubelt's visibility for the lights and Estevez and Kulla's fit of its
  albedo for the image-based light, over a base it takes
  1 - 0.157 max(sheen colour) from (as three.js).
- **Transmission** (`transmission`, `transmissionMap` red, `thickness`,
  `thicknessMap` green, `attenuationDistance` (0: none, three.js's
  Infinity), `attenuationColor`, `ior`): when a frame draws a transmissive
  material, the main pass splits after the sky as it does for water, the
  resolved HDR target is copied to the renderer's `transmission` texture
  (full mip chain, box-filtered by the mip pass) and transmissive
  materials are drawn with the blended ones, back to front, writing depth.
  Each refracts the view at its surface by `ior`, follows the ray
  `thickness` (object space, scaled with the model) through the volume,
  projects its exit and reads the copy there bicubically at a level of
  log2(width) × roughness × clamp(2 ior - 2, 0, 1), tints it by the base
  colour and by `attenuationColor` to the power of the distance over
  `attenuationDistance`, and puts it in place of the diffuse light, as
  three.js does. So a transmissive object shows the opaque scene, the sky
  and terrain behind it, not water, transparent or other transmissive
  objects; it casts a shadow like an opaque one (as in three.js) and is
  left out of the depth prepass, the planar reflection and GPU occlusion
  like a blended one. `ior` also sets a dielectric's F0, ((ior - 1) / (ior + 1))².
- **Iridescence** (`iridescence`, `iridescenceIOR`,
  `iridescenceThicknessRange`, `iridescenceMap` red,
  `iridescenceThicknessMap` green between the range's ends): Belcour and
  Barla's thin film evaluated per pixel at the view angle, mixed into the
  specular Fresnel of the lights and the image-based light.
- **Anisotropy** (`anisotropy`, `anisotropyRotation` from +u,
  `anisotropyMap` with the direction in red and green and the strength in
  blue): anisotropic GGX (Burley's distribution, the height-correlated
  Smith visibility) along the vertex tangent frame, and the image-based
  light read along a bent normal (as three.js). Meshes without tangents
  get generated ones at upload.

**Which materials take this path.** `StandardMaterial::is_physical`: any
of these maps, sheen, transmission, iridescence or anisotropy above 0, or
an `ior` other than 1.5, on a material that is neither `unlit` nor `toon`
(both replace the lighting model). Every other material, the scalar
clearcoat included, draws with the standard pipelines and module text it
always did: every earlier og-tests frame is byte for byte the frame of
the renderer before this change (`tests/evidence/P4/materials/`). A physical material with every lobe off draws the standard
frame (RMSE 0, `tests/physical.rs`).

**WebGL2's sixteen texture units: packing, and a pipeline of its own.**
A physical material may sample 18 maps; the skinned lit pipeline already
spends all 16 units WebGL2 gives a pipeline. A physical material keeps
its standard bind group (every pass but its lit one: shadows, the depth
prepass, the planar reflection and scatter draw it as before) and gets a
second one for the physical pipelines, whose layout differs:

| Group | Textures |
|---|---|
| 0 frame | cascades, local shadows, irradiance, prefiltered, BRDF LUT (5) |
| 1 physical material | base colour, normal, emissive, gradient, the pack (5), plus the material and physical uniforms |
| 2 physical effects | SSAO, decal atlas, the transmission copy (3) |
| 3 skin | the joint palette texture (1, WebGL2) |

The pack is an RGBA8 2D array texture made once per set of maps (shared by
materials with the same maps; made again when one of them is replaced)
by a render pass per layer and mip level (`shaders/pack.wgsl`: each output
channel is one channel of one of four sources or a fallback, the implicit
derivatives pick each source's mip). Only the layers a material has maps
for are made, at the largest source's size; the uniform names each
group's layer (-1: white):

| Layer | r | g | b | a |
|---|---|---|---|---|
| standard | metalness (b) | roughness (g) | ao (r) | dissolve (r) |
| coat | clearcoat (r) | clearcoat roughness (g) | transmission (r) | thickness (g) |
| sheen | sheen colour (rgb, stored sRGB-encoded) | | | sheen roughness (a) |
| film | the coat's normal (xyz; z rebuilt from a two-channel BC5/EAC map) | | | iridescence (r) |
| anisotropy | anisotropy (rgb) | | | film thickness (g) |

The alternative, pipeline permutations keyed by which map groups a
material uses, would have needed more than one permutation for a material
with every map, and a texture atlas would have bled between maps under
mipmapping. The pack costs one RGBA8 texel per layer used and a few passes
when a material is first drawn. `tests/webgl2.rs` translates the physical
shader and the pack shader to GLSL ES 3.0 and counts the physical module's
textures (14, the static and skinned pipelines alike).

**Tiers.** Every lobe and map draws on both tiers. The transmission copy's
mip chain is made with render passes (the mip pass), so WebGL2 has it
too: no difference by design. The physical shader asks no texture for its
level count (WebGL2 cannot); it derives it from the copy's size.

**From glTF.** The cook and the glTF bridge read the seven extensions with
one parser (`gltf::physical_extensions`, the textures as their kinds: the
sheen colour as sRGB colour, the coat's normal as a normal map, the rest
linear) into `PhysicalExtensions`; a cooked model stores them in its
material rows (`og.ModelMaterial` version 2) and adds a material with any
as a standard material (`CookedModel::standard_material`), one without in
its glTF shape as before.

## Array maps (materials and terrain layers)

Texture arrays (`add_texture_array`, og-cook's `type texture-array`) are
sampled by lit materials and terrain layers as well as by sprites, so a
set of variants (tiles, skins, damage states, a terrain's ground types) is
one resource, one material and one draw.

**Materials** (`src/material.rs` `ArrayMaps`, `shaders/pbr_array.wgsl`,
`shaders/shadow_array.wgsl`). `StandardMaterial::mapArray` and
`normalMapArray` are texture-array slots (`TextureArrayHandle::slot`, 0
none) with `mapLayer` and `normalMapLayer`; a material with either reads
its base colour and normal map from the arrays only (`map` and `normalMap`
are not read; a missing one is a one-layer white or flat-normal array).
`Instance::material_layer` replaces both layers for one instance, so
instances of one material on any layers stay one (mesh, material) batch
and one instanced draw. The layer travels beside the instance's
transform in its 64 bytes (`instances::GpuTransform::layer`: the layer
plus one, 0 for every other instance; [Instance and vertex
encodings](#instance-and-vertex-encodings)), so culling, sorting, GPU
culling's compaction and the instance buffer carry it without a new
stream (a float holds layers exactly to 2^24). The array-map variant's
vertex stages read it at location 8, the others never do, so positions
are the standard path's, and array-map materials are
never occluders (the occluder depth
pass reads transforms as they are).

One variant of the standard lit, prepass and shadow pipelines draws them,
made when a scene first draws such a material: its module is the standard
lit module's own text with bindings 1 and 4 declared `texture_2d_array`,
every sample of them at the fragment's layers, a flat `layer` varying on
`VertexOut` and `PrepassOut` (`@location(7)`), and `vs`, `fs`,
`vs_prepass`, `fs_prepass` renamed `*_standard` with `pbr_array.wgsl`'s in
their place; the shadow variant renames `vs` and `vs_lean` and appends
`shadow_array.wgsl`'s. Each rewrite asserts the text it replaces, so a
change to the standard shaders fails `tests/shaders.rs` instead of drifting.
The standard builders (`pbr_pipeline`, `prepass_pipeline`,
`shadow_pipeline`) make the variant's pipelines from those modules and the
variant's layouts (the material group with two `D2Array` bindings, same
texture count, so WebGL2's skinned pipeline still fits its sixteen units),
keyed as the standard ones: a typical scene adds one lit, one prepass and
one shadow pipeline. A material's array group is a second bind group (its
standard group stays for shadows with a dissolve, scatter and everything
else); the layers are in the material uniform's `coat.zw`, 0 for every
other material, whose uniforms, modules and pipelines are unchanged.
Skinned instances take their material's layers. Physical materials and
material shaders keep their 2D path (`StandardMaterial::draws_array_maps`).
og-runtime: `StandardMaterialDesc.mapArray`, `mapLayer`, `normalMapArray`,
`normalMapLayer` and `og.Mesh.materialLayer` (-1: the material's); an
array a material samples stays resident while it does. Tests: the
material unit tests, `tests/shaders.rs`, og-runtime
`tests/material_arrays.rs`, og-tests `materials-array`.

**Terrain layers** (`src/terrain.rs`). `Terrain::layer_arrays` names an
albedo array and a normal array, each `TerrainLayer` its `albedo_layer`
and `normal_layer` (in its uniform's unused `slope.w` and `altitude.w`).
Such a terrain binds the two arrays in place of up to eight textures,
through a variant of the terrain group layout (bindings 5 and 9 as
`D2Array`, 6 to 8 and 10 to 12 gone) and of its lit, prepass and shadow
pipelines, made when a terrain first needs it; the variant's modules are
the standard ones' text with the two arrays declared and `layer_albedo` /
`layer_normal_map` reading them. Every `TerrainDraw` carries its own
pipelines and `draw_levels` sets them per terrain, so terrains of either
kind draw in one pass. A terrain of separate textures uploads, binds and
draws exactly as before. og-cook's `layers` / `layerNormals` and
`albedoLayer` / `normalLayer` reach it through og-runtime. Tests: the
terrain unit test of the variant's source, og-runtime
`tests/material_arrays.rs`, og-tests `layer_arrays_draw_as_separate_layer_textures`
(terrain-near both ways, within the arrays' CPU-made mips).

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

## Phase 2: terrain

`src/terrain.rs`, `src/terrain_tiles.rs` (the tile pyramid),
`shaders/terrain.wgsl` (the clipmap vertex, the tile lookups),
`terrain_lit.wgsl` (shading, the prepass), `terrain_shadow.wgsl`. Off
unless `RenderScene::terrains` names one: every earlier reference frame is
unchanged.

- **Data.** `add_terrain(TerrainData)` once (a field in memory);
  `set_terrain` replaces it (a runtime edit). Or streamed:
  `add_terrain_stream(TerrainLayout, range)` and `set_terrain_tile(handle,
  index, Some(Arc<TerrainTile>) | None)` as tiles arrive or leave (phase
  3); `set_terrain_stream` starts it over.
- **Tiles (phase 3).** A terrain is drawn from a pyramid of tiles
  (`TerrainLayout`: level `k` every `2^k`-th sample of the field, cut into
  tiles of `tile` quads, 128 by default; the last level one tile).
  A `TerrainTile` holds `(tile + 3)²` heights (a border of one), the normal
  map of its `tile + 1` samples as half floats with every mip (built on the
  CPU, one-sided at the terrain's edge), and `(control + 2)²` control
  texels with mips (`ControlPyramid`: the map resampled to `control`
  texels per level-0 tile, halved per level). Tiles live in three texture
  arrays (`R32Float` heights, `Rg16Float` normals, `Rgba8Unorm` control),
  one layer per slot; a 16 KiB uniform table gives every tile its slot and
  fade. `terrain_tiles_wanted(layout, terrain, range, TerrainView,
  capacity)` ranks what a frame needs: from the top level down, the tile
  whose samples look largest on screen is refined into its four children
  until every sample is under a pixel or `TerrainLod::budget` (16 MiB by
  default, at least 8 slots) is spent, the level-0 tiles within
  `TerrainView::detail` first; each after its parent. A field in memory
  makes its wanted tiles at once; a streamed one draws those of them it
  holds. A vertex and a pixel read the finest resident tile at their point,
  so a tile that has not arrived shows its parent; a streamed tile fades in
  over twelve frames once its parent has. Levels are decimations of the
  field, so heights agree wherever two of them meet: no cracks with any
  set of resident tiles (`FrameStats::terrain_tiles`, `terrain_uploads`).
- **The clipmap.** Level `l` is `ring` × `ring` quads (64 by default) of
  `spacing · 2^l` around the camera, its centre snapped to twice its
  spacing so vertices never move in the world. One index buffer holds the
  full grid (level 0) and the ring with its hole at each of the four
  places the finer level's snapping can put it; each level is one indexed
  draw with its uniform slot by dynamic offset (WebGL2 has no first
  instance). The levels end when the next one's hole would cover the
  terrain or lie beyond the far plane. In the outer eighth of each level
  its odd vertices slide onto the coarser grid (CDLOD's morph, by the
  camera's own position, so it is continuous as it moves): at the edge the
  two levels share vertices and heights bit for bit (positions are whole
  grid units until scaled), so there are no cracks and no pops.
- **Layers.** Up to four: weight = control channel × slope band × altitude
  band, sharpened by the layers' heights (the albedo's alpha) with
  `height_blend`, normalised (all zero: the first layer). Each is sampled
  over world xz every `tile` m or triplanar (Golus's whiteout normals), its
  normal map applied in the frame of the terrain's normal; samples inside
  branches use `textureSampleGrad` with the pixel's own derivatives. The
  maps are each layer's own textures (up to eight bound), or layers of two
  texture arrays (`Terrain::layer_arrays`, two bound): [Array
  maps](#array-maps-materials-and-terrain-layers).
- **Light.** The `StandardMaterial` lighting from `lighting.wgsl` (shared
  with pbr.wgsl and water.wgsl now): the directional light with cascaded
  shadows, the eight local lights with their shadows, the image-based
  light, caustics under the frame's water, then fog.
- **Shadows.** The terrain casts into the cascades and the local lights'
  faces (both faces of the open surface; each level culled to the pass's
  volume) and receives them; its bounds join `ShadowBounds::Auto`.
- **Prepass.** Drawn into the depth-normal prepass (water's depth below,
  AO, outlines, depth of field, particles' collision).
- **Scatter.** Detail meshes (grass, stones) over a terrain where a layer
  shows: one candidate per cell of a grid fixed to the terrain
  (`1 / sqrt(density)` m), jittered, turned and sized by a hash of the
  cell and the seed, kept with the probability of its layer's weight there
  (`layer_weights`: the shader's weights before the height blend), upright
  or leaning with the ground by `align`. `RenderScene::scatters`
  (`ScatterDraw { terrain, mesh, material, scatter }`, phase 3) has the
  renderer place them itself (`src/scatter.rs`, `shaders/scatter*.wgsl`),
  32 × 32 cells at a time: a chunk's transforms are built once when it
  comes within `distance` (again only when its terrain data, placement,
  layers or settings change; a streamed terrain's once its level-0 tiles
  are resident, sixteen a frame), kept in an instance buffer of its own
  while in use, culled per pass and drawn with one instanced draw. The
  vertex shader drops copies past `distance` and the fragment shader thins
  them out over the outer `fade` with an interleaved-gradient dither, at
  full size, in the lit, prepass and shadow passes alike. The copies take
  their StandardMaterial's colour (factor, texture, vertex colours),
  cut-off, metalness, roughness, emissive, unlit and flat modes; not its
  normal map or effects. `Terrain::scatter(data, TerrainScatter, camera)`
  answers the same copies within `distance` on the CPU (tests, tools):
  nearest rows first, at most 65,536. `FrameStats::scatter_draws`,
  `scatter_copies`, `scatter_built`.

Cost: `tests/evidence/P2/terrain-water/feature-costs.md`.

## Phase 2: water

Each off by default on `Water` (every earlier water frame is unchanged):

- **Planar reflection** (`planar_reflection`, the fraction of the output's
  resolution; one surface a frame, the first that asks, with the camera
  above it and perspective). The camera mirrored in the surface
  (`look_at` of the mirrored eye, target and up: a proper rotation, so
  the image is flipped left to right and winding is unchanged) draws the
  opaque meshes, skinned meshes, terrain and sky into the graph's
  `reflection` target (`FramePass::Reflection`, before the main pass;
  multisampled with a resolve on the MSAA tier) with its own frame and
  effect uniforms. Its projection's near plane is the water's plane, 2 cm
  up (Lengyel's oblique clipping), so nothing below the surface is
  reflected; the sky's rays use the unclipped projection. The water
  samples it at `(1 - u, v)`, shifted by the waves.
- **Shadows** (`shadows`, 0 to 1): the directional light's cascaded shadow
  at the surface darkens its sun highlight, its foam's light and its deep
  colour's scattering (to 40 %); the sky's reflection stays.
- **Underwater** (`underwater_color`, `underwater_density`; on by
  default, density 0.09): with the camera inside a surface's extent and
  under its waves, the frame's fog becomes the water's (exponential from
  the eye, the sky left clear), and the surface seen from below shows the
  scene above inside Snell's window (steeper than about 49°) and the
  water's colour reflected outside it.
- **Caustics** (`caustics`, `caustics_scale`): two scrolling cell patterns
  multiplied, added to the sunlight of every lit and terrain surface below
  the first surface that has them, inside its rectangle, fading with depth.
- **Tessellation**: a surface is drawn as a grid of at most 256 m cells
  (`MAX_CELL`, at least `segments` a side, at most 256), so a 24 km sea is
  94 x 94 cells, not two triangles; a clipped giant triangle is
  interpolated metres off on llvmpipe, which moved the shore foam
  (`tests/evidence/P3/streaming/foam/NOTES.md`).

Tests and references (`metal`, `vulkan-cpu`): `og-tests/tests/terrain.rs`
and the scenes `terrain-near`, `terrain-mid`, `terrain-far` (the 2 km
island of `og_tests::terrain_scenes` from 2 m, 150 m and 700 m), and
`water-planar`, `water-shadows`, `water-underwater`; each test was broken
once (`tests/evidence/P2/terrain-water/break-and-restore.md`).

## Phase 2: 3D fluids

`Scene::fluids_3d` (`src/fluid3d.rs`, `shaders/fluid3d.wgsl`,
`shaders/fluid3d_blur.wgsl`; og-runtime fills it from every visible
`og.Fluid3D`) draws particle fluids as one screen-space surface. The CPU
uploads each frame's particles as instances (their positions come from
og-physics' CPU solver anyway), so every tier, WebGL2 included, runs the
same passes with no storage buffer or compute. The frame pass `fluid-3d`
runs after the depth prepass and before the main pass, while a fluid has
particles (the prepass and the split main pass are then forced on):

1. sphere imposters write the nearest view depth into `fluid.depth`
   (`fluid.zbuf` for the depth test), discarded behind the prepass's depth;
2. the same spheres add up their chords through each pixel, clamped where
   they pass behind the scene and times the fluid's optical density, and
   their colours, into `fluid.thick`;
3. a separable bilateral filter (two passes, `fluid.blur` then
   `fluid.smooth`, 12 taps each side, at most 24 px) smooths the depth over
   the drawn radius times `smoothing` as seen at that depth;
4. the composite, in the main pass after the scene copy and water, rebuilds
   normals from the smoothed depth, refracts the copied scene by
   `refraction` and absorbs it by the thickness (the exponential of the
   optical depth, what passes taking the particles' colour), lights the body with the sun and its cascades, the
   image-based light and the clustered local lights, reflects the
   environment by Schlick's Fresnel times `reflectivity`, fogs, and writes
   its depth, so later transparent draws and the frame's effects see it.

All the frame's fluids share one surface: they merge where they meet, each
particle carrying its own colour (a group's mix colours them, so poured
fluids blend), but the look (reflectivity, refraction, smoothing) is the
first fluid's. Marching cubes was not built (nor measured): the
screen-space passes cost two blur passes and a composite over the pixels
the fluid covers plus two instanced draws, whatever the particle count, where a polygonised
surface needs a density grid and a mesh rebuilt every frame, on the CPU for
WebGL2.

Cost (`og-gfx --costs-p2`, the `fluid-3d` scene: two fluids, 2,534
particles, about a third of a 1280×720 frame): about 1.1 ms of GPU on an Apple M4 Pro
(metal: +1.08 to +1.17 ms over three runs of 36 draws, 0.5 to 1.3 ms
without it) and +144 ms on lavapipe (`vulkan-cpu`, a CPU rasteriser, 219
vs 75 ms). The CPU's
part is og-physics' step (`crates/og-physics/README.md`).

Tests and references (`metal`, `vulkan-cpu`): `og-tests/tests/fluids3d.rs`
and the scene `fluid-3d` (a blue pool in a tiled basin with an orange drop
above it, lit by the sun and an orange point light); each check was broken
once (`tests/evidence/P2/sph/break-and-restore.md`). fx-showcase's
`fluids3d` station has references on all four tiers.

## Phase 4: environment maps, AgX, exposure adaptation

**Environment maps** (`src/environment.rs`, design §2.6 "IBL from an
environment map prefiltered at cook time"). og-cook's `type environment`
prefilters an equirectangular `.hdr`/`.exr` at cook time with this crate's
`bake` feature (it must match the renderer's face orientation, the
roughness of each level and ibl.wgsl's GGX sampling, so it lives here; the
runtime wasm never enables it): the image resampled 2×2 per texel into the
level-0 cube (three.js's equirectangular mapping: the centre column is +x,
three quarters +z), a box mip chain, levels 1–4 by GGX importance sampling
(Hammersley, the source level picked from each sample's solid angle) at
roughness m/4, and the irradiance projected onto order-2 harmonics by texel
solid angle and convolved with the cosine lobe (band factors 1, 2/3, 1/4,
stored as E/π). Texels are RGB9E5 (`Rgb9e5Ufloat`): 4 bytes a texel
against RGBA16F's 8, sampled and filtered on every tier (WebGPU core, GL ES
3.0 core so WebGL2, Metal, Vulkan's mandatory formats); the map is only
sampled, never a render target. A 128² map is 524 KB before zlib. A KTX2
cube whose own mips are already GGX-prefiltered (og-cook's `prefiltered
true`: cmgen's, IBLBaker's, the glTF IBL sampler's, level k of L at
roughness k / (L − 1)) is not filtered again: `bake::from_prefiltered`
reads the map's level m at lod m/4 · (L − 1), trilinearly between the
file's two nearest levels, turned and scaled, and copies a level whose lod
is a whole level of the map's size texel for texel (a chain of five levels
from the map's size comes through unchanged); the irradiance is projected
from level 0 as above.

`RenderScene::add_environment_map` uploads it as a cube; a frame sets
`environment_map: Some(EnvironmentMap { map, rotation, background_intensity })`.
The IBL generator then copies it, turned about +y, into its own cubes
instead of drawing the sky (`fs_env_copy`: level 0 into the 256² sky cube
scaled by the background intensity, whose mips are downsampled as for the
sky, level m into prefiltered mip m; `fs_env_irradiance`: the harmonics
evaluated into the 32² irradiance cube): 90 passes, as many as the sky's, only when the map, its
turn or background intensity changes. Every shader that reads the
image-based light (pbr, terrain, water, the sky pass, the planar
reflection) reads the map with no change, so `pbr.wgsl` is untouched;
`Environment::intensity` still scales the light and `Background::Sky`
draws the map. Without a map the sky's passes, parameters and WGSL math
are as before: the sky-lit references (`gltf-reference`, `sky-only`,
`showcase`, …) match at max channel difference 0.

**AgX** (`Tonemap::Agx`, `post.wgsl`'s `agx`): linear sRGB to Rec.2020,
the inset matrix, log2 encoding over −12.47..+4.03 EV, the sixth-order
contrast polynomial, the outset, 2.2 back to linear and Rec.2020 to sRGB:
the base look of Blender 4, Filament and three.js `AgXToneMapping`. ACES
stays the default, so no reference moved.

**Exposure adaptation** (`Exposure::Auto { adapt_speed_up,
adapt_speed_down }`, EV per second; the IDL's
`RenderSettings.adaptSpeedUp`/`adaptSpeedDown`): the exposure-adapt pass
(`shaders/exposure.wgsl`) moves a 1×1 `R32Float` EV (two textures, frame
`n` reading the other's and writing `n % 2`'s) towards the metered target
by at most speed × dt, where dt is the scene's tick delta since the
previous adapted frame over `tick_rate`: the tick owns time, nothing reads
a clock, so the same ticks give the same frames however often they are
drawn (a capture at the same tick moves nothing). A speed of 0 is instant
in its direction; the first adapted frame, one after a frame without
adaptation, or one whose tick went back starts at the metered EV. The
adapted EV is world-time state: `Renderer::save_state` copies it (with its
tick) into a 64-texel ring on the GPU, no readback, and `restore_state`
copies it back into both textures; og-probe's snapshot, restore and sweep
call them through `Capture::save_state`/`restore_state` (og-runtime's
`RenderBackend`), so a sweep's values all start from the call's exposure
and a restored world redraws its frames. `og-tests/tests/exposure.rs`: two
renderers adapt identically over 330 ticks after a 4 EV drop, the frame
1.5 s in lies between, it settles within 1 % of instant metering, a
restore to tick 31 redraws tick 120 byte for byte, and zero speeds equal
`Exposure::auto()` bit for bit.

**Where adaptation starts** (web findings #5, wave 13). A frame of tick
0, drawn before the first tick, meters at once and is never adapted
from: the next adapted frame starts afresh. The web shell draws one
behind the play gate (and the reference page one at boot), `og run`
draws none, so the adaptation used to start at a frame one host drew
and the other did not. That frame was also the first a streamed terrain
was drawn in, before any of its tiles had arrived (og-runtime's
streamer now settles a terrain the first frame it is drawn, as a
capture does): Lantern Vale's first frame drew no ground, metered 2.5 EV
brighter than the vale, and the dusk look adapting at 1 EV/s up stayed
under-exposed for its first two seconds, a little less on the web,
whose tick-0 frame metered differently (title RMSE 0.031 and start
0.036 against `metal` on both browser tiers). Now the first ticked frame
meters the whole vale on every host: the title and start references
were re-recorded (RMSE 0.173 and 0.044 against the old ones, the
exposure at its target from the first frame), and a native run that
draws a tick-0 frame first (as the web does) matches one that does not
within RMSE 0.007 at the title. `tests/exposure.rs`: a tick-0 frame of a
16× darker view leaves ticks 1 and 40 byte for byte those of a renderer
that never drew it (fails on the old code at tick 1).

## Phase 4: stylisation

The primitives a look composes into a non-photographic frame
(`src/stylise.rs`; the IDL's `Stylise` and `MaterialStyle` resources, set
by a `type look` asset through `og.render.applyLook`), each off at its
zero value and then costing nothing: no pass is recorded, no pipeline
built. `RenderCapabilities.stylise` names what this renderer draws
(`STYLISE_DRAWN`). All of it draws on both tiers (render passes only).

- **The pixel grid** (`PixelScale`): the scene is drawn at a logical size
  (`height` rows, the width from the output's aspect, both at most the
  output's) into every target the frame has, and the last post pass
  (`upscale`, stylise.wgsl) scales it up by a whole number with
  `textureLoad` (nearest), centred in the output with the camera's
  background colour round it. `Renderer::size` is the output's;
  `width`/`height` inside the renderer are the scene's. The TAA history
  follows the scene size, the offscreen target and the overlay stay at the
  output's. With `snap`, og-runtime moves the 2D camera and every world
  sprite by whole logical pixels. *The fit* (`PixelScale::fit`,
  `PixelFit`): the height decides the scale, `upscale` when `height ×
  upscale` fits the output's height (a hard request, with the border it
  implies), else and for 0 the largest that does; the width is the
  aspect's columns rounded, at most the output's width over the scale, so
  at most one column gives way to the scale. 240 rows at 1280 × 720 were
  427 columns, 1281 pixels at 3x, and fell back to 2x inside a 213-pixel
  border with `upscale` 3 ignored (hollowbell FINDINGS #7); they are 426
  columns at 3x, a 1-pixel border each side. Every other height of the
  look presets fits as before at 720p. Tests: `stylise.rs`'s
  `the_pixel_grid_keeps_its_scale_and_drops_at_most_a_column` (720p,
  1080p, a phone both ways, every fit inside its output); og-tests
  `stylise-palette` and `stylise-retro` (240 rows) were re-recorded at
  3x.
- **A look change is a cut**: when the primitives that are on change
  between two frames (`Stylise::chain_bits`: a pass added or removed, the
  pixel grid, retro 3D, the material style; not their settings), the
  renderer drops the TAA history as `reset_history` does, so the first
  frame of a new look holds no ghost of the old one
  (`FrameStats::taa_history` false there). og-tests `tests/taa_look.rs`:
  the look's first frame equals, byte for byte, the frame of a renderer
  given `reset_history`; a posterize change keeps the history.
- **The chain after the tonemap** (`chain_passes` in post.rs): the tonemap
  writes `ldr.0` (sRGB8) when any pass follows, and each of `paint`,
  `lines`, `quantize`, `fxaa` and `upscale` reads one LDR target and writes
  the next, the last into the output (sRGB-encoded in the shader where the
  output is not an sRGB format). Pipelines are made on first use
  (`PostPasses::ensure_chain`).
  - `paint`: a Kuwahara filter of four sectors (turned and stretched along
    the image's edges by `anisotropy`), pigment pooling at the edges, a
    bleed from a ring two radii out, and a tileable canvas texture laid
    over the frame through a repeating sampler, or on the surfaces
    (`Paint::canvas_space`, below).
  - `lines`: cross-hatching from the tone (one stroke set per level, each
    turned further, over a paper-light ground), halftone dots on a
    rotated grid sized by the tone, and ink on luminance edges (a Sobel
    gradient, widened by the max over the line's width). With
    `Ink::depth` (the IDL's `Stylise.inkDepth`) above 0 the frame runs the
    depth-normal prepass and the lines pass binds its `lindepth` and
    `gbuffer` (stylise bindings 7 and 8; the far-depth and black
    placeholders for the other chain passes and frames without one, and
    the graph declares the reads so nothing aliases them first): a pixel
    is also inked, at that strength, where a neighbour at the line's
    width is farther by more than `Ink::depth_threshold` (4 % by default;
    a silhouette, on the nearer surface) or of about the same depth and
    turned past `Ink::normal_threshold` (1 − cos, 0.4 by default; a
    crease), the outline pass's defaults (the IDL's `inkDepthThreshold`
    and `inkNormalThreshold`), never on the sky. Comic looks get contours
    round flat colours that luminance misses.
  - `quantize`: posterisation in display space with an ordered Bayer 2/4/8
    dither (bit-interleaved in the shader), blue noise or
    interleaved-gradient noise (`DitherPattern::InterleavedGradient`),
    then the nearest colour of a 256-entry palette (`RenderScene::add_palette`,
    a 256 × 1 sRGB texture) by Oklab distance. The blue noise is a 64 × 64
    void-and-cluster tile (`src/blue_noise.rs`, `data/blue-noise-64.r8`,
    4096 B: each of 256 levels on 16 texels, a σ 1.5 Gaussian on the torus,
    integer energies, a fixed seed; its test runs the routine again and
    compares the bytes) uploaded once as `R8Unorm` with the post passes
    (binding 6) and repeated per dither cell: every 8 × 8 block within 8.6
    levels of the middle (31 for the same values shuffled), no
    4-neighbours within 8 levels.
- **Surface space** (`StyliseSpace::Surface`: `Paint::canvas_space`,
  `Hatching::space`, `Halftone::space`; the IDL's `paintCanvasSpace`,
  `hatchingSpace`, `halftoneSpace`, capability `surfaceSpace`): the
  pattern is laid on the surfaces instead of over the frame. The frame
  runs the depth-normal prepass (`Stylise::on_surfaces`) and the paint and
  lines passes bind its `lindepth` and `gbuffer`; each pixel's point is
  rebuilt from its view depth (the inverse projection, the camera's
  rotation) relative to a surface origin, the 1024 m grid point under the
  camera, computed in f64 on the CPU (`post::surface_origin`), so the
  patterns keep their precision far from the world's origin, and its
  world normal weights three planar projections (the planes facing x, y
  and z, weights |n|⁴). The canvas is sampled on the three planes at
  `canvas_world_scale` texels per metre (128 by default) with the level
  of one pixel's footprint, its uv at the origin carried per axis so the
  grain does not jump when the origin moves. Hatching and halftone lay
  the screen pattern's stroke sets and dot grids, same tone thresholds,
  turns and angle, in metres on each plane: the spacing that is
  `scale` (`size`) pixels where the surface lies falls between two
  powers of two, and the coarser octave's lines (dots) are a subset of
  the finer's, so the finer one fades in and out by the fraction between
  them (a tonal art map's levels): the strokes wrap round spheres, stay
  on the objects as the camera moves, and keep about their pixel spacing
  from near to far. The sky keeps the screen pattern. The space is a
  setting, not a pass (`chain_bits` unchanged). og-tests
  `stylise-surface`: a pencil drawing of clay forms (hatching and a paper
  canvas on the surfaces, depth ink with its thresholds set);
  `stylise-halftone-surface`: the toon scene's dots on the surfaces, with
  ink on luminance and depth edges.
- **Object space** (`StyliseSpace::Object`; the IDL's `object` of the same
  three fields, capability `objectSpace`): the surface patterns laid in
  each mesh instance's own space, so a moving, turning or skinned object
  carries its strokes, dots and grain. *Chosen: the object-space
  position, not an instance id.* An id would need every instance's
  inverse transform where the post passes can read it (storage buffers,
  which WebGL2 lacks, or a texture of transforms uploaded again) and a
  free channel in the prepass targets, which has none (normal and
  velocity, depth, outline). Instead, after the prepass the object pass
  (`stylise_object.rs`, `stylise_object.wgsl`; inside the prepass's graph
  pass, which then stores `pre-depth` and also writes `object-space`)
  draws the prepass's opaque instances again, placed by the lit shader's
  `lit_place` as every lit and prepass vertex is, against the prepass
  depth (`LessEqual`, no depth write, a bias of −4 units and −0.5 slope
  towards the camera) into an `Rgba32Uint` target: xyz the bits of the
  position in metres along the instance's axes (the mesh position times
  each model axis's length, so a scaled instance keeps the pattern's
  size; a skinned one its blended bind pose, so the marks follow the
  skin), w its normal there (inverse-scaled, two 15-bit octahedral
  components) with bit 0 set. The paint and lines passes bind it at
  stylise binding 9 (a zero texel on other frames and passes) and, where
  the space's code asks (`StyliseUniform::surface` 2, 1 world, 0 screen),
  lay the triplanar patterns at that position with the world footprint
  for the octaves (rotation keeps lengths) and no world offset for the
  canvas. Pixels the pass leaves at 0 keep the world pattern: terrain and
  scattered copies (not instances) and the vertices a material shader's
  vertex offset moves (drawn without it, they miss the prepass depth);
  the sky keeps the screen pattern. *Limits*: alpha-cut holes are not cut
  in the object pass (a pixel seen through one takes the cut surface's
  coordinates); neighbouring instances meet with unrelated pattern
  phases. *Cost*: a second geometry pass of the opaque instances, only on
  frames a look uses object space; the module (the lit source with the
  built-in hooks) and its pipelines are made the first such frame. og-tests
  `tests/stylise_space.rs` (the hatched scene moved with its camera: 0
  pixels differ in object space, 1.07 % in world space; with the
  placeholder bound instead the object frames differ as the world ones
  do) and the scene `stylise-object` (the surface drawing in object space
  with a box tilted on two axes and a 4 × 0.4 × 0.5 beam: the strokes run
  along each one's faces, square on the beam).
- **Hatching from depth** (`Hatching::depth`, the IDL's `hatchingDepth`,
  0 to 1): the prepass's silhouettes and creases darken the tone the
  strokes follow (`depth_band`, stylise.wgsl), so the strokes gather along
  contours and folds as `inkDepth` inks them; with it the frame runs the
  prepass (`Stylise::depth_edges`), and the weight rides in the uniform's
  `ink_depth.w`. The edges are `Ink`'s thresholds (the outline pass's
  defaults without ink) found 1, half and one stroke spacing away
  (`contour_at`). A silhouette there is a break in the inverse depth,
  which is affine across a plane in screen space: of two opposite
  neighbours, d0/d + d0/d' − 2 is 0 on any plane however steeply it
  recedes and below minus the depth step on the nearer side of a
  silhouette. The ink's own test (a farther neighbour by the step) would
  ink a long floor at the stroke spacing's distance; the ink keeps it,
  byte for byte, at its line width.
- **Depth bands** (`DepthBands`): an HDR pass after the main pass
  (fxpost.wgsl `fs_depth_bands`, over the prepass depth) cuts the scene from
  `near` to `far` into `count` equal layers, the first in the near colour,
  the last (from one band before `far`, and beyond it) in the far colour,
  those between stepping evenly in OkLCh (`band_color`: lightness and
  chroma straight, hue the short way round, a grey end taking the other's
  hue), so a plum-to-gold ramp passes through rose and amber instead of
  the grey-brown a straight line in RGB crosses. `flatten` 1 replaces the
  shading with the band's colour; 0 tints it: the lit colour times the
  band's colour at unit luminance, a coloured filter through which the
  shading, the value and the surfaces' own hues show (it used to be the
  band's colour times the lit luminance, which replaced every hue:
  lantern-vale FINDINGS #19); between, a mix. The outline pass reads the
  banded image. The bands follow view depth, so they read in side and
  three-quarter views; a camera looking down sees its play area at one
  depth, in one band.
- **Retro 3D** (`Retro3d`, pbr.wgsl `retro_clip` and `stylised_uv`): the
  clip position snapped to a screen grid of `vertex_snap` logical pixels,
  depth quantised to `depth_bits`, and affine texturing by interpolating
  uv × w and dividing by the interpolated w in the fragment (no
  `noperspective` qualifier, so WebGL2 has it too). The physical shader
  shares the vertex path; the prepass alpha cut stays perspective-correct.
- **The material style** (`MaterialStyle`, material.rs `styled`): every lit
  material that leaves a stylised field at its default takes the scene's
  (toon and its ramp, the rim, the outline, flat facets), re-uploaded
  whenever the style changes; `shadow_tint` moves shadowed sunlight
  towards a colour (pbr_fs.wgsl and physical.wgsl, an additive term on the
  diffuse), and `outline_distance` scales each material outline's width by
  that distance over the view depth in the prepass (never for an
  orthographic camera). Two exceptions (lantern-vale FINDINGS #16, where a
  look's toon silently switched the lanterns' lacquer and the slimes'
  jelly off): a material with `styled` 0 (the IDL's
  `StandardMaterialDesc.styled false`) is drawn exactly as declared,
  `styled` returning it whole and its uniform's mode bit 8 turning the
  frame's shadow tint and outline distance off for it; and the style's
  toon passes by a material with a physical lobe
  (`StandardMaterial::has_physical_lobes`: a clear coat, sheen,
  transmission, iridescence, anisotropy, an `ior` other than 1.5, or one
  of their maps), since toon shading has none of them: it keeps its
  physical shading and takes the rest of the style (rim, outline, flat
  facets, shadow tint). *Chosen*: a material that names a physical lobe
  has chosen its lighting model, and a scene-wide default must not undo
  that without a word; a game that wants it toon anyway sets its own
  `toon`, knowingly dropping the lobe. og-tests `stylise-toon-physical`:
  under a toon style, a plain red sphere in bands, a lacquered one with
  its coat's sharp highlight, a jelly refracting the stripes behind it,
  and an unstyled one in plain PBR without rim or outline.
- **Hull outlines** (`StandardMaterial::outlineWidth`, the style's
  `outline_width`; fxpost.wgsl `fs_outline`): a material's own outline is
  what toon shading's inverted hull draws, the object's silhouette (a
  farther neighbour at the line's width), where one part of it passes in
  front of another, and its creases against *other* objects (a box on
  the floor, a character against a wall), never the edges between its
  own faces. The prepass writes each fragment's object id (pbr.wgsl
  `object_id`: 1 to 255, a hash of the instance's transform, the skinned
  instance's and not its joints'; 0 for terrain and scattered copies)
  into the outline target beside the colour and width (Rgba8Unorm read
  as bytes: the colour in 5:6:5 over r and g, the width in eighths of a
  pixel, the id; `outline_bytes`), and the outline pass lines a crease
  only where the ids differ. *Chosen over smoothed outline normals*: an
  outline here is a screen-space pass over the prepass, not an extruded
  shell, and its crease test inked every edge of an unwelded mesh (a
  `.vox` model's greedy quads, a flat-shaded rock): a voxel canopy became
  a net of lines and a small voxel slime all ink (lantern-vale FINDINGS
  #15). Smoothed normals would need a vertex stream and a prepass channel
  the prepass has no room for (WebGL2's sixteen attributes are spent on
  the skinned prepass) and still ink small hard-edged meshes whose
  smoothed normals turn within the line's width; the id keeps every face
  of one object clean whatever its topology, skinned included, at no
  extra target. Neighbouring objects share an id one time in 255 (their
  contact line then goes). The scene's outline (`PostEffects.outline`)
  still lines every crease. og-tests `stylise-outline-voxels`: a voxel
  tree, a voxel mound and a stepped block (unwelded, flat-shaded) and a
  smooth sphere under a 2.5 px toon outline, each a clean silhouette
  with its contact line on the ground.

- **Built-in canvases and palettes**: a look names the engine's own
  `"@canvas/paper"`, `"@canvas/linen"`, `"@canvas/watercolor"` (tileable
  256² grey, the paint pass reads the red channel) and `"@palettes/lcd2"`,
  `handheld4`, `dusk16`, `bright32`, `1bit`; the cook adds the ones its
  looks name to the bundle under reserved ids (og-cook's README, *Built-in
  look assets*), so the renderer sees an ordinary texture and palette.
  watercolor-wash, paper-craft and painterly-3d lay the canvases,
  monochrome-pixel and monochrome-dither-3d quantise to `handheld4` and
  `1bit`.
- Beside the frame's stylisation: sprite drop shadows ([The 2D
  batch](#the-2d-batch-e4)) for pixel sprites in 3D sets, on terrain,
  decks and steps and faded inside the cascades' shadow, and og-anim's
  stepped playback (`AnimationPlayer.stepFps`) for stop motion on twos.

Scenes: `stylise-pixel`, `stylise-palette`, `stylise-print`,
`stylise-halftone`, `stylise-paint`, `stylise-bands`, `stylise-retro`,
`stylise-material`, `stylise-ink-depth` (flat unlit shapes of one colour
where every line is a silhouette or crease) and `sprites-shadow` (og-tests,
`metal` and `vulkan-cpu` references), `stylise-surface` and
`sprites-shadow-terrain` (round three, `metal`; the latter moves in round
four with its hollow), `stylise-object` and `stylise-halftone-surface`
(round four; recorded by the round's merge), and fx-showcase's `looks`
station, one frame per preset of `templates/looks/`, which the browser
tiers compare too (`reference.ts --looks` in `scripts/check.sh
web-reference`, every frame against `metal`). The station's HUD line
names the preset shown (`P4 station  looks  <slug>`, `none` before a
look; `look:<slug>` sets it), so each frame identifies itself. Every style
of the catalogue (`packages/knowledge/looks/catalogue.json`) is now
`full` or `content`; `inked-comic-3d` hatches its shadows on the surfaces
(`hatchingSpace surface`, `hatchingDepth` 0.7) in place of the screen
halftone, as its check asks (ink marks painted onto surfaces).
Round five adds `stylise-outline-voxels`, `stylise-toon-physical`,
`sprites-untonemapped` and `stylise-bands-tint` (`metal`).

Round three delivered what round two left: hatching and halftone on the
surfaces (strokes follow the tone and the form), the canvas on the
surfaces, drop shadows on terrain and explicit heights that fade inside
the cascades' shadow, and `inkDepth`'s thresholds as fields. Round four
delivered object space (a moving object carries its marks), drop
shadows bent over hollows and ridges narrower than themselves, hatching
from depth (`hatchingDepth`), the halftone-on-surface scene, a preset in
surface space (`inked-comic-3d`) and the looks frames' HUD naming their
preset. Still open: the octave fade shows as faint bands on a long
receding floor (the footprint is measured across the pixel, not along
the depth gradient; `stylise-halftone-surface` shows it on the far
floor); object space leaves terrain, scattered copies and vertex-offset
materials in world space and does not cut alpha holes; a bent shadow
bridges features narrower than a cell of its 4 × 2 grid.

## What phase 2 owes

Delivered in P4: clearcoat maps, sheen, transmission (with volume and
`ior`), iridescence and anisotropy on both tiers ([Physical
materials](#physical-materials-p4)); cascade blending, PCSS soft shadows
for the sun, point shadows by tier (dual-paraboloid on WebGL2), up to 8
shadowed local lights (4 on WebGL2), TAA's Catmull-Rom history and
sharpening, FXAA on every tier, and the CPU particle path's collision
(WebGL2's sparks bounce) ([Shadows and antialiasing by
tier](#shadows-and-antialiasing-by-tier)).

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

- **Render graph**: a depth prepass for every frame. *Decided (P4 perf):
  on demand.* The depth-normal prepass runs when an effect reads it (AO,
  outlines, depth of field, motion blur, god rays, soft particles, water,
  3D fluids). Forced on every frame it changes no lit pixel (the lit pass
  keeps its own depth test, and the clusters come from light volumes, not
  scene depth) and costs a second geometry pass: spheres-20k 9.98 to 16.56
  ms (+66 %), city-street +0.3 ms, city-aerial +0.5 ms, showcase within
  noise (`tests/evidence/P4/perf/prepass.md`). Design §2.6 says so.
- **GL-tier references**: native GL needs wgpu's `gles`/`angle` features
  (Cargo.lock change, ANGLE dylibs on macOS) and was not set up, so there
  is no native `gl` tier. *The browser half is done (phase 2, owed row
  22):* og-tests-web, a wasm harness of og-tests' scenes, draws them on
  `webgpu` and `webgl2` in `make web-reference` (56 scenes per tier in the
  P4 UI shell round's run), judged against `metal` unless the tier keeps a
  reference by design, beside `templates/fx-showcase`.
- 2D: billboards (camera-facing world sprites; particles have them).
  (Fog landed in E5, in the lit, sky, particle, strip and sprite shaders
  rather than as a post pass (§E5: atmosphere); per-texture filtering is
  `set_texture_filter`, nearest for pixel art. Kerning,
  shaped placements and glyph tables that grow at run time landed in the
  phase 4 UI text round; shaping and bidi themselves are og-ui's, CJK
  coverage the cook's `runtime true` with og-ui's run-time glyphs.)
- *Delivered (P4 perf, `tests/evidence/P4/perf/`):*
  - **Instance upload dirty tracking** (12749c1): the CPU path's instance
    buffer and GPU culling's input region keep a copy of what they hold and
    write only the 4 KiB blocks that changed (`src/upload.rs`), on both
    tiers; a buffer just grown, or a frame of another length, is written
    whole. `FrameStats::instance_upload_bytes`; og-tests `tests/upload.rs`.
  - **Resources can be removed** (db105bd): `RenderScene::remove_mesh`,
    `remove_texture`, `remove_material`, `remove_font` free the slot on the
    GPU before the next frame; the next add of the kind reuses it under a
    new generation (handles carry the slot in 24 bits and an 8-bit
    generation), so a removed handle is refused by every setter and, in a
    frame's instances, sprites, texts or font atlases, by `render`
    (`InvalidHandle`). Removing a texture clears the material slots that
    sampled it. A skinned or morphed mesh's slot is retired, not reused.
    og-runtime unloads a streaming group's model textures through
    `remove_texture`. og-tests `tests/removal.rs`.
  - **Shader cooking per backend** (912e333, 8779ca8): every WGSL module
    of both tiers (`og_render::shader_sources`) is validated by naga and
    written as MSL and SPIR-V (full tier) or GLSL ES 3.0 (GL tier) in the
    gate (`tests/shaders.rs`), so a shader error is a failed build, not a
    player's crash. Compiled pipelines are kept across runs where wgpu has
    a pipeline cache (Vulkan; `set_pipeline_cache_dir`,
    `RenderInfo::pipeline_cache`); Metal and the browsers keep their own.
    The wasm still ships WGSL: the browser compiles WGSL itself, and a
    second form would only add to the download.
  - **Cluster build in compute**: measured, not kept. For `lights-256` the
    CPU build takes 38 to 53 us a frame; the cheapest compute build (one
    dispatch, no importance order, so not even byte-identical) takes 97 to
    134 us to upload, encode and submit and 279 to 385 us of GPU on Metal
    (`tests/evidence/P4/perf/clusters-compute.md`,
    og-tests `tests/cluster_compute.rs`). The CPU build stays on both tiers.
  - **The lit pass's per-instance GPU cost** (owed row 50's remainder,
    branch `r-lean`): plain materials draw through lean lit, prepass and
    shadow pipelines ([Lit pipeline variants](#lit-pipeline-variants)):
    10 interpolated floats instead of 23, only the attributes read, no
    discard where no derivative is taken. spheres-20k natively 7.1 to
    4.6 ms (GPU-bound to about CPU-bound in the dev build), the §7
    reference scene unchanged, every Metal `og-gfx --all` frame
    byte-identical to main's (`tests/evidence/P4/lean/`). Then (branch
    `render-parity`, `tests/evidence/P4/parity/`): instances are 52 bytes
    on the GPU, plain meshes can be 16-byte vertices (opt-in), the sort and
    the bounding spheres are kept across frames, wgpu's indirect-call
    validation is off natively and objc2 has no debug assertions in the
    dev profile; every Metal `og-gfx --all` frame byte-identical. The
    field is GPU-bound natively, on its 7.3 million triangles a pass. The
    browser comparison with three.js (`tools/bench/threejs-compare.ts`)
    has not been run again.
