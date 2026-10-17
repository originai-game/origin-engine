# og-render

The Origin Engine renderer (`docs/design.md` §2.6): wgpu, WGSL shaders in
`crates/og-render/shaders/` at the repository root, one code path for native (Metal, Vulkan,
headless for executors) and the browser (WebGPU, WebGL2 through wgpu's GL
backend). It started as the spike's renderer (`crates/og-spike`) moved
behind a crate boundary; M0 reproduced the spike's frame byte for byte
(`tests/evidence/M0/spike-frame-reproduced.png`) before the sky pass and mips
were added.

The crate knows nothing about the ECS. Its whole input is a `RenderScene`; the
world → scene extraction belongs to the layer above.

## Public types

| Type | What it is |
|---|---|
| `RenderDesc` | How to create a renderer: `width`, `height`, `backend` (`Auto`, `WebGpu`, `WebGl2`, `Metal`, `Vulkan`), `output_format` (`None` = `Rgba8UnormSrgb` offscreen, the surface's choice on a surface). Default 1280×720, `Auto`. |
| `Renderer` | `Renderer::new(&desc).await` (headless), `Renderer::with_surface(&desc, &instance, surface).await` (presents), `render(&scene, &target) -> Result<FrameStats>`, `resize`, `info`. Native only: `new_blocking`, `wait_idle`, `read_offscreen() -> RgbaImage`. |
| `render_to_rgba(&desc, &scene)` | Native, blocking: one frame headless → `RgbaImage` (RGBA8, rows top to bottom). The executor and capture path. |
| `RenderTarget` | `Offscreen` (the renderer's own texture, readable), `Surface` (acquire, draw, present), `View(&TextureView)` (a caller's view of the output format and size). |
| `RenderScene` | Resources with handles (`add_mesh`, `add_texture`, `add_material` → `MeshHandle`, `TextureHandle`, `MaterialHandle`; `set_material` replaces one), and the per-frame public fields `instances`, `camera`, `light`, `environment`. Resources are append-only; the renderer uploads what is new or changed since the last frame. |
| `MeshData`, `Vertex` | Triangle lists: position, normal, uv, colour; `u32` indices. |
| `TextureData` | RGBA8 pixels with `color_space` (`Srgb`, `Linear`) and `mips` (`Generate`, `None`). |
| `Material` | glTF metallic-roughness as a plain struct: base colour, metallic, roughness, normal scale, emissive, their optional textures, `alpha_mode` (`Opaque`, `Mask { cutoff }`). The reflected material asset of M3 will map onto it. |
| `Instance` | `(mesh, material, transform)`. Callers submit instances one by one; the renderer batches them. |
| `Camera` | `position`, `target`, `up`, `fov_y_degrees`, `near`, `far`, `exposure`. Perspective, right-handed, depth 0..1. |
| `DirectionalLight` | `direction_to_light`, `color`, `intensity`, `cast_shadows`, `shadow_bounds` (`Auto` fits the instances' world bounds; `Sphere { center, radius }`). |
| `Environment` | `sky: ProceduralSky { sun_direction, sun_color }`, `intensity` (IBL strength), `background` (`Sky` or `Color(linear rgb)`). |
| `FrameStats` | `draw_calls` (every pass), `batches`, `instances`, `triangles` (main pass). |
| `RenderInfo` | Adapter, backend, device type, `Tier` (`Full`, `NoCompute`), `backend_tier()`: the reference-frame directory name. |
| `RenderError` | `NoAdapter`, `NoDevice`, `Surface`, `InvalidHandle`, `Readback`, `Unsupported`, `InvalidData`. |
| `shapes` | `sphere(segments, rings)`, `plane(size, uv_repeat)`, `checker(size, cells)`. |
| `gltf` (feature `gltf`) | `load_glb(bytes) -> GltfModel` (node transforms baked into the meshes), `GltfModel::add_to(&mut scene) -> ModelParts`, `ModelParts::instance(&mut scene, transform)`. A bridge until the cook (M4); the runtime wasm leaves the feature off. |

## The render graph as it is now

A fixed sequence, not yet a graph object. Per `render` call:

1. **Sync.** New meshes become vertex and index buffers; new textures are
   uploaded (level 0 via `write_texture`) and, with `Mips::Generate`, get a
   full mip chain from the **mip pass** (one fullscreen triangle per level,
   sampling the level above through a single-level view with a linear filter:
   a 2×2 box filter in linear space, since sRGB is decoded on sampling and
   re-encoded on write). New or changed materials get a uniform buffer and a
   bind group. When `environment.sky` differs from the last frame's, the
   **IBL passes** run (below).
2. **Batch.** Instances are stably sorted by (mesh, material); each run is a
   batch. All transforms go into one instance buffer, batch after batch; a
   draw binds its batch as a buffer slice (not `first_instance`, which WebGL2
   lacks). Instances of an empty mesh are skipped.
3. **Uniforms.** View-projection and its inverse, the light's orthographic
   shadow box around `shadow_bounds` (looking at the centre from
   `2 × radius` along the light), radiance, IBL intensity, exposure.
4. **Shadow pass** (when `cast_shadows`): depth from the light into a 2048²
   `Depth32Float` and, in the same pass, into an `R32Float` colour target.
   Front faces culled, depth bias 2 / slope 2. One instanced draw per batch.
5. **Main pass** into the `Rgba16Float` HDR target with `Depth32Float`:
   forward PBR (`pbr.wgsl`): GGX direct light × a 3×3 PCF shadow factor
   (explicit compares on `textureLoad` of the `R32Float` copy), split-sum IBL
   (irradiance cube, prefiltered specular mips, BRDF LUT), emissive, alpha
   mask by discard, normal maps with a cotangent frame from screen
   derivatives (no vertex tangents). One instanced draw per batch.
6. **Sky pass**, in the main pass, when `background` is `Sky`: a fullscreen
   triangle at the far plane (`LessEqual`, no depth write) that unprojects each
   pixel and samples the sky cubemap. Drawn after the opaque batches, so it
   shades only uncovered pixels. With `Color(c)` the HDR target is cleared to
   `c` instead.
7. **Post pass**: HDR × exposure → ACES (fitted) → the output. An sRGB output
   format is encoded by the hardware; a non-sRGB one (WebGPU canvases are
   `Bgra8Unorm`) by the shader (`fs_srgb_encode`). Draw calls per frame are
   therefore `2 × batches + 2` with shadows and the sky.

**IBL passes** (`ibl.wgsl`, only when the sky changes; render passes, no
compute, so WebGL2 runs them): the procedural sky into a 256² cubemap, its
mip chain, a 32² irradiance convolution, a 128² GGX-prefiltered cube with 5
roughness mips (filtered importance sampling), and a 256² BRDF LUT. The sky
cube is also what the sky pass draws. The spike's cube-face mapping flipped
`v` a second time, which mirrored the sky vertically (invisible in the spike,
which never drew the sky, but it also mirrored the ambient light on vertical
surfaces); `face_dir` now follows the WebGPU/Vulkan cube convention and the
background shows the horizon where it is.

## Backend tiers

| Tier | Backends | Reference directory | Notes |
|---|---|---|---|
| `Full` | Metal, Vulkan (GPU or lavapipe), WebGPU | `metal`, `vulkan`, `vulkan-cpu`, `webgpu` | compute available, unused so far |
| `NoCompute` | WebGL2 (wgpu GL) | `webgl2` (`gl` natively) | the same passes; constraints below |

Both tiers request `Limits::downlevel_webgl2_defaults()` at the adapter's
resolution, so one code path runs everywhere. The WebGL2 constraints the spike
solved stay in place: the shadow depth is also written to an `R32Float` colour
target and read with `textureLoad` (WebGL2 can neither filter nor
`textureLod` a depth texture), and PCF compares are explicit; the HDR target is
`Rgba16Float` (`EXT_color_buffer_float`); IBL and mips are render passes; no
`first_instance`; the GL surface is RGBA (sRGB when offered, otherwise encoded
in the post shader).

On 2026-10-17 a throwaway wasm page drew the `gltf-reference` scene through
`Renderer::with_surface` in headless Chrome 150 (ANGLE Metal) and was captured
over CDP: WebGPU (`Bgra8Unorm`, shader encode) RMSE 0.00068 and WebGL2
(`Rgba8UnormSrgb`) RMSE 0.00052 against the native Metal reference, 78 draws
on both. That page is not in the repository (see the open questions in the
M3 list).

## Testing

`crates/og-tests` is the gfx reference harness. Each named scene renders
headless at 1280×720 and is compared with
`crates/og-tests/references/<backend-tier>/<scene>.png`: a pixel is an outlier
when a channel differs by more than 32/255; the frame fails above 0.5 %
outliers or an RGB RMSE above 0.01. On failure the actual frame and a diff
(differences ×4, outliers red) go to `target/…/tmp/og-gfx/<tier>/`. A missing
reference fails and says how to record one.

| Scene | What it proves |
|---|---|
| `gltf-reference` | The spike's reference scene (trickster + dropship × 25, checker ground, the spike's camera at t = 59/60 s, exposure 0.8) with the sky pass, mips and the auto-fitted shadow box. |
| `spheres-20k` | 20,000 spheres (`shapes::sphere(16, 12)`, 384 triangles) submitted one instance at a time: 2 batches, 6 draw calls, 7,680,002 triangles. |

```sh
export CARGO_TARGET_DIR=$PWD/target/agent-render   # any target dir works
cargo test --release -p og-tests                   # gfx + the wasm32 compile check
OG_GFX_CAPTURE=1 cargo test --release -p og-tests --test gfx              # re-record every reference
OG_GFX_CAPTURE=spheres-20k cargo test --release -p og-tests --test gfx    # one scene
cargo run --release -p og-tests --bin og-gfx -- --all --frames 120        # stats, timings, RMSE
cargo run --release -p og-tests --bin og-gfx -- gltf-reference --out frame.png
```

References were recorded from release builds and are bit-for-bit stable
between release runs. A debug build differs slightly (RMSE 0.0035 and
0.0057, within tolerance) because `cos` of the constant camera angle is
constant-folded by LLVM in release and computed by the platform libm in
debug, one ulp apart.

Native frame times on an M4 Pro (Metal, 1280×720, `og-gfx --frames 120`):
`gltf-reference` 1.63 ms (spike 1.64 ms), `spheres-20k` 7.8 ms (spike, static,
8.0 ms), including the per-frame sort and instance upload and the sky pass.

## What M3 still owes

- **Render graph object** in place of the fixed pass sequence; a depth
  prepass and Forward+ clustered lights (CPU tile table on WebGL2).
- **Material system** from reflected material assets (this crate's `Material`
  is the plain struct they will map to); transparency and blending,
  double-sided materials, a normal matrix for non-uniform scale, vertex
  tangents.
- **3-cascade CSM** for the directional light. The current single
  orthographic box spends its depth bias in normalized depth over the whole
  scene, so on large scenes the bias exceeds small-object scale: in
  `spheres-20k` shadows have no visible effect at all.
- **Point and spot lights** with shadows by tier.
- **MSAA (GL) / TAA (WebGPU)**; nothing is antialiased today.
- **Bloom, auto exposure, AgX and a grading LUT**; SSAO/GTAO.
- **2D sprite batch and MSDF text.**
- **Capture readback on the web** (`read_offscreen` is native only; the
  offscreen target already exists on both) and a committed browser runner for
  the `webgpu`/`webgl2` reference tiers.
- Instance upload is the whole buffer every frame; dirty tracking and frustum
  culling come with the graph. Resources cannot be removed yet. Textures are
  uncompressed RGBA8 (KTX2/Basis is M4) and sampled without anisotropy.
- Shaders are WGSL compiled by naga at pipeline creation; cooking per backend
  at build time is still to do. `include_str!` reaches `crates/og-render/shaders/` outside the
  crate directory, which `cargo package` would reject.
