# og-render

The Origin Engine renderer (`docs/design.md` §2.6): wgpu, WGSL shaders in
`crates/og-render/shaders/`, one code path for native (Metal, Vulkan,
DX12, headless for executors) and the browser (WebGPU only).

The crate knows nothing about the ECS. Its whole input is a `RenderScene`;
the world → scene extraction belongs to the layer above.

## Public types

| Type | What it is |
|---|---|
| `RenderDesc` | How to create a renderer: `width`, `height`, `backend` (`Auto`: DX12 on Windows, Metal or Vulkan elsewhere, WebGPU in a browser; `WebGpu`, `Metal`, `Vulkan`), `output_format` (`None`: `Rgba8UnormSrgb` offscreen, the surface's choice on a surface), `hdr` with `paper_white_nits` (0: 203) and `peak_nits` (0: 1000), `antialias`, `shadow_map_size` (2048), `shadow_cascades` (at most `MAX_CASCADES`, 4), `texture_compression`, `particles`, `culling`, `point_shadows` (`PointShadows::{Auto, Cube, Paraboloid}`), `taa_history` (`TaaHistory::{Bilinear, CatmullRom}`), `taa_sharpen` (0 to 1), `volumetric`, `planar_reflections` (1 or 2, `MAX_PLANAR_REFLECTIONS`), `transparency`, `transparency_layers`, `low_power`, `timestamps`, and the test switches `without_features` and `indirect_per_draw`. Default 1280×720, `Auto`, `Antialias::Off`, point shadows `Auto`, a bilinear TAA history, no sharpening, one planar reflection. `RenderDesc::mobile()` is the phone preset. |
| `Antialias` | `Off`, `Tier` (the engine's default method: TAA on every backend), and the explicit `Msaa4`, `Taa`, `Fxaa`, `Smaa` and `SmaaT2x`. `RenderInfo::antialias` reports what was resolved; `Msaa4` falls back to `Fxaa` where the HDR format cannot be multisampled ([Shadows and antialiasing](#shadows-and-antialiasing)). |
| `Renderer` | `Renderer::new(&desc).await` (headless), `Renderer::with_surface(&desc, &instance, surface).await` (presents), `render(&scene, &target) -> Result<FrameStats>`, `resize`, `info`, `reset_history` (TAA, on a camera cut), `save_state() -> RendererState` and `restore_state(state) -> Restored` (the display's frame history: the adapted exposure, the TAA, SMAA, GI, SSR and FSR histories, wind, trample, the frame counters and terrain fades, copied on the GPU for a world snapshot, 64 saves within 768 MiB; `RendererState::id`/`from_id` carry it as a u64; `Restored::Afresh` when the save was evicted), `capture().await -> RgbaImage`, `poll()` and `warm()` on every host. Native only: `new_blocking`, `wait_idle`, `read_offscreen()` (= `capture` blocking). |
| `renderer_disabled()`, `RENDERER_SWITCH` | Native: `OG_RENDERER=none` makes `Renderer::new` and `with_surface` answer `NoAdapter` naming the switch, as on a machine without a usable adapter, so a host's no-renderer path is testable beside a GPU (`docs/ci.md` §Switches in the environment). |
| `render_to_rgba(&desc, &scene)` | Native, blocking: one frame headless → `RgbaImage` (RGBA8, rows top to bottom). |
| `RenderTarget` | `Offscreen` (the renderer's own texture, readable), `Surface` (acquire, draw, present), `View(&TextureView)`. |
| `RenderScene` | Resources with handles (`add_mesh`, `add_texture`, `add_texture_source`, `add_material`, `add_font`, `add_terrain`, `add_terrain_stream`, `add_environment_map`, `add_texture_array`, `add_palette`, `add_lut`; `set_material`, `set_terrain`, `set_terrain_stream`, `set_terrain_tile` replace one; `remove_*` free one: [Resources and removal](#resources-and-removal)), and the per-frame public fields `instances`, `camera`, `light`, `lights`, `environment`, `environment_map`, `post`, `stylise`, `quality`, `sprites`, `sprite_arrays`, `texts`, `overlay`, `debug`, `terrains`, `water`, `particles`, `colliders` (`SceneCollider::{Plane, Box, Sphere, HeightField}`: what CPU-simulated particles bounce off), `tick`, `tick_rate`. `mesh_handle(index)` turns a mesh index back into its handle. The renderer uploads what is new or changed and frees what was removed before the next frame. |
| `MeshData`, `Vertex`, `TextureData`, `Material` | Triangle lists (a vertex's `tangent` in glTF's frame, generated at upload when `w` is 0: `MeshData::generate_tangents`); RGBA8 with `color_space` and `mips`; glTF metallic-roughness with `alpha_mode` (`Opaque`, `Mask`). |
| `VertexEncoding` | How a mesh's vertices are held on the GPU: `Full` (default, `Vertex` as given, 64 bytes) or `Compact` (16 bytes where a plain material draws it; quantised): `RenderScene::set_vertex_encoding(mesh, encoding)`, `vertex_encoding(mesh)` ([Instance and vertex encodings](#instance-and-vertex-encodings)). |
| `StandardMaterial`, `PhysicalExtensions` | The reflected three.js material (`MeshStandardMaterial` fields, `MeshPhysicalMaterial`'s clearcoat, sheen, transmission, iridescence, anisotropy and `ior`: [Physical materials](#physical-materials)); glTF's `KHR_materials_*` extensions of one material, which `apply` lays over a `StandardMaterial` as three.js's `GLTFLoader` does. `mapArray`/`mapLayer` and `normalMapArray`/`normalMapLayer` read the base colour and normal from texture-array layers ([Array maps](#array-maps-materials-and-terrain-layers)). |
| `Instance` | `(mesh, material, transform, material_layer, fx, cast_shadows, receive_shadows)`, submitted one by one; batched by (mesh, material, cast_shadows). `material_layer` (`None`: the material's) is the texture-array layer an array-map material samples for this instance. `fx` (`InstanceFx { flash, flash_color, tint }`, `InstanceFx::NONE` the material's own look; `SkinnedInstance::fx` the same): [Instance looks](#instance-looks-the-hit-flash-of-one-instance). `cast_shadows` and `receive_shadows`, both on from `add_instance` and `SkinnedInstance::new`: [Shadows and antialiasing](#shadows-and-antialiasing). The transform is affine; the GPU record is 64 bytes ([Instance and vertex encodings](#instance-and-vertex-encodings)). |
| `Camera` | `position`, `target`, `up`, `projection` (`Perspective { fov_y_degrees }`, `Orthographic { height }`), `near`, `far`, `exposure`. `Camera::orthographic_2d(center, height)` is the 2D camera (looking down −z at the z = 0 plane). |
| `Exposure` | `Manual { ev }` (×2^ev; `Exposure::linear(k)` from a multiplier) or `Auto { compensation, min_ev, max_ev, adapt_speed_up, adapt_speed_down }`: the frame's log-average luminance brought to 0.18. With both speeds 0 (`Exposure::auto()`) from the current frame only; otherwise adapted over the scene's ticks ([Exposure adaptation](#environment-maps-agx-exposure-adaptation)). |
| `DirectionalLight` | `direction_to_light`, `color`, `intensity`, `cast_shadows`, `shadow_bounds` (`Auto`: the instances' bounds; `Sphere`), `max_shadow_distance` (default 200 m; shadows fade over its last tenth), `shadow_softness` (PCSS: the penumbra's width per metre between caster and receiver; 0, the default, is the hard edge) and `cascade_blend` (the fraction of each cascade's depth over which it fades into the next; 0 by default). |
| `Light`, `LightKind` | Point and spot lights (`Point`; `Spot { direction, angle_degrees, penumbra }`) with `position`, `color`, `intensity` (radiance at 1 m), `range`, `cast_shadows`. Clustered ([Forward+](#forward)): lights whose range misses the view are culled (`FrameStats::lights_culled`), up to `MAX_LIGHTS` = 256 in view are drawn (past it the brightest at the camera, the rest counted in `lights_dropped`); of those with `cast_shadows`, at most the quality's `shadowed_lights` within `RenderInfo::shadowed_lights` (`MAX_SHADOWED_LIGHTS` = 16) get shadows, the most radiant at the camera first (the rest counted in `shadowed_lights_dropped`). |
| `PostEffects`, `Bloom`, `Tonemap` | `bloom: Option<Bloom { threshold, knee, intensity }>` (threshold in exposed units: 1.0 is display white), `tonemap: Aces` (default), `Agx` (Blender 4's and three.js's AgX, base look), `Neutral` (Khronos PBR Neutral, three.js `NeutralToneMapping`) or `None` (clamp: authored 2D colours come out exact). |
| `EnvironmentMapData`, `EnvironmentMap`, `bake` | A prefiltered environment map (level 0's face `size`, `ENVIRONMENT_MAP_LEVELS` = 5 levels of RGB9E5 `texels`, level m at roughness m/4, `irradiance` as nine harmonics) and how a frame uses one (`map`, `rotation` about +y in radians, `background_intensity`); `rgb9e5`, `rgb9e5_decode`, `sh9`, `face_direction`; feature `bake`: `bake::bake(&Equirect, &BakeDesc)`, `bake::bake_cube` and `bake::from_prefiltered(&PrefilteredCube, &BakeDesc)` ([Environment maps](#environment-maps-agx-exposure-adaptation)). |
| `Sprite` | A textured quad in the world xy plane facing +z: `texture` (`None` = solid), `position`, `size`, `anchor`, `rotation`, `color` (linear RGBA), `uv_rect` (v down; a flipped region swaps u0/u1 or v0/v1), `layer`, `clip` (an index into the overlay's clip rectangles; ignored in the world). Sorted by (layer, sort key, texture); within those, submission order. `anchored_at(pivot)`: the same quad anchored elsewhere. |
| `SpriteFacing`, `TextFacing`, `sun_card_right` | Billboards: `sprite_facing` (by sprite index) `None` (the xy plane), `Camera` (the view plane), `CameraY` (upright, turned about y); `RenderScene::text_facing` for world text ([Billboards](#billboards)). |
| `SpriteArray`, `TextureArrayData`, `TextureArrayInfo` | `sprite_arrays` (and `Overlay::sprite_arrays`): a sprite drawn from layer `layer` of a texture array (`add_texture_array`: same-sized layers uploaded as one `D2Array`; `set_texture_array_filter` for pixel art) instead of its texture; `ParticleEmitter::array` draws a 2D emitter's particles from one ([The 2D batch](#the-2d-batch)). |
| `SpriteShadow`, `ShadowGround`, `ShadowBend`, `SPRITE_SHADOW_LIFT`, `SHADOW_BEND_ROWS`, `SHADOW_BEND_COLUMNS`, `SHADOW_BEND_TOLERANCE` | `sprite_shadows`: a world sprite's drop shadow along the sun onto its `ground` ([The 2D batch](#the-2d-batch)). |
| `SpriteOrder`, `TextOutline`, `TextureFilter` | Per-frame side lists by index, so `Sprite` and `Text` keep their fields: `sprite_order` gives a sprite a sort key within its layer (top-down y sorting: `-y` of the feet; default 0), `text_outlines` an outline (width as a fraction of half the font's distance range, linear colour) to a world text. `set_texture_filter(texture, Nearest)` samples a texture by its nearest texel wherever sprites, tiles and particles draw it (pixel art); `CleanEdge` runs torcado's cleanEdge (MIT, `third_party/cleanedge/`) in sprite.wgsl, so a turned pixel sprite or tile keeps its texels' colours and its edges are redrawn as slopes (texture arrays, the overlay and particles take it as nearest). |
| `SpriteOutline`, `paletted_texture` | The art contract's runtime pieces (`docs/art.md` §2.4). `sprite_outlines` gives a world sprite a pixel outline drawn by sprite.wgsl from its texture's alpha: the clear logical pixels within `width` (a diamond) of an opaque one take its colour, a logical pixel `pixel` texels, the quad grown by the width, texels outside the sprite's region clear. `paletted_texture(index, palette, row)` is a palette swap: an index texture as the cook writes `<id>#index` (alpha the palette index plus one) through row `row` of a palette texture, made once per (index, palette, row) on the CPU and drawn as any texture. |
| `TileLayer`, `TileChunk`, `TileQuad`, `TileAnimation` | `tile_layers`: chunked tile quads (shared `Arc<Vec<TileChunk>>`, so an unchanged layer copies no tiles) drawn through the sprite batch before the sprites of their layer, chunks culled to an orthographic view, with `repeat` per axis (scrolling backgrounds) and animated quads cycling uvs by `time`. `FrameStats::tiles` / `tile_chunks`. |
| `nine_slice(sprite, insets, texture_size, world_per_pixel)` | The nine sprites of a nine-slice panel: corners keep their size, edges stretch along one axis, the centre along both; insets clamp to half the sprite. |
| `MsdfFont`, `Glyph`, `Kern`, `Text`, `GlyphPlacement` | An MSDF atlas (`TextureHandle`, linear, no mips) plus `distance_range`, `line_height`, the glyph table (`advance`, `plane_bounds` in em y up, `atlas_bounds` in pixels y down: msdf-atlas-gen's layout) and `kerning` (`Kern { left, right, adjust }` em). `Text { font, text, position, size, rotation, color, layer, clip, tracking, weight, glow, placements }` lays out left to right with `\n`, each glyph advancing by its advance, the kerning with the character before it and `tracking` em (CSS `letter-spacing`), world and overlay alike (`FontLayout::walk`); a shaper's `placements` (one `GlyphPlacement { advance, x, y }` em per character, as og-ui hands a bidi or shaped line) replace those. Missing characters take no space, break the kerning pair and are counted in `FrameStats::missing_glyphs`. `MsdfFont::measure` is a line's width. `RenderScene::set_font` replaces a font's glyph table (glyphs rendered at run time); fonts carry a revision and the renderer re-lays them out. Text sorts with the sprites. `weight` and `glow` are the overlay's ([The overlay](#the-overlay)). |
| `Overlay` | The screen-space layer drawn last (`RenderScene::overlay: Option<Overlay>`): `sprites`, `texts`, `sprite_arrays` and `glass` in CSS pixels, origin top left, y down; `width`, `height` (the CSS viewport) and `scale` (device pixels per CSS pixel; 0 fits the height); `clips` (`add_clip([x, y, w, h])` → the index a sprite or text names in `clip`). |
| `TerrainData`, `TerrainControl`, `TerrainHandle` | A terrain's geometry: `columns` × `rows` heights (m) over a `size` rectangle centred on its origin, row-major from -z and -x, and an optional RGBA8 control map (one channel per layer). `height(local)` (on the triangles the renderer and physics use), `normal(local)`, `range()`, `cell()`, `check()` ([Terrain](#terrain)). |
| `Terrain`, `TerrainLayer`, `TerrainLod` | A terrain drawn this frame (`RenderScene::terrains`): its data, its `transform`, up to `MAX_TERRAIN_LAYERS` = 16 layers (four with textures of their own, sixteen from `layer_arrays`; a pixel blends its four strongest), `height_blend`, `cast_shadows`, and the clipmap's `lod` (finest `spacing`, `ring` quads per side, `budget`). |
| `RenderQuality`, `QualityPreset` | What a frame may cost, read every frame ([What a frame may cost](#what-a-frame-may-cost-renderquality)). |
| `DebugView` | `None`, `Cascades` (red/green/blue per cascade, black beyond), `AmbientOcclusion`, and others listed on the enum. |
| `FrameStats` | `draw_calls` (every pass), `batches`, `instances`, `culled`, `triangles` (main pass), `shadow_passes`, `shadow_cached`, `shadow_casters`, `lights`, `lights_dropped`, `lights_culled`, `cluster_lights`, `cluster_lights_dropped`, `sprites`, `sprites_culled`, `glyphs`, `sprite_draws`, `missing_glyphs` (overlay ones included), `overlay_sprites`, `overlay_glyphs`, `overlay_draws`, `texture_bytes`, `instance_upload_bytes`, `frame`, `taa_history` (whether TAA blended history this frame), `shadowed_lights`, `shadowed_lights_dropped`, `particle_collision_skipped`, `terrain_draws`, `terrain_triangles`, `terrain_shadow_draws`, `terrain_tiles`, `terrain_uploads`, `scatter_draws`, `scatter_copies`, `scatter_built`, `reflection_draws`, and GPU culling's counts ([The render graph](#the-render-graph)). |
| `RenderInfo` | Adapter, backend, device type, `antialias`, `msaa_samples`, `shadow_map_size`, `shadow_cascades`, `shadowed_lights`, `point_shadows` (resolved: `Cube` or `Paraboloid`), `planar_reflections`, `transparency`, `texture_formats`, `pipeline_cache`, `hdr_supported`, `hdr`, and `backend_tier()`: the reference-frame directory name and the tier the probe's hello and `og.render.capabilities` report (`tier_name(backend, device_type, adapter)`: `vulkan-cpu` for a software rasterizer, `is_software_adapter`: a CPU device type, or an adapter named llvmpipe, lavapipe or SwiftShader). |
| `RenderError` | `NoAdapter`, `NoDevice`, `Surface`, `SurfaceOccluded`, `InvalidHandle`, `Readback`, `Unsupported`, `InvalidData`, `DeviceLost`. |
| `shapes` | `sphere`, `plane`, `cuboid(size)`, `tetrahedron`, `checker`. |
| `gltf` (feature `gltf`) | `load_glb(bytes) -> GltfModel`, `GltfModel::add_to(&mut scene) -> ModelParts`, `ModelParts::instance`; `gltf::physical_extensions`. Games load cooked models; this bridge serves tests and tools. |
| `wgpu` | The wgpu this crate is built on, re-exported for hosts. |

## Resources and removal

Every resource kind a game streams has a generation-tagged handle: a
handle's low 24 bits are its slot, the top 8 the slot's generation.
`remove_mesh`, `remove_texture`, `remove_material`, `remove_font`,
`remove_environment_map`, `remove_texture_array`, `remove_palette` and
`remove_lut` empty a slot and move its generation on; the renderer frees
the GPU copy before the next frame (by slot revision: a refilled slot is
uploaded again, an emptied one dropped), and the next add of the kind
fills the slot again under the new generation, so an old handle never
reaches the new resource. A skinned or morphed mesh's slot is retired,
not reused.

A stale handle is refused everywhere: the setters (`set_mesh`,
`set_material`, `set_font`, `set_texture_filter`,
`set_texture_array_filter`, ...) return false, the getters (`mesh`,
`font`, `environment_map_data`, `texture_array`, `palette`, `lut`) `None`,
`contains_*` false, and `render` refuses a frame that still names one
(`RenderError::InvalidHandle`, naming it: an instance's mesh or material, a
sprite's or an overlay sprite's texture or array, a 2D emitter's array, a
text's font or a live font's atlas, `environment_map`, the colour
grading's LUT, the stylisation's palette, a terrain's layer array). A
removed texture is also cleared from every material slot and the decal
atlas, a removed texture array from every material's `mapArray` and
`normalMapArray` (they draw without it). Removing what the frame still
uses is the caller's to avoid: og-runtime's streaming keeps such a
resource, with a warning, until the frame stops using it.

`FrameStats::texture_bytes` counts textures (every level as allocated),
arrays (padding layers included), environment maps (every level of the
six RGB9E5 faces), palettes (256 × 1 RGBA8) and LUTs (32³ RGBA8); removed
ones count nothing. The image-based light is keyed by the map slot's
revision, the stylisation's bind group by the palette's, so a refilled
slot never draws the old resource. Tests: og-tests `tests/removal.rs`.

## The render graph

A feature-gated pass sequence per `render` call, run on the render graph
object (`src/graph.rs`: declared targets and passes, data-flow order,
aliased transients). Passes whose feature is off are not recorded. An
effect pass declares every prepass texture it binds (`lindepth`,
`gbuffer`), not only its own input, so the graph never aliases a target
a pass still reads.

1. **Sync.** New meshes, textures (with the **mip pass**: one fullscreen
   triangle per level, linear-space box filter), materials, fonts. When the
   sky changes, the **IBL passes** run (sky cube, irradiance, GGX prefilter,
   BRDF LUT; render passes only); with an environment map they copy the
   cooked map into the same cubes instead. Instance buffers write only the
   4 KiB blocks that changed (`src/upload.rs`, `FrameStats::
   instance_upload_bytes`): on the CPU path the sorted records of the
   batches a list draws whole and the culled copies appended after them; a
   buffer just grown, or a frame of another length, is written whole.
2. **Batch and cull** (CPU). Instances stably sorted by (mesh, material);
   each sorted instance gets a world bounding sphere. Both are kept across
   frames (`instances.rs`): the sort is redone only when the submission's
   (mesh, material) sequence or a drawn mesh's bounds changed, an
   instance's sphere and GPU transform only when its transform changed
   bit for bit. Every pass gets a draw list per batch: the instances whose
   sphere touches the pass's volume (`Frustum::test_spheres`; the lists
   are the same on any number of threads). The tests run on the calling
   thread below 131,072 bounds and on the lanes from there (og-lanes'
   `cull` site, 1,024 bounds a task). A batch that loses less than a
   tenth of its instances is drawn whole; otherwise its visible transforms
   are appended to the instance buffer. Draws bind a buffer slice. Sprites
   and glyphs are sorted by (layer, kind, texture or font) into runs.

   **GPU culling** (`cull_gpu.rs`; `RenderDesc::culling`: `Culling::Auto`
   by default, which takes it from `GPU_CULLING_MIN_INSTANCES` (16,384)
   instances, `Culling::Gpu` at any size) replaces the per-pass lists with
   compute and indirect draws: a compute stage tests the sorted instances
   against every shadow face, the planar reflection and the camera, last
   frame's visible opaque occluders are drawn at this frame's camera into a
   depth pyramid, and a second stage drops camera instances whose box lies
   behind it. Each pass's visible instances are compacted in sorted order
   by a deterministic prefix sum and every batch is one
   `draw_indexed_indirect`, so both paths draw the same frame.
   `Culling::Cpu` (no compute dispatch or readback for a small frame),
   object motion blur and frames over the storage-binding limit keep the
   CPU path. The counts come back through a readback ring for frames after
   `Renderer::request_culling_counts`: `FrameStats::gpu_culling`,
   `occluded`, `occluded_triangles`, `in_view_triangles`, `occluder_draws`
   and `counts_lag` (the GPU totals lag by that many frames;
   `finish_stats` waits for the frame's own natively). wgpu's
   indirect-call validation is off (`Backend::instance`): cull.wgsl writes
   the arguments in range by construction. `og-gfx` takes
   `OG_GFX_CULLING=gpu|cpu|auto`; `og-tests/tests/culling.rs` renders
   eight scenes both ways.
3. **Cascaded shadows** for the directional light (`frame.rs`): splits
   between the near plane and the nearer of `max_shadow_distance` and the
   far side of `shadow_bounds`, λ = 0.8 between logarithmic and uniform;
   `RenderQuality::shadow_cascades` of them (3 by default, at most
   `RenderInfo::shadow_cascades`). Each cascade is the bounding sphere of
   its camera slice (its size does not change as the camera turns), its
   centre snapped to the shadow map's texel grid in light space (texels do
   not swim), its depth range reaching towards the light to the top of the
   scene bounds. Casters are culled to the cascade's box **and** to the
   camera slice swept towards the light. One pass per cascade into a
   `shadow_map_size`² `R32Float` array (depth also written as colour) with
   a shared depth buffer, front faces culled.
4. **Local shadows**: per shadowed light, six cube faces or two
   paraboloid hemispheres (point, by `PointShadows`) or one perspective
   view (spot), each a pass into a `LOCAL_SHADOW_SIZE` (512)² array layer,
   casters culled to the face frustum. With terrain, each of its clipmap
   levels that reaches into the face's or cascade's volume is drawn too;
   with a planar water reflection the **reflection pass** follows the
   shadows ([Water](#water)). While a 3D fluid has particles the
   **fluid-3d pass** follows ([3D fluids](#3d-fluids)).
5. **Depth-normal prepass**, on demand: it runs when something reads it
   (AO, outlines, depth of field, motion blur, god rays, soft particles,
   water, 3D fluids, the stylisation's depth ink and surface patterns).
   It changes no lit pixel; it costs a second geometry pass.
6. **Main pass** into the `Rgba16Float` HDR target (multisampled with a
   resolve when MSAA is on): forward PBR (`pbr.wgsl`) with the directional
   light, its cascade picked by view depth (the same slices its casters
   were culled to), the point and spot lights of the fragment's cluster
   ([Forward+](#forward); inverse square windowed to the range, smooth
   spot cone), split-sum IBL, emissive, alpha mask, normal maps in the
   vertex tangent frame. The specular lobe is GGX with the
   height-correlated Smith visibility for alpha = roughness² and no
   roughness remap (`ggx.wgsl`, one function for the lights and for the
   BRDF LUT's integral, so direct and image-based specular agree). Shadow lookups are bilinear PCF with explicit
   compares on `textureLoad`. **Shadow bias works at any scale:** the
   normal offset and the depth bias are in world units proportional to the
   cascade's texel size (the depth bias then converted to that cascade's
   normalized depth). The **sky pass** draws the sky cube at the far plane
   over uncovered pixels. **Sprites and text** follow, alpha blended,
   depth-tested against the scene without writing depth: one instanced draw
   per run, quads built from the vertex index; text uses the MSDF median
   with a screen-space distance range from `fwidth`. With water,
   transmission or particle distortion the main pass splits after the sky
   and copies the resolved scene (`scene-copy`, the `transmission` copy).
7. **TAA** (when on): a Halton(2,3) jitter of 8 positions is applied to the
   projection as a clip-space translation; the TAA pass reprojects the
   history through the depth buffer with the unjittered matrices, clamps it
   to the current 3×3 neighbourhood in YCoCg and blends 10 % of the current
   frame with luminance weights. History ping-pongs between two targets and
   restarts on resize, a new scene, a look change or `reset_history`.
8. **Metering** (auto exposure): log2 luminance into 64², then 8×8 averages
   to 8² and 1² (`textureLoad`, no float filtering), read by the bloom
   prefilter and the tonemap through `textureLoad`. With adapt speeds the
   **exposure-adapt** pass follows: the 1×1 `R32Float` adapted EV walks
   towards the metered one, and the bloom prefilter and the tonemap read it
   instead.
9. **Bloom**: a 13-tap prefilter with exposure and a soft-knee threshold
   into a half-resolution chain (up to 6 levels), 13-tap downsamples, then
   tent upsamples added back level by level.
10. **Tonemap**: exposure × (HDR + bloom × intensity) → ACES, AgX, Khronos
    PBR Neutral or clamp → the output, or the LDR chain when a stylise or
    antialiasing pass follows ([Stylisation](#stylisation)). An sRGB
    output is encoded by the hardware; a non-sRGB one (WebGPU canvases are
    `Bgra8Unorm`) by the shader (`fs_srgb_encode`). A surface takes an sRGB
    format where one is offered (native windows).
11. **Overlay** (when `scene.ui.overlay` is set; `overlay.rs`,
    `overlay.wgsl`): after the graph, straight into the output or its sRGB
    view ([The overlay](#the-overlay)).

## Forward+

Point and spot lights are clustered (`src/clusters.rs`). The view is cut
into `CLUSTER_GRID` = 16 × 9 screen tiles × 24 depth slices (logarithmic
between the camera's near and far planes; even for an orthographic
camera). Each frame the CPU culls the lights whose sphere (`range`)
misses the view frustum, keeps up to `MAX_LIGHTS` = 256 (past it the ones
with the most radiance at the camera: intensity times the colour's
brightest channel over the squared distance, at least 1 m), and assigns
each to the clusters its sphere can touch: the view-space box of the
sphere, clipped to the near plane and projected, widened by a pixel for
the TAA jitter, and the slices of its depth range. A cluster holds at
most `MAX_CLUSTER_LIGHTS` = 64; past it a cluster keeps its most
important lights. The lit shader finds its cluster from the pixel and
the view depth and shades that cluster's lights in scene order, so a
clustered frame is the frame of the same lights brute-forced over every
pixel.

The tables are three storage buffers in the frame group (bindings 7 to
9: the light records, one word per cluster with its first reference and
count, and the references as bytes). The build runs on the CPU: a
compute build costs more to record and submit than the CPU build takes
([What phase 2 owes](#what-phase-2-owes-decisions)). Stats: `lights`
(drawn), `lights_culled`, `lights_dropped`, `cluster_lights`
(references stored), `cluster_lights_dropped`.

The terrain's lit pass reads the same tables. The planar reflection pass
shades the main view's lights through clusters of its own, built for the
mirrored camera over the same light records (`ClusterBuilder::build_kept`;
the oblique projection moves only depth, so the tiles follow the mirrored
camera's x and y).

A reflection probe's face has no clusters (its uniform's `cluster` is
zero, `probes.rs`): every lit shader loops every light record of the
frame there (terrain, scattered copies, the 3D fluid's composite and full
material shaders through area.wgsl's `surface_local_lights`). *Limit:*
the records are the main camera's kept lights, so a light whose range
misses the main view lights nothing in a capture either.

## Shadows and antialiasing

`src/shadows.rs`, `src/shadow_cache.rs`, `src/frame.rs`,
`shaders/lighting.wgsl`, `shaders/shadow_pass.wgsl`, `src/post.rs`,
`shaders/fxaa.wgsl`, `shaders/taa.wgsl`. Everything below is off by
default unless stated.

**Shadowed local lights.** Up to `RenderInfo::shadowed_lights`
(`MAX_SHADOWED_LIGHTS` = 16, `og.render.capabilities().maxShadowedLights`)
point and spot lights cast shadows in a frame, `RenderQuality::
shadowed_lights` of them (8 by default). When more in view cast, the
ones with the most radiance at the camera keep theirs (the clusters'
score; ties in scene order) and the rest are counted in
`FrameStats::shadowed_lights_dropped`; `shadowed_lights` counts the drawn
ones. Each gets consecutive layers of one array, handed out in scene
order: a spot one view, a point light six cube faces or two paraboloid
hemispheres. The array starts at twelve layers and grows once, to its
capacity, the first frame whose scene could need more.

**Which faces a pass stores.** Every shadow pass culls a front-sided
caster's front faces. A local layer is `Rg32Float` and holds its casters'
far sides (red) and near sides (green), both drawn in one render pass per
face; the lit shaders compare against their midpoint (Woo 1992,
`shadow_target.rs`, `lighting.wgsl` `local_depth`), so a closed caster
never shadows its own lit side and a zero-thickness floor does not shadow
itself. A texel with no far side (a single-sided plane, a wall) casts
nothing. A local light's biases are a tenth of a texel of depth and a
quarter of the normal offset, per metre of distance (`frame.rs`
`local_lights`). The paraboloid hemispheres mirror x so they store the
same faces as cube faces. The cascades keep far sides alone.

**Which instances cast and receive.** `Instance::cast_shadows` keeps an
instance out of every shadow pass while it still receives (its instances
are runs of their own). `Instance::receive_shadows` (and
`SkinnedInstance::receive_shadows`) lights an instance as if nothing
stood between it and any light; the flag rides in the instance record's
layer lane (`instances::layer_bits`), so it never splits a batch.
og-runtime sets them from `og.Mesh.castShadow` (off by default, as
three.js), `og.Model.castShadow` (on by default, from the load's
`GltfDesc`), and `og.Mesh.receiveShadow` / `og.Model.receiveShadow` (on
by default).

**Static shadow caching** (`src/shadow_cache.rs`, after Unreal's virtual
shadow maps' static cache and HDRP's mixed cached shadows). Each cascade
and local face keeps what its static casters drew while its view stands
still. A caster is static once its drawn transform, sway, mesh and
material and their revisions have been unchanged for eight frames.
Whatever a vertex shader moves without its transform changing is
dynamic, either through the batch (`shadow_cache::deforms`: foliage
sway, material vertex hooks) or by drawing over every layer it reaches
each frame (skinned and morphed meshes, scattered copies, og.Grass,
terrain levels, the meshlet path's casters); a new vertex deformation
must join one of the two. A layer with nothing dynamic is not drawn; one
with dynamic casters copies its static casters back from its cache and
draws only the dynamic ones over them; a caster that appears, goes,
starts or stops moving redraws only the layers its old and new spheres
reach; a layer whose view changed draws everything and caches from the
first frame its view stays. `FrameStats::shadow_cached` counts the layers
served from the cache (`shadow_passes` still counts every layer).

**Contact shadows** (contact.wgsl `cs_contact`, `post/contact.rs`). The
three shadowed point and spot lights brightest at the camera get
screen-space contact shadows; rect lights never do. The pass is Bend
Studio's screen-space shadows (`third_party/bend-sss`, Apache-2.0) with
its recommended settings (60 samples along the line to the light, a
depth slab 0.5 % of the pixel's depth thick) as a compute wave of 64
threads per run of pixels; overlapping runs resolve by `atomicMax` on a
key, so the result does not depend on GPU order. The masks share the AO
texture (its gba), so they are off with screen-space GI, and only the
standard and physical lit shaders read them (not terrain, scattered
copies, custom lit shaders, probe faces, views or planar reflections).
They make the frame draw the prepass.

**Point shadows** (`RenderDesc::point_shadows`, the IDL's
`RenderOptions.pointShadows`): `Cube` draws six 90° faces, `Paraboloid` the
hemisphere below the light and the one above, each in one pass (the
projection scaled into 0.9 of the disc). A large caster's silhouette is
approximate (triangles stay straight between projected vertices); the
stored distance is exact for flat triangles of any size. The texels are
about 2.2 times coarser at a hemisphere's centre than a cube face's.
`Auto` picks cube faces; `Paraboloid` is the choice for a game with many
shadowed point lights (two passes instead of six each).

**The sun's soft shadows and cascade blending** (`DirectionalLight::
shadow_softness`, `cascade_blend`, the IDL's `shadowSoftness` and
`cascadeBlend`). PCSS: 16 Vogel-disc reads search the cascade for blockers
over the widest penumbra the cascade allows (24 texels at most), then 16
bilinear PCF taps (64 compares) filter over the penumbra their mean depth
gives, softness × the distance from blocker to receiver; a slope term
keeps the wide kernel off receivers tilted from the light. Blending: each
cascade's last `cascade_blend` of its depth fades into the next, whose fit
reaches back over that band so its casters are there. Both read with
`textureLoad` compares.

**FXAA** (`Antialias::Fxaa`): FXAA 3.11's quality path (eight edge-search
steps) over the tonemapped image, which the tonemap writes into an sRGB
8-bit target the FXAA pass reads and writes to the output before the
overlay, so the HUD stays sharp. One pass, no history.

**4× MSAA** (`Antialias::Msaa4`): the HDR target and the depth buffer
multisampled, resolved before the post chain. No jitter and no history,
so sprites, text and pixel art stay sharp and a moving camera never
blurs; it does not smooth shading or texture aliasing as TAA does.

**SMAA 1x** (`Antialias::Smaa`, TypeScript `antialias: "smaa"`): the
reference high preset (Jimenez et al., 2012), with perceptual luma edges,
diagonal/corner searches and area/search lookup textures. Three passes
after tonemapping and stylisation, before pixel-grid upscale and the
overlay. Edges use sRGB-encoded luma; neighbourhood blending uses linear
colour. No jitter or history; it replaces TAA rather than stacking with
it. The edge/weight/blend layouts use 2/5/3 bindings (weights read the
shared temporal uniform), with no extra lit bindings. Licences, provenance
and texture dimensions: `third_party/smaa/`.

**SMAA T2x** (`Antialias::SmaaT2x`, TypeScript `antialias: "smaaT2x"`)
uses the reference's alternating quarter-pixel diagonal jitter and area
layers 1/2. It reuses TAA's velocity, reprojection, cut/reset validity
and neighbourhood clamp, but keeps two raw SMAA frames in its own sRGB
history pair and takes their linear mean (the first frame after a cut,
resize, scene or stylisation switch uses only the current one). It runs
after tonemapping, before stylisation, lens distortion, upscale and UI,
so the overlay never enters history. It is exclusive of TAA, TAAU and
MSAA; render scale uses spatial upscaling, and `taaHistory`/`taaSharpen`
apply only to TAA. The mode is chosen when the renderer is created. S2x
is not implemented.

**Bloom follows the antialiasing.** Bloom's prefilter reads the lit image
after antialiasing, so the method decides how much of a thin emitter's
energy reaches it: the MSAA resolve averages the HDR samples linearly and
keeps a sub-pixel emissive line's energy; TAA's resolve weights each
colour by 1 / (1 + luma) and clamps the history, so a thin line settles
darker and its halo is weaker. A look with bloom over thin emissive lines
therefore differs between `msaa4` and `taa` by design, on every backend.

**TAA's history and sharpening** (`RenderDesc::taa_history`,
`taa_sharpen`; the IDL's `taaHistory` and `taaSharpen`):
`TaaHistory::CatmullRom` resamples the history with a 9-tap Catmull-Rom
(Jimenez's bilinear-tap form) before the neighbourhood clamp, which keeps
it sharp when the camera moves; `taa_sharpen` adds a pass after TAA into
its own target (the history stays unsharpened): an unsharp mask over the
four neighbours clamped to their range.

**Where TAA reads its history.** The pixel's centre in uv is
`in.pos.xy / size` (the window position, exact on every backend), not the
interpolated `in.uv`. A still camera is decided on the CPU
(`still_camera`: the history's view-projection and the current one equal
to a millionth of each entry) and sent in the uniform
(`TaaUniform::still`); then, and wherever a reprojection moves less than
a thousandth of a pixel, a pixel reads its own history texel with
`textureLoad`, unfiltered, so no sampler or varying precision can move the
history of a still camera. The resolve reads the depth through a nearest
sampler of its own (`depth_sampler`, binding 5) at the pixel's centre.

**Ambient occlusion.** `AoMethod::Auto` (the default, and what
`PostEffects.ao` asks for) is GTAO on every backend; `Ssao` (a cheaper
hemisphere of depth samples, softer and noisier) and the other methods
stay explicit choices.

## What a frame may cost: `RenderQuality`

`RenderScene::quality` (`src/quality.rs`) is read every frame on top of
what the scene asks for; the API's RenderSettings carries its fields one
to one and `og.render.applyQuality` writes a `QualityPreset`'s row of
them, so the probe reads what is drawn. Each switch only takes away, each
scale only scales what the scene asks for. The default is medium's row.

| Field | low | medium | high | Changing it |
|---|---|---|---|---|
| `rect_shadows` | false | true | true | free |
| `shadowed_lights` | 2 | 8 | 16 | free; `RenderInfo::shadowed_lights` caps it |
| `shadow_cascades` | 2 | 3 | 4 | remakes the cascade array; `RenderInfo::shadow_cascades` caps it |
| `shadow_map_scale` | 0.5 | 1 | 2 | remakes the shadow maps |
| `volumetric_fog` | false | true | true | free |
| `soft_particles` | false | true | true | free |
| `ssr` | false | true | true | free |
| `ambient_occlusion` | false | true | true | free |
| `global_illumination` | false | true | true | free |
| `ao_resolution` | `Some(Half)` | `None` | `Some(Full)` | free (the graph adds or drops `ao-upsample`) |
| `ssr_step_scale` | 0.5 | 1 | 2 | free (a uniform) |
| `volumetric_step_scale` | 0.5 | 1 | 2 | remakes the froxel volumes (the raymarch's steps are a uniform) |
| `msaa_samples` | 1 | 4 | 4 | remakes the multisampled targets; `RenderInfo::msaa_samples` caps it |
| `upscaler` | `Taau` | `Taau` | `Fsr3` | `docs/design.md` §11 |

`ao_resolution` replaces `AmbientOcclusion::resolution` where it is
`Some`; `None` draws the scene's. `ssr_step_scale` multiplies
`ScreenSpaceReflections::steps` (within 4 to 128) and
`volumetric_step_scale` the froxel grid's 64 slices (8 to 256) or the
raymarch's 24 steps (4 to 96). `shadow_map_scale` keeps each map within
64 texels and the device's limit (and 4096 for the cascades).
`RenderQuality::capped(preset)` is a quality no costlier than a preset's
row (automatic quality's steps cap the game's own quality, never raise
it).

The device caps two fields, and the fields keep what was asked: at most
`RenderInfo::shadowed_lights` lights cast shadows, and
`RenderInfo::msaa_samples` (4 where `Antialias` resolved to `Msaa4` when
the renderer was made, else 1) caps `msaa_samples`, so under TAA, FXAA
or none a quality of 4 draws one sample; `RenderInfo::antialias` says
what is drawn now. Not run-time knobs, so not in a preset: the PCSS taps
and the PCF kernel (WGSL constants), GTAO's slices and steps (constants)
and the bloom chain's depth (set by the target's size).

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
values.

Positions are CSS pixels from the top-left corner, scaled by
`Overlay::scale` (0: the output's height over `Overlay::height`). Sprites
and text are sorted by (layer, texture or font) into instanced runs
(stable within a run); each instance carries an optional clip rectangle
tested per pixel, so a scrolled list cuts a glyph in half instead of
dropping it.

Overlay text also takes `weight` (the fill's edge moved out by that many
CSS pixels of the distance field: a faux bold) and `glow`
(`TextGlow { color, radius }`): a soft halo read from the font's distance
field in the same MSDF pipeline, drawn as one extra instance per glyph,
every glyph's glow before every fill so no halo lies over ink. The
distance is the MTSDF's true field in alpha (round past corners), or the
median for a plain MSDF; it falls off quadratically to `radius`, capped
where the field ends (half the font's `distance_range` in atlas pixels at
the size drawn, so a font cooked with `px_range 12` glows three times as
far as one with 4). The weight stays within the field's reach too. og-ui's
skins use it for `textGlow`.

*The UI's layer* (`UI_LAYER` 1000, `UI_OVERLAY_LAYER` 2^20,
`screen_layer`): og-runtime emits og.ui's painter levels from
`UI_OVERLAY_LAYER` up and a game's screen sprites and screen text at
`screen_layer(layer)`: their own layer up to `UI_LAYER`, under every UI
element, and lifted by 2^21 above it, over every one (a HUD icon of
sprites on a UI panel), the order among screen layers kept. og-ui's
`tests/screen_layers.rs` and `references/<tier>/screen-layers.png` hold it.

`Overlay::sprite_arrays` draws overlay sprites from a layer of a texture
array, one draw per array ([The 2D batch](#the-2d-batch)).

`Overlay::glass` holds glass panels (`Glass`: a rounded rectangle, a blur
radius, a tint and its share, an opacity, a layer and a clip): a frame
with any renders its tonemap into a texture of its own (`src/glass.rs`),
copies it into the output exactly, and downsamples it into a six-level
`Rgba16Float` pyramid of 13-tap blurs (decoding an sRGB-encoded source);
each panel samples the level its blur asks for, trilinear between levels,
tints it and masks it to its rounded rectangle, sorted with the overlay's
other runs at its layer. It blurs the scene under the overlay, not other
overlay content; a frame without glass pays nothing.

Every tier runs the overlay (render passes only); screenshots, `capture`
and `read_offscreen` include it; `FrameStats` counts `overlay_sprites`,
`overlay_glyphs` and `overlay_draws`. Tests: `tests/gfx.rs`
`the_overlay_is_drawn_after_post_untonemapped_and_clipped`,
`the_overlay_blends_in_linear_light_on_a_non_srgb_output`, the `overlay`
reference scene (the showcase under TAA with a HUD); og-ui's `tests/gfx.rs`
`a_text_glow_is_a_halo_from_the_distance_field` and
`a_glass_panel_blurs_the_frame_under_it_and_nothing_else`.

## From TypeScript

Every effect is declared from a game through the IDL
(`crates/og-api/api/render.ogt`, module `render`), on both hosts, off by
default:

| og-render | TypeScript |
|---|---|
| AO, outlines, depth of field, motion blur, screen blur, god rays, glow, LUT grading, colour adjustments, vignette, aberration, lens distortion, grain, flash, shake | resource `og.PostEffects` (zero = off; `lut` is a handle from `og.load.lut`) |
| `Lighting2d`, `Light2d`, `Occluder2d` | resource `og.Lighting2D`; components `og.Light2D`, `og.Occluder2D`; `SpriteDesc.castShadow` |
| `Decal` | component `og.Decal` with the atlas in `og.RenderSettings.decalAtlas` |
| `ParticleSystem` | component `og.ParticleSystem` naming an emitter from `og.load.emitter` (`.ogt` of type `particles`, cooked by og-cook) |
| `Trail`, `Beam`, `Ribbon` | components `og.Trail`, `og.Beam`; `og.render.ribbon(world, desc)` per tick |
| `SpriteFx` | `SpriteDesc` `flash`, `flashColor`, `glow`, `blend`, `toneMapped` |
| Toon, rim, dissolve, flash, outline per material | `StandardMaterialDesc` fields, set with `og.render.standardMaterial` / `updateMaterial` |
| Skinning and blendshapes | `og.Skeleton` + `og.AnimationPlayer`, `og.Blendshapes` (the runtime keeps skinned copies of a model's meshes); `og.Garment`'s simulated vertices through `SkinnedInstance::deformed` and `RenderScene::deformed_vertices`; `og.CrowdAnimation`'s baked clips through `RenderScene::baked_clips` and `SkinnedInstance::baked`, blended on the GPU (`skin.rs`; frustum-culled on the CPU like every skinned instance, not GPU-culled or clustered) |
| `Overlay` | `SpriteDesc.screen` / `TextDesc.screen` with an optional `clip` `[x, y, w, h]`; og-ui draws into it |

**Capture** (`capture.rs`): the offscreen target copied to a buffer and
mapped. One async function on both hosts: natively it polls the device; on
the web the mapping completes through WebGPU's `mapAsync`. It reads only
the offscreen target, so a page needs no `preserveDrawingBuffer`; a probe
capture renders a frame to `Offscreen` first. A capture whose frame was
read leaves its buffer to the next capture of the same size.
`png(&image)` is the probe's encoder (`capture`, `sweep`, `screenshot`):
rows filtered `Up` and deflated by fdeflate as they are filtered, the
pixels decoding exactly.

## Backend tiers

Every backend is one tier: compute, storage buffers and indirect draws
are assumed (`docs/design.md` §2.6, §8). On the web the engine is WebGPU
only.

| Backends | Reference directory (`tier_name`) | Antialiasing (`Tier`) | Point shadows (`Auto`) | AO (`Auto`) | Particle collision |
|---|---|---|---|---|---|
| Metal, Vulkan (GPU or lavapipe), DX12, WebGPU | `metal`, `vulkan`, `vulkan-cpu`, `dx12`, `webgpu` | TAA | cube faces | GTAO | the depth buffer (compute) |

The device is asked for `Limits::downlevel_defaults()` at the adapter's
resolution. Shadow maps are `R32Float` (cascades) and `Rg32Float` (local
layers) colour arrays read with `textureLoad` and compared explicitly,
the HDR target is `Rgba16Float`, and IBL, mips, metering and bloom are
render passes. A lit pipeline has 16 sampled textures per stage (the
downlevel limits): the lit skinned pipeline uses frame 5, material 8,
fx 2, so the sky cube is the sky pass's own group. The physical pipelines
pack their maps into one array texture and use 14
([Physical materials](#physical-materials)). The remaining choices
(`Antialias::Msaa4`, `Fxaa`, `Smaa`, `SmaaT2x`, `PointShadows::Paraboloid`,
`AoMethod::Ssao`, the volumetric raymarch, `Culling::Cpu`,
`ParticleSimulation::Cpu`) are a game's to make for its look or its
budget, on every backend.

## Shared frames

`shared_frames` (macOS, Linux, Windows) shows the renderer's frames in
another process's window, Origin Desktop's Preview (`docs/design.md` §2.6,
Shared frames). `FrameLink::connect(path, &renderer)` connects on a thread
of its own to the socket a host names in `SOCKET_ENV`
(`ORIGIN_FRAMES_SOCKET`), and again whenever the host comes back; it never
blocks or fails. Each loop frame, after `render(&scene,
&RenderTarget::Offscreen)`, `present(&renderer)` copies the offscreen
target into a free slot on the GPU (`Presented::Slot`; `Busy` when the host
holds every slot, `NoHost` before its slots exist) and sends `Msg::Ready`
when the copy has finished; `take_resize()` is the size the host asked for,
which the caller applies with `resize` before its next frame (the link then
sends `Rehello` and the next generation's `Hello`). The renderer draws in
`OUTPUT_FORMAT` (`Bgra8UnormSrgb` on macOS, `Rgba8UnormSrgb` elsewhere),
`SLOT_FORMAT` seen as sRGB, so the copy is byte for byte; captures read it
as RGBA either way. The wire (`Msg`, each platform's `Hello`, `send`,
`recv`) is Origin Desktop's `shared-frame` contract; the host's side
(`recv_hello`, `import_slot`) is here for the tests
(`tests/shared_frames.rs`), which play the host on a second device. The
native host (`og-host-node`) links when the variable is set.

## Testing

`crates/og-tests` is the gfx reference harness. Each named scene renders
headless at 1280×720 (after its warm-up frames, with its per-frame update)
and is compared with `crates/og-tests/references/<backend-tier>/<scene>.png`
(`og_tests::compare::Tolerance`): a pixel is an outlier when a channel
differs by more than 32/255; the frame fails above 0.5 % outliers or an
RGB RMSE above 0.01. On failure the actual frame and a diff go to
`target/…/tmp/og-gfx/<tier>/`. A missing reference fails. Metal is the
reference platform; `vulkan-cpu` references are not recorded. The same
scenes compile for wasm32, where `crates/og-tests-web` draws them on
`webgpu` (`make web-reference`), judged against `metal`.

Scenes named here (the effect, atmosphere, terrain, water, stylisation
and sprite scenes are listed in their sections):

| Scene | What it shows |
|---|---|
| `gltf-reference` | Trickster + dropship × 25 with CSM, sky, mips. |
| `spheres-20k` | 20,000 spheres submitted one by one: 2 batches, ≤ 10 draws. |
| `cascades-200m` | A 420 m field, pillars to 190 m, a wall at 200 m, short kerbs and blocks: shadows in every cascade. |
| `point-spot-lights` | Night, 7 local lights, a shadowed spot and a shadowed point light (7 shadow passes). |
| `point-shadows-paraboloid` | The same with the point light's shadow in two paraboloid hemispheres: 3 passes. |
| `shadows-local-8` | Night plaza: eight shadowed lights (six point, two spot; 38 passes) and two dim ones far behind whose shadows are left out. |
| `shadows-local-contact` | Local shadows with contact shadows at casters' feet. |
| `shadows-crypt` | A roofed room under a low sun with a shadowed point light inside a glass box that casts no shadow (`Instance::cast_shadows`): the light reaches the altar and the pillar, a post's shadow falls beside them. |
| `shadow-flags` | A low sun over a field: a trickster casting beside one with `cast_shadows` off, and in a wall's shadow a box and a trickster that receive beside ones with `receive_shadows` off. |
| `vox-keyframes` | A `.vox` cooked by og-cook (native only, `NATIVE_SCENES`) under a transform keyed at frames 0 and 2 (MagicaVoxel `nTRN` frames), sampled with og-anim and skinned as og-runtime poses a model. |
| `shadows-soft` | A low sun with PCSS (softness 0.03) and cascade blending (0.2): a cube's contact shadow sharp, a raised plank's and a floating ball's penumbrae wide, poles across the cascade split. |
| `taa` | Thin poles and bars after 16 TAA frames. |
| `fxaa` | The same geometry with FXAA. |
| `taa-sharp` | The same under TAA with the camera sliding 2 cm a frame, Catmull-Rom history and 0.5 sharpening. |
| `smaa`, `smaa-t2x`, `smaa-t2x-moving` | The thin geometry under SMAA 1x; SMAA T2x still, and under half-scale camera and object motion. |
| `bloom` | Emissive spheres with bloom. |
| `exposure` | A dim dusk scene metered to middle grey. |
| `post-agx`, `post-neutral` | The `bloom` scene through AgX and through Khronos PBR Neutral. |
| `post-blur` | The `bloom` scene through a zoom blur (0.15, 16 taps) towards a point off centre. |
| `post-lens` | The spike's scene through a centred barrel lens (0.35), the corners fitted. |
| `post-adjust` | The spike's scene through a LUT at 0.6 with colour adjustments baked into the same table. |
| `environment-map` | Chrome and white spheres from mirror to rough lit only by a cooked 64×32 HDR studio, turned 0.6 rad, drawn as the background. |
| `text-msdf` | Latin and CJK MSDF text from 0.32 to 4.5 world units, rotated. |
| `sky-only` | The sky with the sun in view: 2 draws. |
| `showcase` | Models, CSM, a shadowed spot and point light, bloom, auto exposure, world-space text and sprites, the default antialiasing (`Tier`: TAA). |
| `overlay` | The showcase under TAA with a HUD. |
| `lights-256` | Night over a plaza lit by 256 point and spot lights (Forward+; 243 in view). |
| `materials` | Metalness and roughness rows, emissive, normal maps, clearcoat, alpha mask and blend, double-sided, unlit, a non-uniformly scaled ellipsoid. |
| `materials-array` | Texture-array maps: eight cubes of one tile material on their own layers in one batch, a cube of plain 2D textures beside them, blended panes of one array material, a sphere with only a normal array. |
| `materials-physical` | Every `MeshPhysicalMaterial` lobe: clearcoat with its maps, sheen, iridescence, anisotropy, clear, frosted and amber glass, a transmission-mapped cube and a high-`ior` ball. |
| `model-flash` | Three instances of one glTF model, one flashing white and one tinted, and two skinned cylinders of one material, one flashing red. |
| `skinned-hooks` | Skinned cylinders drawn with a material shader that ripples, bands and lifts them; their shadows and AO follow. |

Behaviour tests in `tests/gfx.rs` read frames rather than code: the cascade
debug view puts cascade 0 at the camera's feet and cascade 2 at the horizon;
short casters (0.5 m blocks, 0.6 m kerbs) shadow pixels in every cascade of
the 400 m scene; local shadows darken the ground; MSAA and TAA turn
black/white edges into partial coverage; bloom brightens the surroundings
of emitters; metered exposure gives the same brightness for a scene lit 16×
brighter (manual does not); higher sprite layers draw on top; missing
glyphs and foreign font handles are reported; a BGRA non-sRGB output
captures the same picture as RGBA sRGB; lights past the shadow budget are
counted. Each was checked by breaking the code it covers.

```sh
cargo test --release -p og-tests -p og-tests-cooked                 # gfx + the wasm32 compile check
OG_GFX_CAPTURE=1 cargo test --release -p og-tests -p og-tests-cooked --test tests gfx::      # re-record this tier
OG_GFX_CAPTURE=spheres-20k cargo test --release -p og-tests -p og-tests-cooked --test tests gfx::  # one scene
cargo test --release -p og-tests --test tests -- --ignored perf::          # perf vs crates/og-tests/perf/
OG_PERF_RECORD=1 cargo test --release -p og-tests --test tests -- --ignored perf:: --test-threads=1
cargo run --release -p og-tests-cooked --bin og-gfx -- --all --frames 120        # stats, timings, RMSE
cargo run --release -p og-tests-cooked --bin og-gfx -- --costs                   # the feature-cost table (--costs-p2, --costs-e5)
OG_GFX_DEBUG_CASCADES=1 cargo run --release -p og-tests-cooked --bin og-gfx -- cascades-200m --out c.png
```

`og-gfx` also takes `OG_GFX_BACKEND`, `OG_GFX_CULLING`, `OG_GFX_COMPACT`
(every mesh compact) and `OG_GFX_QUALITY`. Perf baselines are in
`crates/og-tests/perf/` per tier and adapter, compared with 15 %
tolerance.

**The MSDF test atlas** (`crates/og-tests/assets/fonts/noto-sans-sc-msdf.*`)
is Noto Sans SC (an OFL subset; `OFL.txt` alongside), ASCII plus
原点引擎文字渲染, generated with
[msdf-atlas-gen](https://github.com/Chlumsky/msdf-atlas-gen) (commit
6148900):
`msdf-atlas-gen -font NotoSansSC-Regular-subset.ttf -charset noto-sans-sc-msdf.charset -type msdf -format png -size 32 -pxrange 4 -dimensions 512 256 -yorigin top -imageout noto-sans-sc-msdf.png -csv noto-sans-sc-msdf.csv -json noto-sans-sc-msdf.json`.
`og_tests::scenes::load_font` reads the PNG, the CSV (glyphs) and two
numbers of the JSON (`distanceRange`, `lineHeight`). The cook owns
producing atlases; the renderer only takes a texture and a glyph table.

**HDR output** (`RenderDesc::hdr`, `paper_white_nits`, `peak_nits`) is
opt-in and not yet accepted on a real HDR display; with HDR off the SDR
frames are unchanged. Neither `og run`'s shared-frame preview nor the
headless harness configures an HDR swapchain, so acceptance needs a real
surface: the opt-in `og-tests` surface example opens a native window,
calls `Renderer::with_surface` and presents each frame through
`RenderTarget::Surface`:

```sh
cargo build -p og-tests --example surface --features surface
OG_GFX_BACKEND=metal target/debug/examples/surface bloom hdr 203 1000 120 /tmp/hdr-sdr-preview.png
OG_GFX_BACKEND=metal target/debug/examples/surface bloom sdr 203 1000 120 /tmp/sdr-preview.png
OG_GFX_BACKEND=metal target/debug/examples/surface ui-glass-bright hdr
OG_GFX_BACKEND=metal target/debug/examples/surface views-split hdr
```

Arguments are scene (the reference harness's), `hdr|sdr`, paper-white
nits, peak nits, frame count (0: until the window closes) and an optional
preview PNG (a clipped SDR preview, never HDR evidence). An HDR request
resolving to SDR fails; there is no headless fallback, and the example is
outside the default gate. `RenderError::SurfaceOccluded` means no drawable
was acquired and no frame presented: the driver waits for the window to
become visible, and closing before any present reports that acceptance
did not run. `hdr_resolved=true` proves surface configuration, not
physical luminance.

## Performance

### Pipelines made on first use

Every pipeline the renderer does not need for its first frame is a
`Deferred` (`src/deferred.rs`), made from the same descriptor when a pass
first sets it, and the shader modules only those passes use are parsed
then too (`deferred::module`), so a frame compiles only what it draws and
every frame is the frame it would be with every pipeline made up front.
`Renderer::warm()` makes the rest one per call, the likeliest first, and
answers false when none is left: the web Worker calls it in idle time
behind the play gate (`og_web_warm`), so a pass met later does not stop a
frame to compile. Compiled pipelines are kept across runs where wgpu has
a pipeline cache (Vulkan; `set_pipeline_cache_dir`,
`RenderInfo::pipeline_cache`); Metal and the browsers keep their own.

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
shader (a styled material is judged as styled). Its unskinned instances
draw with `vs_lean` / `fs_lean` (position, normal and colour in; 10
interpolated floats) and in the prepass with `vs_prepass_lean` /
`fs_prepass_lean` (14 floats). Positions come from the same `lit_place`
and the fragments run the full path's shading, so the pixels are the full
pipeline's byte for byte. The `DISCARD` override drops the fragment
discard, which keeps the early depth test, except where the fragments
take screen derivatives (flat shading, or decals in the frame;
`Batch::derivatives`): on Metal a shader with a discard computes
derivatives differently. The lean pipelines are keyed (side, effects,
derivatives), the lean prepass (side, derivatives), made at first use.
Materials with texture-array maps draw through a variant of the full lit,
prepass and shadow pipelines ([Array
maps](#array-maps-materials-and-terrain-layers)), never the lean ones.
Shadow casters that are not skinned, dissolving or hooked read their
position alone whatever their material (`shadow.wgsl` `vs_lean` /
`fs_lean`: the world position the only varying; no extra pipeline).

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
`physical.wgsl`, each behind a branch, so an instance without a look
shades as without the feature. One (mesh, material) is one draw whatever
its instances show. The runtime fills it from `og.Model`'s `tint`,
`flash` and `flashColor` (`og.tween.flash` takes an og.Model entity) and
from og.Mesh's `flash` and `flashColor`. Scene `model-flash`.

### Instance and vertex encodings

**Instances: 64 bytes.** Every instance buffer (the CPU path's, GPU
culling's regions, the previous transforms of object motion, the
scattered copies) holds an instance's transform as the x, y and z of its
four columns and, after them, an array-map instance's layer with whether
it receives shadows (`instances::layer_bits`) and the instance's look
(`instances::GpuTransform`: `Float32x3` attributes 4 to 7 and a
`Float32x4` at 8, pbr.wgsl's `InstanceFxIn`; 16 floats a slot in
`cull.wgsl`); a skinned instance holds the columns before its palette and
its look after its morph weights (`skin::SkinnedInstanceGpu`, the look at
attribute 13). `instance.wgsl`'s `instance_matrix` rebuilds the 4x4 with
the fourth row (0, 0, 0, 1), so an `Instance::transform` is affine. The
projections of the points such a matrix moves go through `mul_point`, a
column-order fused chain, so Metal does not reorder the sum and move
vertices by an ulp; a point moved by a material's `vertex_offset` keeps
the plain product (`hooks.rs` `moved_clip`).

**Vertices: 16 bytes, opt-in.** `VertexEncoding::Compact`
(`RenderScene::set_vertex_encoding`; `compact.rs`) uploads a mesh as the
position in `Unorm16x4` over its bounds, the normal octahedral in
`Snorm16x2` and the colour in `Unorm8x4`; no uv and no tangent, which no
plain material reads. A run that draws the compact vertices folds the
bounds into its instances' GPU transforms (`Run::dequant`). The lean lit,
lean prepass, lean shadow caster and occluder-depth pipelines have
compact variants. The f32 vertices of a compact mesh are uploaded only
when something else draws it (a material that is not plain, a scatter, a
skinned instance: `Renderer::ensure_full`). Skinned and morphed meshes
ignore it. It is quantised (the position to 1/65535 of the bounds, the
normal to about 1e-4 rad, the colour to 8 bits), so its pixels are near
`Full`'s, not equal: the default stays `Full`.

## The 2D batch

Everything world-space 2D goes through one sorted list per frame: sprites,
2D particles and glyphs keyed by (layer, sort key, source), then tile
layers and layered trails (`trail_layers: (Trail::id, layer)`) placed
before the sprites of their layer (tiles first), adjacent runs of one
source merged into one instanced draw. A layered trail is drawn with the
ribbon pipeline inside the sprite order (unlayered trails, ribbons and
beams draw with the 3D scene), so a 2D sprite's trail shows over a tile
background. Glyph instances carry an outline in their otherwise unused
flash colour and glow strength; `fs_msdf` grows the shape by the width
and mixes the fill over the outline. A sprite or glyph instance is 104
bytes.

**Texture arrays** (`sprite_arrays`, `SpriteArray { sprite, array, layer
}`): a world sprite draws from a layer of a texture array instead of its
texture, through pipeline variants (`shaders/sprite_array.wgsl`, made on
first use) for the alpha, additive and glow blends. The run key is (blend,
array): every sprite of one array is one instanced draw whichever layer
it shows. A layer past the array's count is refused (`InvalidData`).
Array sprites take the hit flash, 2D lighting, fog, the soft dot and the
glow pass. Screen sprites draw from arrays too (`Overlay::sprite_arrays`,
indexing `Overlay::sprites`; `shaders/overlay_array.wgsl`), one draw per
array. 2D particles draw from an array when their emitter names one
(`ParticleEmitter::array`, sprite mode only: a billboard emitter with an
array is refused), the layers picked by the sheet's mode (`OverLife` and
`Fps` by age, `Random` per particle; `sheet.frames` limits them, 0 every
layer); `ParticleEmitter::round` gives sprite particles the soft dot's
round edge. MSDF text never samples an array. og-runtime's `og.Sprite { array,
arrayLayer }`, `SpriteDesc.array` (world or screen), an emitter's `array`
line and a flipbook sheet with `array true` (its frames cooked into an
array, frame n drawn as layer n) all come here. Scenes `sprites-20k`
(20,000 sprites of 3 textures, one a 2×2 atlas, 2 layers: 3 sprite
draws), `sprites-array` (a pixel-art flipbook array and a smooth array
over a plain strip: three draws), `sprites-array-fx` (glowing array
sprites, array particles at random and by age, overlay array sprites: 5
sprite draws).

**Drop shadows** (`sprite_shadows`, `SpriteShadow { sprite, opacity,
ground }`; the IDL's `SpriteDesc.dropShadow` and `Sprite.dropShadow`): a
world sprite's shadow is one more instance of its quad, region and
texture (or array layer) in black at `opacity` times its alpha, keyed
`f32::MIN` so it draws before every sprite of its layer; the vertex shader
carries each corner of the standing quad along the scene's sun
(`RenderScene::light`) down to its ground, lifted `SPRITE_SHADOW_LIFT`
(1 cm). Depth-tested as the sprites are. None with the sun at or below
the horizon; a low sun is raised to about 11° so a shadow is at most five
times the sprite's height (`shadow_ground`). Shadows are not counted in
`FrameStats::sprites`.

- *The ground*: `ShadowGround { height, normal }` is the plane through
  `height` under the sprite's anchor, tilted to `normal` (y = 0 by
  default; a slope steeper than about 70° is taken as that steep).
  `ShadowGround::fit(height_at, sprite, sun)` fits the plane to uneven
  ground so the shadow never sinks into a hill. og-runtime gives every
  shadow the level ground of `dropShadowHeight` (0 by default) and, where
  an og.Terrain under the anchor is higher, the plane fitted to it
  (`terrain::ground_drop_shadows`).
- *Bent*: where the plane misses the ground under a 5 × 3 grid of the quad
  (`SHADOW_BEND_ROWS` × `SHADOW_BEND_COLUMNS`) by more than
  `SHADOW_BEND_TOLERANCE` (3 cm plus 3 % of the sprite's height), `fit`
  bends it (`ShadowGround::bend`, `ShadowBend`): the shadow is drawn as
  4 × 2 seamless cells over the ground's heights. *Limit*: a feature
  narrower than a cell is bridged, and where the ground falls away steeper
  than the sun the cells stretch across it.
- *In the cascades*: the shadow's alpha is multiplied by the sun's
  cascaded light at its point, so where a wall or tree already shadows the
  ground the drop shadow fades out instead of darkening it twice.

Scenes `sprites-shadow` and `sprites-shadow-terrain` (figures on hills,
on a deck, in a wall's shadow, over a hollow); og-runtime
`tests/drop_shadows.rs`.

Tests: `tests/sprites2d.rs` (sort keys across textures within a layer,
layers still win; tile culling, tiles under their layer's sprites, repeat
filling the view's edge, an animated quad on its second frame; nearest
against linear sampling; nine-slice borders at their texel size; an
outline only when asked; a trail hidden under a layer-0 sprite without a
layer and shown over it on layer 1).

### Billboards

`RenderScene::sprite_facing` (`SpriteFacing`, by sprite index; past the
end, `None`; the IDL's `og.Sprite.facing` and `SpriteDesc.facing`, enum
`SpriteFacing` `none | camera | cameraY`): a world sprite turned towards
the camera that draws it, three.js `Sprite`. It is a mode of the sprite:
everything else about the sprite (texture, region, flash, glow, 2D
lighting, blend, fog, drop shadow, runs) is unchanged; only where its
corners stand changes (`stand` in sprite.wgsl and sprite_array.wgsl, from
the frame uniform's `view_right` and `view_up`). Every pass that draws
sprites (the main camera, the planar reflection, every `View`, every probe
face) turns them to its own eye on the GPU; nothing on the CPU reads the
turn. `rotation` still turns the quad within its own plane, about its
anchor.

- *Sorting.* A tone-mapped faced sprite without a `sprite_order` key sorts
  back to front with the transparent meshes by its anchor's view depth
  (`SpriteFrame::billboards`, `DrawList::before`), its drop shadow drawn
  before them; ties go by layer, then submission. With a key, or
  untonemapped, it keeps its place in the sprite order. Particles and
  unfaced sprites draw after the transparent meshes, in their layers'
  order.
- *Culling.* The main camera and each view drop a faced sprite (and its
  glow, not its drop shadow) whose bounding sphere about the anchor misses
  their frustum (`FrameStats::sprites_culled`); probe faces cull none.
  Unfaced sprites are not culled.
- *Drop shadows.* A faced sprite's shadow is cast by the quad stood
  upright facing the sun (`FACING_SUN`, `ShadowGround::fit_facing`), so it
  is the same in every view. The id pass turns faced sprites to its camera
  (`face_axes`). og-runtime anchors a faced nine-slice's pieces at the
  panel's anchor (`Sprite::anchored_at`), so the panel turns as one.
- *World text* (`RenderScene::text_facing`, `TextFacing { text, facing,
  align }`; the IDL's `TextDesc.facing` and `FloatingTextDesc.facing`):
  every glyph of a faced text is anchored at its pivot, each line shifted
  by `align` of its width, so the whole text turns as one panel. It sorts
  as a billboard by its pivot's depth and is not culled.

Tests: `sprites.rs`'s
`faced_sprites_sort_back_to_front_cull_to_the_frustum_and_cast_sun_facing_shadows`
and `a_sprite_anchored_elsewhere_keeps_its_corners`, og-runtime
`tests/billboards.rs` and `tests/text_facing.rs`, og-tests
`sprites-billboard`, gfx `billboards_sort_among_transparent_meshes` and
`world_text_faces_the_camera`.

### Untonemapped sprites

`SpriteFx::tone_mapped` false (the IDL's `SpriteDesc.toneMapped` and
`Sprite.toneMapped`, three.js `Material.toneMapped`): an alpha-blended
world sprite that keeps its colours as given while the scene around it
rides exposure, bloom and the tone curve (a speech bubble at dusk under
auto exposure). It is drawn in place, flagged in the HDR alpha, so it
stays depth-tested against the scene and sorted with the other sprites.
A cover pass sets the HDR target's alpha to 1 before the frame's first
such run; the sprite (`UNTONEMAPPED_BLEND`, `fs_untonemapped`: no fog)
takes the alpha towards 0 by its coverage and a tone-mapped sprite over
it takes it back. The alpha is then each pixel's tone-mapped fraction,
which MSAA, TAA and the HDR effects carry, bloom weights its taps by, and
the tonemap uses to mix the clamped, unexposed colour with the curve
(post.wgsl `tonemapped`). Grading, vignette, grain, the screen flash and
the stylise chain after the tonemap still apply. An additive sprite adds light to the scene and is
always tone mapped. Scene `sprites-untonemapped`.

## Effects

Everything a game declares on `RenderScene` as data (`src/effects.rs`,
`particles.rs`, `ribbons.rs`, `light2d.rs`, `skin.rs`); a field left at
its default records no pass and costs nothing
(`effects_off_add_no_passes`).

- **Post stack**: ambient occlusion (`AoMethod::Gtao` / `Ssao`, applied
  to the ambient term; `AoResolution::Half` searches and blurs every
  second pixel and adds a depth-aware `ao-upsample` pass), outlines (depth
  and normal edges, per-scene or per-material colour and width;
  [Stylisation](#stylisation)), depth of field (`DofQuality::Low` /
  `High`), motion blur from per-object and camera motion vectors (`fx.rs`
  carries the previous view-projection), god rays, colour grading by
  `ColorLut`, colour adjustments after it (`ColorAdjust`: white balance,
  brightness and contrast, colour balance, hue, saturation, vibrance,
  greyscale, invert and threshold, RedGPU's adjustments; baked with the
  grading table into one 32³ table on the CPU when either changes, so they
  add no pass), a zoom, spin or directional blur of the whole frame
  (`ScreenBlur`, RedGPU's, the spin and the directional reach in pixels:
  `fs_screen_blur`, one HDR pass after motion blur into `hdr.blur`, before
  the custom passes, bloom and the tonemap), vignette, chromatic
  aberration, lens distortion (`LensDistortion`: the `lens` pass of the
  chain after the tonemap, [Stylisation](#stylisation)), film grain
  seeded by the tick, screen flash and camera shake (a deterministic
  offset of the camera, not a post pass).
- **Decals**: up to `MAX_DECALS` (65,536) projected boxes per frame, read
  by the lit shader from two storage buffers of the effect group (the
  records, binding 5, and their clusters, binding 6: each decal's
  bounding sphere binned into the light clusters' grid, at most
  `MAX_CLUSTER_DECALS` (255) a cluster, the newest kept), each capped to
  its box; views without decal clusters (a planar reflection, a
  render-to-texture view) walk every record. A decal's `normal_rect` is a
  tangent-space normal map in the same atlas, blended over the surface's
  normal after its own normal map by the box's fade. The shader picks a
  decal's mip level itself, capped where its region still spans eight
  texels, and keeps the bilinear taps inside the region, so neither image
  bleeds into its neighbour (pbr.wgsl `decal_sample`).
- **Particles**: `ParticleEmitter` assets and `ParticleSystem`s with
  curves, bursts, noise, sprite sheets, collision against the depth
  buffer (compute) or against `RenderScene::colliders` (the CPU path:
  planes, boxes, spheres and terrain height fields, which og-runtime fills
  from its og.Collider components), soft fade against the scene, local or
  world space, 2D emitters drawn as sprites (from a texture or a texture
  array's layers). Simulated by compute (`ParticleSimulation::Auto`;
  sprite-mode emitters on the CPU), or on the CPU for every emitter with
  `ParticleSimulation::Cpu` (the state is Rust's to read); without
  collision both paths give the same particles
  (`gpu_and_cpu_paths_simulate_the_same_particles`) and a 600-tick run is
  deterministic on either. `ParticleRender::Mesh { mesh }` draws each
  particle as an instance of a mesh, sized by the size curve and turned by
  its rotation and spin about its own random axis, depth-written and
  back-face culled. `lit` shades billboards (as a soft ball facing the
  camera) and meshes with the sun and its cascades, the sky's irradiance
  and the clustered local lights (the particle pipeline binds 8 sampled
  textures). `distortion` (billboards) bends the opaque scene behind each
  particle by up to that fraction of the screen's height along the
  texture's red and green (an untextured one by a smooth screen-space
  field), from `scene-copy` after the opaque pass. `sub_emitters` (up to
  `MAX_SUB_EMITTERS`, 4) fire another emitter's particles at a parent's
  birth, death or collision (`ParticleEventKind`; a collision is a hit
  faster than `IMPACT_SPEED`, 0.5 m/s), one level deep; a system with any
  runs on the CPU path. `CpuParticles::advance` reports the system's
  `ParticleEvent`s, and a host that reads them simulates the system in
  its own tick and hands the renderer its state
  (`RenderScene::hosted_particles`): og-runtime does this for emitters
  with an `events` line once a game names og.ParticleEvent.
- **Trails, ribbons and beams**: `Trail` keeps a per-id history for its
  `TrailStyle`'s lifetime; `Ribbon` is an explicit polyline; `Beam` joins
  two points and scrolls with the tick.
- **2D lighting**: `Lighting2d` with `Light2d`s shadowed by `Occluder2d`s
  (soft penumbrae) and an order-independent glow buffer; only the lit
  sprite layer range is affected; `SpriteFx` adds flash, glow, lit and
  soft per sprite. Tile layers in the range are lit as its sprites are
  (`TileLayer::lit`, true unless the map's layer opts out: og-tilemap's
  `lit false`, a Tiled layer's bool property `lit`, the blob's
  `layer_flag::UNLIT`). Tests: `tests/sprites2d.rs`
  `tile_layers_in_the_lit_range_take_the_2d_light`, scene `tilemap-lit`.
- **The glow in views**: a `View` (views.rs) draws its glowing sprites
  into a chain of its own at half its size and runs the frame's bloom
  filters over it (`bloom_chain.wgsl`'s `downsample13` and `bloom_tent`,
  which post.wgsl's `fs_bloom_down` and `fs_bloom_up` call too), then adds
  level 0 onto its image at the weight the tonemap gives the frame's glow,
  so a sprite glowing in a view and in the frame has the same halo
  (`a_views_glow_spreads_as_the_frames_bloom`); the view has no lit-image
  bloom of its own (post effects are the main camera's).
- **Skinning and blendshapes**: `SkinData` + `JointPalette` (a storage
  buffer, the morph deltas beside the palette), `MorphTargets` weighted
  per instance, up to 64 targets per mesh, their deltas packed sparsely
  and their weights in the palette buffer after the joints, this frame's
  and the previous one's (for the skinned prepass's motion vectors); the
  buffer's layout is in one place, the module doc of `skin.rs` (§The
  palette buffer). og-anim writes the palettes.
- **Stylised materials**: toon ramp, rim light, dissolve per material.
  Every fragment entry point has one `discard` (`dissolve_at` returns a
  flag the caller folds in): on Metal a second discard in an entry point
  does not take effect (`dissolved_pixels_leave_no_trace_in_the_prepass`).

Scenes: `ao-gtao`, `ao-ssao`, `decals`, `dof-low`, `dof-high`, `fx-2d`,
`god-rays`, `motion-blur`, `particles`, `particles-cpu`, `post-grade`,
`skinning`, `stylised`, `trails-beams`; `every_effect_pass_together_under_taa`.

## Atmosphere, water, fluids, material shaders

All off by default: a scene that sets none of them draws no pass for them.

- **Fog** (`RenderScene::fog`, `src/atmosphere.rs`): distance fog from the
  camera (exponential from `start` and, when `end > start`, a linear ramp
  to `end` as well), integrated exponential height fog, forward
  scattering towards `sun_color`, applied in the lit, sky, particle, strip
  and world-sprite shaders (blended effects fade into it by `fog_keep`);
  the overlay is never fogged.
- **Sky** (`SkyLook` on `ProceduralSky::look`, `Background::Procedural`):
  zenith, horizon and ground colours, sun disc and glow, hashed stars, a
  moon disc (`moon_size`, `moon_phase` 0 new to 0.5 full,
  `moon_intensity`, `moon_color`, `moon_direction`, zero meaning opposite
  the sun; lit from the right while waxing, darker maria, a faint
  earthshine and halo, hiding the stars behind it, set below the horizon;
  off at size 0), drawn per pixel so a moving sun moves smoothly; the
  image-based light is generated from the same look (`IblGenerator` keeps
  its pipelines and textures, so regenerating it for a day-night step
  allocates nothing). `day_night(hours, tilt, azimuth)` gives the sun or
  moon light, the sky, the ambient and a fog tint as a pure function of
  the clock. The physical sky and clouds: [Sky and weather](#sky-and-weather).
- **Weather** (`weather(kind, …)`, `src/weather.rs`): rain, snow, dust and
  mist as an emitter placed over the camera (`WeatherLook::origin`), a fog
  and a dimming of the light.
- **Water**: [Water](#water).
- **2D water** (`RenderScene::water_2d`): a side-on body and its lip as
  strips in the sprite order.
- **Metaballs** (`RenderScene::metaballs`, `src/metaballs.rs`): a fluid's
  particles added as kernels into one channel of a field target before the
  main pass, composited at the fluid's sprite layer by a threshold with an
  antialiased edge and a rim. Fluids sharing a `group` are one surface:
  the group takes a field's run of four channels (one per `lane`), a
  particle adds its kernel into its lanes' channels by its `mix`, and the
  surface is thresholded on the sum and coloured by the channels' shares,
  so blended particles draw blended colours. Lone fluids take one channel.
  Surfaces are packed into up to `FIELDS` (4) field targets (`metaballs`,
  `metaballs.1`…; one field pass each), so up to `MAX_FLUIDS` (16) fluids
  draw a frame. Scenes `fluid-2d`, `fluid-groups`.
- **Mesh replacement** (`RenderScene::set_mesh`): cloth and other
  generated geometry, uploaded again in place when the sizes match.

Scenes: `atmosphere-fog`, `atmosphere-dusk`, `atmosphere-night`,
`atmosphere-moon`, `water-3d`, `water-2d`, `fluid-2d`, `shader-hooks`,
`weather-rain`, `weather-snow`; behaviour tests in
`og-tests/tests/atmosphere.rs` and `tests/moon.rs` (phases, size, the
horizon).

### Material shaders

`MaterialShader` (`src/hooks.rs`): a WGSL snippet defining any of
`vertex_offset`, `surface_color`, `surface_normal`, `surface_emissive`,
with up to eight declared parameters read from the material uniform;
`RenderScene::add_shader`, `set_material_shader`, `set_shader_param`. A
material with a shader draws with its own lit, shadow and prepass
pipelines, static and skinned: shadows and the depth prepass are
displaced by `vertex_offset` and cut by `surface_color`'s alpha (the
shadow pass's uniform carries the clock for `h.time`); every other
material draws with the built-in identity hooks. `lit_module_source` is
the module og-cook compiles with naga for every back end. A material
shader makes its batch dynamic for the shadow cache.

## Materials

`og-tests/tests/materials.rs`, scene `materials`:

- **Normals under a non-uniform scale** go through the inverse transpose of
  the model's upper 3×3 (its cofactor matrix, flipped for a mirroring
  transform), for static and skinned meshes and the shadow hooks alike.
- **Vertex tangents.** `Vertex::tangent` (`xyz` along +u, `w` the
  bitangent's sign) in glTF's frame: the bitangent cross(normal, tangent) ×
  w points up the image (towards −v), so a normal map's red tilts towards
  +u and its green up the image. It comes from the file (the glTF bridge
  reads `TANGENT`), from the cook (MikkTSpace on v flipped, so cooked and
  exported tangents agree), or is generated at upload for a mesh without
  (`MeshData::generate_tangents`, Lengyel's method).
- **Clearcoat** (`StandardMaterial::clearcoat`, `clearcoatRoughness`,
  three.js `MeshPhysicalMaterial` names): Filament's clear coat, a GGX
  lobe with 4 % reflectance and Kelemen's visibility on the surface's
  normal before the normal map, over the base it dims by its Fresnel, for
  the lights and the image-based light. Its maps: [Physical
  materials](#physical-materials).
- `StandardMaterial` from reflected material assets with sorted
  transparency (`transparent`: back to front, no depth write),
  double-sided and unlit materials.

## Physical materials

`StandardMaterial` carries three.js `MeshPhysicalMaterial`'s fields under
its names, with glTF `KHR_materials_*` meanings (`src/physical.rs`,
`shaders/physical.wgsl`; `og-tests/tests/physical.rs` and the
`materials-physical` scene):

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
  objects; it casts a shadow like an opaque one and is left out of the
  depth prepass, the planar reflection and GPU occlusion like a blended
  one. `ior` also sets a dielectric's F0, ((ior - 1) / (ior + 1))².
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
clearcoat included, draws with the standard pipelines. A physical
material with every lobe off draws the standard frame.

**Sixteen texture slots per stage: packing, and a pipeline of its own.**
A physical material may sample 18 maps, past the 16 sampled textures per
stage of the downlevel limits. It keeps its standard bind group (every
pass but its lit one: shadows, the depth prepass, the planar reflection
and scatter draw it as before) and gets a second one for the physical
pipelines, whose layout differs:

| Group | Textures |
|---|---|
| 0 frame | cascades, local shadows, irradiance, prefiltered, BRDF LUT (5) |
| 1 physical material | base colour, normal, emissive, gradient, the pack (5), plus the material and physical uniforms |
| 2 physical effects | SSAO, decal atlas, the transmission copy (3), plus the decal records and cells (storage) |
| 3 skin | none: the joint palette and the morph deltas are storage buffers |

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

Pipeline permutations keyed by which map groups a material uses would
need more than one permutation for a material with every map, and a
texture atlas would bleed between maps under mipmapping. The pack costs
one RGBA8 texel per layer used and a few passes when a material is first
drawn. `tests/shaders.rs` validates the physical shader and the pack
shader and translates them to MSL, SPIR-V and HLSL.

**Backends.** Every lobe and map draws on every backend. The
transmission copy's mip chain is made with render passes. The physical
shader asks no texture for its level count; it derives it from the copy's
size.

**From glTF.** The cook and the glTF bridge read the seven extensions with
one parser (`gltf::physical_extensions`, the textures as their kinds: the
sheen colour as sRGB colour, the coat's normal as a normal map, the rest
linear) into `PhysicalExtensions`; a cooked model stores them in its
material rows (`og.ModelMaterial` version 4) and adds a material with any
as a standard material (`CookedModel::standard_material`).

## Array maps (materials and terrain layers)

Texture arrays (`add_texture_array`, og-cook's `type texture-array`) are
sampled by lit materials and terrain layers as well as by sprites, so a
set of variants (tiles, skins, damage states, a terrain's ground types) is
one resource, one material and one draw.

**Materials** (`src/material.rs` `ArrayMaps`, `shaders/pbr_array.wgsl`,
`shaders/shadow_array.wgsl`). `StandardMaterial::mapArray` and
`normalMapArray` are texture-array slots (`TextureArrayHandle::slot`, 0
none) with `mapLayer` and `normalMapLayer`; a material with either reads
its base colour and normal map from the arrays only (`map` and
`normalMap` are not read; a missing one is a one-layer white or
flat-normal array). `Instance::material_layer` replaces both layers for
one instance, so instances of one material on any layers stay one (mesh,
material) batch and one instanced draw. The layer travels in the
instance's 64 bytes (`instances::GpuTransform::layer`: the layer plus
one, 0 for every other instance; a float holds layers exactly to 2^24),
so culling, sorting and GPU culling's compaction carry it without a new
stream. The array-map variant's vertex stages read it at location 8;
array-map materials are never occluders.

One variant of the standard lit, prepass and shadow pipelines draws them,
made when a scene first draws such a material: its module is the standard
lit module's text with bindings 1 and 4 declared `texture_2d_array`, a
flat `layer` varying and `pbr_array.wgsl`'s (`shadow_array.wgsl`'s) entry
points in place of the standard ones. Each rewrite asserts the text it
replaces, so a change to the standard shaders fails `tests/shaders.rs`
instead of drifting. A material's array group is a second bind group (its
standard group stays for shadows with a dissolve, scatter and everything
else); the layers are in the material uniform's `coat.zw`. Skinned
instances take their material's layers.
Physical materials and material shaders keep their 2D path
(`StandardMaterial::draws_array_maps`). og-runtime:
`StandardMaterialDesc.mapArray`, `mapLayer`, `normalMapArray`,
`normalMapLayer` and `og.Mesh.materialLayer` (-1: the material's); an
array a material samples stays resident while it does. Tests: the
material unit tests, `tests/shaders.rs`, og-runtime
`tests/material_arrays.rs`, scene `materials-array`.

**Terrain layers** (`src/terrain.rs`). `Terrain::layer_arrays` names an
albedo array and a normal array, each `TerrainLayer` its `albedo_layer`
and `normal_layer` (in its uniform's `slope.w` and `altitude.w`). Such a
terrain binds the two arrays in place of up to eight textures, through a
variant of the terrain group layout (bindings 5 and 9 as `D2Array`, 6 to
8 and 10 to 12 gone) and of its lit, prepass and shadow pipelines, made
when a terrain first needs it; the variant's modules are the standard
ones' text with the two arrays declared and `layer_albedo` /
`layer_normal_map` reading them. Every `TerrainDraw` carries its own
pipelines and `draw_levels` sets them per terrain, so terrains of either
kind draw in one pass. og-cook's `layers` / `layerNormals` and
`albedoLayer` / `normalLayer` reach it through og-runtime. Tests:
og-runtime `tests/material_arrays.rs`, og-tests
`layer_arrays_draw_as_separate_layer_textures`.

## Compressed textures at runtime

The renderer does not transcode: hosts hand it a `TextureSource`
(`RenderScene::add_texture_source`), RGBA8 level 0 with mips built on the
GPU, or every level pre-encoded in one of `RenderInfo::texture_formats`
(BC7/BC5, ASTC 4×4, ETC2/EAC, RGBA8, chosen per adapter and
`RenderDesc::texture_compression`). og-runtime's loaders pass the
attached backend's formats to the host's texture hook, so cooked Basis
textures reach the GPU as BC7/BC5 on desktop GPUs, ASTC 4x4 on Apple and
most Android, ETC2/EAC RG11 elsewhere (og-cook's README §KTX2 says which
kind gets which). `RenderDesc::mobile()` puts ASTC/ETC2 first.

## Terrain

`src/terrain.rs`, `src/terrain_tiles.rs` (the tile pyramid),
`shaders/terrain.wgsl` (the clipmap vertex, the tile lookups),
`terrain_lit.wgsl` (shading, the prepass), `terrain_shadow.wgsl`. Off
unless `RenderScene::terrains` names one.

- **Data.** `add_terrain(TerrainData)` once (a field in memory);
  `set_terrain` replaces it (a runtime edit). Or streamed:
  `add_terrain_stream(TerrainLayout, range)` and `set_terrain_tile(handle,
  index, Some(Arc<TerrainTile>) | None)` as tiles arrive or leave;
  `set_terrain_stream` starts it over.
- **Tiles.** A terrain is drawn from a pyramid of tiles (`TerrainLayout`:
  level `k` every `2^k`-th sample of the field, cut into tiles of `tile`
  quads, 128 by default; the last level one tile). A `TerrainTile` holds
  `(tile + 3)²` heights (a border of one), its normal map as half floats
  with every mip, and `(control + 2)²` control texels with mips
  (`ControlPyramid`); tiles live in three texture arrays (`R32Float`
  heights, `Rg16Float` normals, `Rgba8Unorm` control), one layer per slot.
  `terrain_tiles_wanted(layout, terrain, range, TerrainView, capacity)`
  refines from the top level down, the tile whose samples look largest on
  screen first, until every sample is under a pixel or `TerrainLod::budget`
  (16 MiB by default, at least 8 slots) is spent, the level-0 tiles within
  `TerrainView::detail` first. A vertex and a pixel read the finest
  resident tile at their point, so a missing tile shows its parent; a
  streamed tile fades in over twelve frames. Levels are decimations of the
  field, so there are no cracks with any set of resident tiles
  (`FrameStats::terrain_tiles`, `terrain_uploads`).
- **The clipmap.** Level `l` is `ring` × `ring` quads (64 by default) of
  `spacing · 2^l` around the camera, its centre snapped to twice its
  spacing so vertices never move in the world. One index buffer holds the
  full grid (level 0) and the ring with its hole at each of the four
  places the finer level's snapping can put it; each level is one indexed
  draw with its uniform slot by dynamic offset. The levels end when the
  next one's hole would cover the terrain or lie beyond the far plane. In
  the outer eighth of each level its odd vertices slide onto the coarser
  grid (CDLOD's morph, by the camera's own position, so it is continuous
  as it moves): at the edge the two levels share vertices and heights bit
  for bit, so there are no cracks and no pops.
- **Layers.** Weight = control channel × slope band × altitude band,
  sharpened by the layers' heights (the albedo's alpha) with
  `height_blend`, normalised (all zero: the first layer); a pixel blends
  its four strongest. Each is sampled over world xz every `tile` m or
  triplanar (Golus's whiteout normals), its normal map applied in the
  frame of the terrain's normal; samples inside branches use
  `textureSampleGrad` with the pixel's own derivatives. The maps are each
  layer's own textures (up to four layers, eight textures bound), or
  layers of two texture arrays (`Terrain::layer_arrays`, up to sixteen
  layers): [Array maps](#array-maps-materials-and-terrain-layers).
- **Light.** The `StandardMaterial` lighting from `lighting.wgsl` (shared
  with pbr.wgsl and water.wgsl): the directional light with cascaded
  shadows, the shadowed local lights, the image-based light, caustics
  under the frame's water, then fog.
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
  (`ScatterDraw { terrain, mesh, material, scatter }`) has the renderer
  place them itself (`src/scatter.rs`), in chunks of 32 × 32 cells built
  once when they come within `distance` (again only when the terrain,
  placement, layers or settings change), each one instanced draw culled
  per pass. Copies past `distance` are dropped and the outer `fade` is
  dithered out, in the lit, prepass and shadow passes alike. The copies
  take their StandardMaterial's colour, cut-off, metalness, roughness,
  emissive, unlit and flat modes, not its normal map or effects.
  `Terrain::scatter(data, TerrainScatter, camera)` answers the same copies
  on the CPU (nearest rows first, at most 65,536); `Terrain::scatter_among`
  is the answer for gameplay. With `TerrainScatter::gpu` (on by default;
  og.TerrainScatter `gpuPlacement`) a compute job places a chunk from the
  level-0 tiles the GPU holds, with nothing read back (`scatter_copies`
  counts only the CPU's). `FrameStats::scatter_draws`, `scatter_copies`,
  `scatter_built`.
- **Impostors** (`src/impostor.rs`). An `Impostor { material, grid,
  center, radius }` is a model baked by the cook (`model { impostor true
  }`) from `grid × grid` directions (an octahedral map) into an albedo
  and a normal-depth atlas, bound as its material's `map` and
  `normalMap`. It draws as one card facing the eye that blends the three
  nearest frames. `RenderScene::impostors` (`ImpostorDraw { impostor,
  transform, cast_shadows }`, one instanced draw per atlas) holds the
  models og-runtime picked at their impostor level; a `ScatterDraw` with
  an `impostor` draws its copies past `impostor_distance` as cards,
  cross-dithered with the meshes over the last tenth before it.
- **Wind.** `RenderScene::wind` (`Wind { strength, direction, speed, gust
  }`; `None` or strength 0 still; og-runtime makes it from og.Wind with
  `Wind::blowing`, a lean of `Wind::LEAN_PER_MPS` per m/s) sways the copies of every
  scatter whose `TerrainScatter::sway` is above 0, in the vertex stage
  (`scatter_sway`): a vertex `h` above the copy's origin moves downwind by
  `lean × h × min(h / top, 1)`, the lean `strength × sway × (1 + gust ×
  w)`, `w` a two-octave wave travelling downwind at `speed` over
  `WIND_WAVELENGTH` (10 m). The phases are reduced in f64 from the tick,
  and the previous frame's are kept for motion vectors. `wind_shader()` is
  the same sway for placed models, a material shader (`og.wind`) that
  og-runtime gives an `og.Model` with a `sway`. Test
  `wind_sways_scattered_copies_downwind`.

Scenes: `terrain-near`, `terrain-mid`, `terrain-far` (the 2 km island of
`og_tests::terrain_scenes`, clipmap, four layers, shadows, fog, from 2 m,
150 m and 700 m) and `terrain-grass-wind` (the meadow in a gusting wind at
tick 90; the boulders, plain instances, stand still); tests in
`og-tests/tests/terrain.rs`. `plant-grown` (og-tests-cooked's
`plant_scenes`): an oak, a pine, a flowering shrub and daisies the cook
grew from `plant` blocks, each drawn as its wood and its leaves;
`plant-scatter`: a 64 m hill with the oak, the pine and the shrub
scattered whole (a `ScatterDraw` per part, the leaves `with` the wood),
each plant's copies cards from where a card's texel is a pixel (12 radii
at 720 rows), in a light wind; `plant-scatter-switch`: oaks and pines
along a 224 m strip, so both cross from mesh to card in the frame.

## Water

`RenderScene::water` (`src/water.rs`, `shaders/water.wgsl`): a grid
displaced by an optional height field, travelling detail normals, colour
by the water's depth over the scene (the depth prepass, which a frame
with water always runs), foam where it meets geometry and on steep
crests, refraction of a copy of the scene, reflections by a march through
the depth prepass and the prefiltered sky cubemap where that misses. The
main pass splits after the sky: the resolved scene is copied to
`scene-copy` (`hdr` gains `COPY_SRC` only in frames with water), then
water, transparents, particles, strips and sprites draw over it. Each of
the following is off by default on `Water`:

- **Planar reflection** (`planar_reflection`, the fraction of the output's
  resolution): one surface a frame, the first that asks (a grid surface
  first), with the camera above it and perspective. With
  `RenderDesc::planar_reflections` 2, a second surface in view draws its
  own into `reflection.1`, and the effect group's binding 4 lets material
  shaders read the first (`planar_reflection`, planar.wgsl; the
  lightmapped pipelines then take a group without it, `fx_lean_layout`).
  With 1, a second surface reflects as without one. The camera mirrored
  in the surface (`look_at` of the mirrored eye, target and up: a proper
  rotation, so the image is flipped left to right and winding is
  unchanged) draws the opaque meshes, skinned meshes, terrain and sky into
  the graph's `reflection` target (`FramePass::Reflection`, before the
  main pass; multisampled with a resolve under MSAA) with its own frame
  and effect uniforms. Its projection's near plane is the water's plane,
  2 cm up (Lengyel's oblique clipping), so nothing below the surface is
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
  94 × 94 cells, not two triangles.

Scenes: `water-3d`, `water-planar`, `water-shadows`, `water-underwater`;
tests in `og-tests/tests/terrain.rs`.

### The sea and flow maps

Each off by default on `Water`:

- **Sea** (`wind`, `swell`, `choppiness`; og.Water.wind): the grid is
  displaced by `og_core::ocean`'s 32 Gerstner waves of a Pierson–Moskowitz
  sea for the wind with JONSWAP's peak enhancement (γ 3.3) (peak
  `0.877 g / U`, significant height `0.21 U² / g × swell`, one wave placed
  at random in each of 32 bands of length from 1.6 to 1/80 of the peak's;
  deep-water dispersion; a random spread about the wind, wider the
  shorter; fixed phases; the waves under a sixth of the peak's length
  raised to Cox and Munk's mean square slope `0.003 + 0.00512 U`; each
  wave's steepness `choppiness / 32`, so the sum never folds). A sea's
  grid is finest at its centre, `w(g) = g (0.15 + 0.85 g²)` across it. The
  waves lie over the absolute world's xz, their phases at the scene's
  origin computed in f64 from the tick (`water::seconds`), so the pattern
  stays put as the origin moves. The vertices take the waves at least four
  grid cells long, the fragments every wave's analytic normal (faded under
  the pixel's footprint) and foam whitecaps where the crests bunch, after
  Monahan's whitecap cover for the wind (`3.84e-6 U^3.41`). The simulation
  reads the same surface: `og.physics.waterHeight` evaluates
  `og_core::ocean::Ocean::height` at the tick, never the GPU; the
  underwater view tests the eye against it too. A level surface only; a
  surface over a mesh has none. Tests `water::tests`,
  `og_core::ocean::tests`.
- **The drawn sea and the tick's sea.** The drawn sea adds render-only
  FFT cascades (`ocean_cascades` in `water.wgsl`) to the Gerstner waves
  the tick reads, so it lies off `og.physics.waterHeight` by decimetres.
  Under a hull `hull_calm` fades the FFT chop over each floater's
  footprint, so a floater sits in the sea that is drawn
  (`shallow_floaters_sit_in_the_drawn_shoaled_sea`). A deterministic CPU
  FFT band the tick reads is not implemented.
- **Flow map** (`flow_map`; three.js Water2's flowMap): rg × 2 - 1 is the
  flow's direction per point over the mesh's uv or the grid from its -x -z
  corner (the image's top at +z), `flow`'s length the speed (zero: 0.03),
  for the normal map's two layers (Water2's look) or, without a normal
  map, the travelling waves, whose fronts then lie across the flow: two
  copies sliding along it, restarting every three seconds half a period
  apart and cross-faded. Binding 9 of the water group.

Scenes (`og-tests/tests/ocean.rs`, `og_tests::ocean_scenes`):
`water-ocean` (a 9 m/s breeze in the afternoon), `water-ocean-storm` (a
17 m/s gale at full choppiness under an overcast, whitecaps),
`water-ocean-sunset` (a 6 m/s breeze into the low sun: the glitter path),
`water-flow-map` (a stream parting round a boulder).

## 3D fluids

`Scene::fluids_3d` (`src/fluid3d.rs`, `shaders/fluid3d.wgsl`,
`shaders/fluid3d_blur.wgsl`; og-runtime fills it from every visible
`og.Fluid3D`) draws particle fluids as one screen-space surface. The CPU
uploads each frame's particles as instances (their positions come from
og-physics' CPU solver), so the passes need no storage buffer or compute.
The frame pass `fluid-3d` runs after the depth prepass and before the
main pass, while a fluid has particles (the prepass and the split main
pass are then forced on):

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
   optical depth), lights the body with the sun and its cascades, the
   image-based light and the clustered local lights, reflects the
   environment by Schlick's Fresnel times `reflectivity`, fogs, and writes
   its depth, so later transparent draws and the frame's effects see it.

All the frame's fluids share one surface: they merge where they meet, each
particle carrying its own colour (a group's mix colours them), but the
look (reflectivity, refraction, smoothing) is the first fluid's. The cost
is two blur passes and a composite over the pixels the fluid covers plus
two instanced draws, whatever the particle count; marching cubes is not
implemented. Scene `fluid-3d` (a blue pool in a tiled basin with an orange
drop above it, lit by the sun and an orange point light),
`og-tests/tests/fluids3d.rs`.

## Environment maps, AgX, exposure adaptation

**Environment maps** (`src/environment.rs`, design §2.6 "IBL from an
environment map prefiltered at cook time"). og-cook's `type environment`
prefilters an equirectangular `.hdr`/`.exr` at cook time with this crate's
`bake` feature (it must match the renderer's face orientation, level
roughness and ibl.wgsl's GGX sampling, so it lives here; the runtime wasm
never enables it): the level-0 cube in three.js's equirectangular mapping
(the centre column is +x), levels 1–4 by GGX importance sampling at
roughness m/4, and the irradiance as order-2 harmonics (stored as E/π).
Texels are RGB9E5 (`Rgb9e5Ufloat`, 4 bytes), sampled and filtered on
every backend. A KTX2 cube whose own mips are already GGX-prefiltered
(og-cook's `prefiltered true`: level k of L at roughness k / (L − 1)) is
not filtered again: `bake::from_prefiltered` resamples its levels to
roughness m/4.

`RenderScene::add_environment_map` uploads it as a cube; a frame sets
`environment_map: Some(EnvironmentMap { map, rotation, background_intensity })`.
The IBL generator then copies it, turned about +y, into its own cubes
instead of drawing the sky (`fs_env_copy`: level 0 into the 256² sky cube
scaled by the background intensity, whose mips are downsampled as for the
sky, level m into prefiltered mip m; `fs_env_irradiance`: the harmonics
evaluated into the 32² irradiance cube), only when the map, its turn or
background intensity changes. Every shader that reads the image-based
light (pbr, terrain, water, the sky pass, the planar reflection) reads the
map with no change; `Environment::intensity` still scales the light and
`Background::Sky` draws the map.

**AgX** (`Tonemap::Agx`, `post.wgsl`'s `agx`): linear sRGB to Rec.2020,
the inset matrix, log2 encoding over −12.47..+4.03 EV, the sixth-order
contrast polynomial, the outset, 2.2 back to linear and Rec.2020 to sRGB:
the base look of Blender 4, Filament and three.js `AgXToneMapping`. ACES
is the default.

**Khronos PBR Neutral** (`Tonemap::Neutral`, `post.wgsl`'s and
`views.wgsl`'s `neutral`; ported from RedGPU's `khronosPbrNeutral`, the
same curve as three.js `NeutralToneMapping`): the 0.04 of a dielectric's
Fresnel reflection is taken off (less near black, `x - 6.25x²` under
0.08), so a base colour lit by white light shows as authored; a colour
whose peak is then under 0.76 passes through, a brighter one is scaled so
its peak rolls off as `1 - 0.24² / (peak - 0.52)` and mixed towards white
by `1 - 1 / (0.15 (peak - new peak) + 1)`.

**Exposure adaptation** (`Exposure::Auto { adapt_speed_up,
adapt_speed_down }`, EV per second; the IDL's
`RenderSettings.adaptSpeedUp`/`adaptSpeedDown`): the exposure-adapt pass
(`shaders/exposure.wgsl`) moves a 1×1 `R32Float` EV (two textures, frame
`n` reading the other's and writing `n % 2`'s) towards the metered target
by at most speed × dt, where dt is the scene's tick delta since the
previous adapted frame over `tick_rate`: the tick owns time, nothing reads
a clock, so the same ticks give the same frames however often they are
drawn (a capture at the same tick moves nothing). A speed of 0 is instant
in its direction. The first adapted frame, one after a frame without
adaptation, or one whose tick went back starts at the metered EV; a frame
of tick 0, drawn before the first tick, meters at once and is never
adapted from, so hosts that draw a tick-0 frame (the web shell behind the
play gate) and hosts that do not adapt alike. The adapted EV is
world-time state: `Renderer::save_state` copies it (with its tick) into a
64-texel ring on the GPU, no readback, and `restore_state` copies it back
into both textures; og-probe's snapshot, restore and sweep call them
through `Capture::save_state`/`restore_state` (og-runtime's
`RenderBackend`), so a sweep's values all start from the call's exposure
and a restored world redraws its frames. `og-tests/tests/exposure.rs`.

## Stylisation

The primitives a look composes into a non-photographic frame
(`src/stylise.rs`; the IDL's `Stylise` and `MaterialStyle` resources, set
by a `type look` asset through `og.render.applyLook`), each off at its
zero value and then costing nothing: no pass is recorded, no pipeline
built. `RenderCapabilities.stylise` names what this renderer draws
(`STYLISE_DRAWN`). All of it draws on every backend (render passes only).

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
  at most one column gives way to the scale (240 rows at 1280 × 720 are
  426 columns at 3x). Test `stylise.rs`'s
  `the_pixel_grid_keeps_its_scale_and_drops_at_most_a_column`.
- **A look change is a cut**: when the primitives that are on change
  between two frames (`Stylise::chain_bits`: a pass added or removed, the
  pixel grid, retro 3D, the material style; not their settings), the
  renderer drops the TAA history as `reset_history` does
  (`FrameStats::taa_history` false there; `tests/taa_look.rs`).
- **The chain after the tonemap** (`chain_passes` in post.rs): the tonemap
  writes `ldr.0` (sRGB8) when any pass follows, and each of `paint`,
  `lines`, `lens`, `quantize`, `fxaa` and `upscale` reads one LDR target
  and writes the next, the last into the output (sRGB-encoded in the
  shader where the output is not an sRGB format). Pipelines are made on
  first use (`PostPasses::ensure_chain`).
  - `paint`: a Kuwahara filter of four sectors (turned and stretched along
    the image's edges by `anisotropy`), pigment pooling at the edges, a
    bleed from a ring two radii out, and a tileable canvas texture laid
    over the frame through a repeating sampler, or on the surfaces
    (`Paint::canvas_space`, below).
  - `lines`: cross-hatching from the tone (one stroke set per level, each
    turned further, over a paper-light ground), halftone dots on a rotated
    grid sized by the tone, and ink on luminance edges (a Sobel gradient,
    widened by the max over the line's width). With `Ink::depth` (the
    IDL's `Stylise.inkDepth`) above 0 the frame runs the depth-normal
    prepass and the lines pass binds its `lindepth` and `gbuffer`
    (stylise bindings 7 and 8; placeholders for the other chain passes
    and frames without one): a pixel is also inked, at that strength,
    where a neighbour at the line's width is farther by more than
    `Ink::depth_threshold` (4 % by default; a silhouette, on the nearer
    surface) or of about the same depth and turned past
    `Ink::normal_threshold` (1 − cos, 0.4 by default; a crease), the
    outline pass's defaults (the IDL's `inkDepthThreshold` and
    `inkNormalThreshold`), never on the sky.
  - `lens`: `PostEffects::lens_distortion` (it bends the frame after the
    two passes that read the prepass, so their lines bend with the image,
    and before the quantisation, so a palette stays exact): each pixel
    samples the image at the centre plus its offset times `1 + amount r²`
    (r in pixels, 1 at a centred lens's corners) times the scale that
    keeps a barrel's farthest corner on the image, bilinear, or nearest
    under a pixel grid; black beyond the image.
  - `quantize`: posterisation in display space with an ordered Bayer
    2/4/8 dither (bit-interleaved in the shader), blue noise or
    interleaved-gradient noise (`DitherPattern::InterleavedGradient`),
    then the nearest colour of a 256-entry palette
    (`RenderScene::add_palette`, a 256 × 1 sRGB texture) by Oklab
    distance. The blue noise is a 64 × 64 void-and-cluster tile
    (`src/blue_noise.rs`, `data/blue-noise-64.r8`, 4096 B, regenerated
    and compared by its test). A pixel whose drawn colour lies within 6
    sRGB levels a channel of a colour of art the cook conformed (a
    texture's `<id>#palette`, added with
    `RenderScene::add_conformed_colors` as the texture loads) is left as
    drawn: posterise and dither step only what the cook did not conform
    (`stylise::conformed_table`, a spatial hash at binding 11).
- **Surface space** (`StyliseSpace::Surface`: `Paint::canvas_space`,
  `Hatching::space`, `Halftone::space`; the IDL's `paintCanvasSpace`,
  `hatchingSpace`, `halftoneSpace`, capability `surfaceSpace`): the
  pattern is laid on the surfaces instead of over the frame. The frame
  runs the depth-normal prepass (`Stylise::on_surfaces`); each pixel's
  point is rebuilt from its view depth relative to the 1024 m grid point
  under the camera (f64 on the CPU, `post::surface_origin`) and its world
  normal weights three planar projections (|n|⁴). The canvas is sampled at
  `canvas_world_scale` texels per metre (128 by default). Hatching and
  halftone lay the screen pattern in metres on each plane, blending the
  two power-of-two octaves around `scale` (`size`) pixels (a tonal art
  map), with the footprint taken along the depth gradient (at most
  `FOOTPRINT_STRETCH`, 32, pixel widths). The sky keeps the screen
  pattern. The space is a setting, not a pass.
- **Object space** (`StyliseSpace::Object`, capability `objectSpace`):
  the surface patterns laid in each mesh instance's own space, so a
  moving, turning or skinned object carries its marks. After the prepass
  the object pass (`stylise_object.rs`) draws the opaque instances again
  against the prepass depth into an `Rgba32Uint` target (the position in
  metres along the instance's axes, a skinned one's blended bind pose, and
  an octahedral normal), read at stylise binding 9. Terrain, scattered
  copies and vertices a material shader's vertex offset moves keep the
  world pattern. *Limits*: alpha-cut holes are not cut; neighbouring
  instances meet with unrelated pattern phases. It costs a second geometry
  pass, only on frames a look uses it. `tests/stylise_space.rs`.
- **Hatching from depth** (`Hatching::depth`, the IDL's `hatchingDepth`,
  0 to 1): the prepass's silhouettes and creases (`Ink`'s thresholds, at
  1, half and one stroke spacing, `contour_at`) darken the tone the
  strokes follow, so the strokes gather along contours and folds; the
  frame then runs the prepass (`Stylise::depth_edges`).
- **Depth bands** (`DepthBands`): an HDR pass after the main pass
  (fxpost.wgsl `fs_depth_bands`, over the prepass depth) cuts the scene
  from `near` to `far` into `count` equal layers, the first in the near
  colour, the last (from one band before `far`, and beyond it) in the far
  colour, those between stepping evenly in OkLCh (`band_color`: lightness
  and chroma straight, hue the short way round, a grey end taking the
  other's hue). `flatten` 1 replaces the shading with the band's colour;
  0 tints it (the lit colour times the band's colour at unit luminance);
  between, a mix. The outline pass reads the banded image. The bands
  follow view depth.
- **Retro 3D** (`Retro3d`, pbr.wgsl `retro_clip` and `stylised_uv`): the
  clip position snapped to a screen grid of `vertex_snap` logical pixels,
  depth quantised to `depth_bits`, and affine texturing by interpolating
  uv × w and dividing by the interpolated w in the fragment. The physical
  shader shares the vertex path; the prepass alpha cut stays
  perspective-correct.
- **The material style** (`MaterialStyle`, material.rs `styled`): every
  lit material that leaves a stylised field at its default takes the
  scene's (toon and its ramp, the rim, the outline, flat facets),
  re-uploaded whenever the style changes; `shadow_tint` moves shadowed
  sunlight towards a colour, and `outline_distance` scales each material
  outline's width by that distance over the view depth, at most four times
  the width (never for an orthographic camera). The outline pass lines a
  surface at a grazing angle only where it ends (`surface_ahead`). A
  material with `styled` 0 (the IDL's `StandardMaterialDesc.styled false`)
  is drawn exactly as declared; a material with a physical lobe
  (`StandardMaterial::has_physical_lobes`) keeps its physical shading
  under the style's toon and takes the rest of the style (a game that
  wants it toon sets its own `toon`).
- **Hull outlines** (`StandardMaterial::outlineWidth`, the style's
  `outline_width`; fxpost.wgsl `fs_outline`): a material's own outline is
  what toon shading's inverted hull draws: the object's silhouette, where
  one part of it passes in front of another, and its creases against
  *other* objects, never the edges between its own faces. The prepass
  writes each fragment's object id (1 to 255, a hash of the instance's
  transform; 0 for terrain and scattered copies) into the outline target
  beside the colour (5:6:5) and width (eighths of a pixel;
  `outline_bytes`), and the outline pass lines a crease only where the ids
  differ, so an unwelded mesh (a `.vox` model, a flat-shaded rock) stays
  clean. Neighbouring objects share an id one time in 255. The scene's
  outline (`PostEffects.outline`) still lines every crease.
- **Built-in canvases and palettes**: a look names the engine's own
  `"@canvas/paper"`, `"@canvas/linen"`, `"@canvas/watercolor"` (tileable
  256² grey, the paint pass reads the red channel) and `"@palettes/lcd2"`,
  `handheld4`, `dusk16`, `bright32`, `1bit`; the cook adds the ones its
  looks name to the bundle under reserved ids (og-cook's README, *Built-in
  look assets*), so the renderer sees an ordinary texture and palette.
- Beside the frame's stylisation: sprite drop shadows ([The 2D
  batch](#the-2d-batch)) and og-anim's stepped playback
  (`AnimationPlayer.stepFps`) for stop motion on twos.

Scenes: `stylise-pixel`, `stylise-palette`, `stylise-print`,
`stylise-halftone`, `stylise-paint`, `stylise-bands`, `stylise-bands-tint`,
`stylise-retro`, `stylise-material`, `stylise-ink-depth` (flat unlit shapes
of one colour where every line is a silhouette or crease),
`stylise-surface` (hatching and a paper canvas on clay forms, depth ink),
`stylise-halftone-surface`, `stylise-object` (the surface drawing in object
space with a tilted box and a beam), `stylise-outline-ground` (a long
ground under a chase camera, lined only where it ends),
`stylise-outline-voxels` (unwelded voxel models under a toon outline),
`stylise-toon-physical` (a toon style over plain, lacquered, jelly and
unstyled spheres). The looks presets of `templates/looks/` are drawn by
fx-showcase's `looks` station, which the browser tier compares too
(`reference.ts --looks` in `scripts/check.sh web-reference`).

## Sky and weather

**The physical sky** (`sky_physical.rs`, `shaders/sky_scatter.wgsl`):
`SkyLook::physical` set to a `PhysicalAtmosphere` (Earth's by default,
Hillaire 2020's coefficients) draws the sky by single scattering through
air (Rayleigh) and haze (Mie, Cornette–Shanks phase) with ozone
absorption, in a spherical atmosphere, plus Hillaire's multiple-scattering
term. The sky pass marches 24 samples per pixel, the image-based light's
sky 16; the optical depth towards the sun is the Chapman function's closed
approximation, so there is no transmittance texture and no new binding:
the atmosphere is 38 vec4s of the frame uniform (`frame.sky_scatter`) and
of the IBL generator's parameters (a 1024-byte slot). The
multiple-scattering table (16 sun angles at 0 and 8 km) is integrated on
the CPU once per atmosphere and cached. The same formulas run on the CPU
(`PhysicalAtmosphere::radiance`, `sun_transmittance`), which
`DayNightLook::with_atmosphere` uses for the sun's colour through the air
and the fog's colour. Saturated highlights of the drawn sky and the sun's
disc are rolled off keeping their hue (`sky_rolloff`); the disc darkens
towards its limb. The night floor (`PhysicalAtmosphere::night`) is set
above airglow, so land under a twilight or night sky keeps a deep
blue-grey ambient. The brightness is scaled so the sky's mean irradiance
at noon matches the gradient sky's. Scenes `sky-physical-noon`,
`-sunset`, `-twilight`, `-hazy`.

**Clouds** (`clouds.rs`, `shaders/clouds_noise.wgsl`, `sky.wgsl`'s
`clouds_over`): `RenderScene::clouds` draws, over the procedural sky, a
volumetric cumulus layer raymarched between two shells around the planet
(48 steps by default, up to twice that towards the horizon, jittered per
pixel and tick by interleaved gradient noise; five light samples towards
the sun with three octaves of Wrenninge's multiple scattering, a two-lobe
Henyey–Greenstein phase, the irradiance cube as ambient) and a streaked
cirrus layer above it, faded into the sky with distance. Under the
physical sky the sun on a sample is the sunlight above the air through
the sky's own transmittance down to that height; otherwise it is the
frame's directional light. A lightning strike lights them from inside
about the bolt's root (`Clouds::flash`, set by `add_lightning`). Their
shapes come from a 64³ tiling noise volume (Perlin–Worley and value noise
equalised to uniform, Worley detail, coverage), made by one compute pass
the first frame with clouds (until then the sky group binds a 1³
placeholder), read at two scales blended across the sky, cut flat at the
base and eroded most at the tops. The wind moves them by the tick's clock
(the offset wraps at four tiles). They are not in the image-based light;
`Clouds::light` is how much their cover dims the sun and the sky, which
the runtime applies. Scenes `clouds-fair`, `-overcast`, `-sunset`,
`-gradient`.

**Weather on surfaces and lightning** (`weather.rs` `SurfaceWeather`,
`add_lightning`; `shaders/weather_surface.wgsl`):
`RenderScene::surface_weather` wets, puddles and snows every standard,
physical (lean included) and terrain surface from two vec4s of the frame
uniform: no binding is added to the lit stages. Wet porous surfaces darken
at their own roughness and sealed ones, smooth or metal, gain a sheen
(Lagarde 2013); puddles fill the hollows of a world-space noise on level
ground, mirror-smooth, rippled by raindrop rings while it rains; snow
settles on what faces up, deeper cover reaching steeper faces, its edge
broken by noise. The masks follow the world position and the geometric
normal. Roofs shelter what is under them through the weather occlusion
map (`shelter_view`): the depth nearest the sky, drawn like a shadow
cascade along the fall (slanted by og.Weather's wind, at most 45°) over
`SurfaceWeather::shelter` metres around the camera (64 by default; 0:
none), filtered 5×5 over `DRIP_EDGE`. It is the cascade array's last
layer while the scene has surface weather (its index from
`frame.weather_shelter.x`, never a constant), takes only static casters
and terrain, and is redrawn only when `shelter_key` changes (the camera
crosses an eighth of the square, the fall turns, or a caster inside
changes). The tick's wetness, puddles and snow depth are
the open ground's; the map is drawn only. A strike (`add_lightning`) is a
double flash of the sky's light, a light over the strike and a bolt by
midpoint displacement seeded by the strike's number. The state they draw
is the runtime's og.WeatherState, simulated in the tick (og-runtime
`weather.rs`). Scenes `weather-wet`, `-snow-cover`, `-lightning`,
`-shelter-snow`, `-shelter-rain`, `-shelter-meshlet`.

## Reflection probe faces

A due capture draws six faces (`probes.rs`, `lib.rs`'s probe pass; the
order and what each face draws: `docs/design.md` §2.6). What the faces
draw besides the capture itself:

- **Face targets** (`FaceTargets`, one set per face size): the face's
  depth prepass, its copy (water and 3D fluids refract it) and the 3D
  fluids' targets, 24, 8 and 36 bytes a texel. They are made the first
  frame a capture of that size needs them and kept only while one is due
  or a realtime probe of that size draws its own (`face_target_sizes`): a
  baked probe's go the frame after its capture and are made again if it
  is ever due again, so a scene of baked probes holds none after its first
  frame. A realtime probe keeps them between its captures. A capture drawn
  after the targets were freed and remade is bit for bit the first;
  `probes::tests::face_targets_are_kept_for_due_and_realtime_captures_only`.
- **Water's reflections.** A face's water marches its screen-space
  reflection through the face's own depth prepass and copy, as the
  frame's water does through its own (`WaterRenderer::face_draws` keeps
  `look.y`): what the surface mirrors as seen from the probe's point. Rays
  that leave the face fall back to the environment, as rays leaving the
  screen do in the frame. og-tests probes
  `water_in_a_capture_reflects_the_face_in_screen_space`. The planar
  reflection stays off in faces: it is a second pass from the mirrored
  camera, one surface a frame, and a face would need its own (six per
  capture, each with an oblique clip and its own target).
- **What water and 3D fluids see through (left out, deliberately).** The
  face's copy is taken after the sky, before the transparent meshes, so
  water and fluids in a face show the opaque face behind them and not
  glass drawn behind them. This is the main pass's rule too (its
  `scene-copy`, and the transmission copy physical materials refract, as
  three.js's transmission pass renders only opaque objects), and a face
  must draw what the frame draws or its reflections disagree with the
  scene they reflect. Changing it for both would need the transparents
  ordered against the water and fluid surfaces: water is drawn in one
  pass of its own and the 3D fluids as one screen-space surface with no
  plane to sort a mesh against, a pane crossing a surface cannot be cut
  per instance, and drawing the transparents into the copy and again
  after would blend them twice.

## What phase 2 owes: decisions

Choices made on the renderer's open lines, and what is not built:

- **The depth prepass runs on demand**, not every frame: forced on every
  frame it changes no lit pixel (the lit pass keeps its own depth test,
  and the clusters come from light volumes, not scene depth) and costs a
  second geometry pass. `docs/design.md` §2.6 says so.
- **Instance upload dirty tracking**: the CPU path's instance buffer, GPU
  culling's input region and the previous transforms of object motion
  keep a copy of what they hold and write only the 4 KiB blocks that
  changed (`src/upload.rs`, [The render graph](#the-render-graph));
  `FrameStats::instance_upload_bytes`; og-tests `tests/upload.rs`.
- **Shader cooking per backend**: every WGSL module
  (`og_render::shader_sources`) is validated by naga and written as MSL,
  SPIR-V and HLSL in the gate (`tests/shaders.rs`), so a shader error is a
  failed build, not a player's crash. The wasm ships WGSL: the browser
  compiles WGSL itself.
- **The cluster build stays on the CPU**: a compute build (one dispatch,
  no importance order) costs more to upload, encode and submit than the
  CPU build takes, plus its GPU time (og-tests `tests/cluster_compute.rs`,
  an ignored measurement run by hand).
- **Fog** is applied in the lit, sky, particle, strip and sprite shaders,
  not as a post pass ([Atmosphere](#atmosphere-water-fluids-material-shaders)).
- **Text**: kerning, shaped placements and glyph tables that grow at run
  time are here; shaping and bidi are og-ui's, CJK coverage the cook's
  `runtime true` with og-ui's run-time glyphs.
