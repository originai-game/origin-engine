# WebGL2 on Windows: the light tables without dynamic vector indexing

studio-win, 2026-10-27. Branch `webgl2-cluster-table` on `origin/main`
`420d64b1`. Follows `../windows-x64-95104f3e/NOTES.md` §5, where WebGL2
did not start in Chrome or Edge on Windows.

## Host

- Windows 10 Pro 25H2 (10.0.26200), NVIDIA GeForce RTX 5060, driver
  610.88 (32.0.16.1088).
- Chrome 155.0.8059.41, Edge 155.0.4283.45. WebGL2 is ANGLE on D3D11:
  `ANGLE (NVIDIA, NVIDIA GeForce RTX 5060 (0x00002D05) Direct3D11 vs_5_0
  ps_5_0, D3D11)`, FXC `ps_5_0`, `MAX_UNIFORM_BLOCK_SIZE` 65536. WebGPU is
  Dawn on D3D12.

## What failed, and why each step of the layout

ANGLE's D3D11 backend turns GLSL into HLSL for FXC. Three of its
translations broke or slowed the lit shaders, all on the uniform blocks of
the GL tier (`LightTables::Uniform`, clusters.rs):

1. **`v[i]` on a uniform vector.** `cluster_table[c >> 2u][c & 3u]` and the
   references' `[(j >> 2u) & 3u]` each became a `dyn_index_uvec4_uint4_int`
   helper, written twice; FXC refused it (`X3003: redefinition`) at the
   first draw, the context was lost and wgpu's next pipeline (`sprite`)
   failed. At d5f4895b, 19 fragment entry points of 10 modules failed so:
   pbr, pbr array maps, pbr lean, physical, custom full lit, scatter,
   terrain, terrain (layer arrays), fluid 3d, volumetric.
2. **A light record read whole from a uniform array of structs.** With the
   word picked by `select` or masks (the validation's attempt), ANGLE copies
   every `LocalLight` of the block into a static array before `main`; FXC
   took 129.7 s, 130.9 s and 133.5 s to link the lit shader.
3. **A block whose only member is an array of 50 or more elements** becomes
   a D3D StructuredBuffer (ANGLE's
   `RecordUniformBlocksWithLargeArrayMember`). With the tables as bare
   `array<vec4<u32>, N>` every lit shader linked in 4.2 to 5.0 s, but
   fx-showcase still died at tick 0: Chrome's GPU process crashed
   (0xC000001D) in `rx::StateManager11::setShaderResourceInternal` from
   `syncUniformBuffersForShader`, drawing the lit `fs` then `fs_lean` in one
   context (the lit shader binds 15 textures; five such buffers come
   after them). That the buffers' SRV slots pass the 16 ANGLE tracks is a
   reading of the stack, not proven; wrapping the arrays removes the crash.

The layout that landed:

- **Storage (full tier):** `light_list: array<LocalLight, 256>`,
  `cluster_table: array<u32, CLUSTERS>`, `cluster_refs: array<u32, refs/4>`:
  one u32 per cluster, indexed directly. The bytes are those of before, so
  the CPU uploads did not change.
- **Uniform (GL tier):** each array the one member of a struct (a constant
  buffer, item 3): `LightWords { v: array<vec4<f32>, 1024> }` (a light is
  four vec4s, `light_at`), `ClusterTableWords { v: array<vec4<u32>, 864> }`,
  `ClusterRefWords { v: array<vec4<u32>, 1024> }`. A word is read by a
  dynamic array index and its lane picked with masks (`lane`), never
  `v[k]` (item 1). The same wrapping for `ltc_table` (area.wgsl,
  binding 11) and `probe_data` (lightmap.rs, binding 10), the other two
  blocks of the GL tier that ANGLE would make StructuredBuffers.
- Block sizes are unchanged: 16,384, 13,824 and 16,384 bytes, probes and
  LTC 16,384 each; within WebGL2's 16 KiB minimum.
- pbr_fs, physical and area's `surface_local_lights` read the frame's
  lights in one loop (`frame_lights`, `frame_light`) instead of two
  (clustered, and every light for a probe capture), so the light function
  is inlined once.
- Two other reads of a uniform vector by a variable on the GL tier:
  terrain.wgsl's `tile_ref` (`table.entries[i / 4u][i % 4u]`, now masks)
  and pack.wgsl's `pack.pick[c]` (now a local copy).

`crates/og-render/tests/shaders.rs`
`gl_tier_dynamic_vector_indexes_are_known` walks naga's IR of every
GL-tier module for a dynamic index into a vector or matrix (or a pointer
to one) and allows only the listed (module, function) pairs, all on local
vectors: `object_id` (pbr, pbr array maps, pbr lean, physical, stylise
object), stylise `hatch_surface` and `dots_surface`, terrain and terrain
(layer arrays) `layer_weights` and `terrain_look`, pack `fs`, custom post
(crt) `post`. Putting `cluster_table.v[c >> 2u][c & 3u]` back fails it in
every lit module.

## Link times (Chrome 155, WebGL2, ANGLE D3D11)

A page links each GL entry point of `og_render::shader_sources(false)`
(256) with its uniform blocks bound and one sampler per unit, then draws
once; cold profile, one program at a time.

| Fragment entry point | d5f4895b | select/masks (reverted) | bare arrays (crashes) | landed |
|---|---|---|---|---|
| pbr `fs` | X3003 | 129.7–133.5 s | 5.0 s | 9.9 s |
| pbr `fs_lean` | X3003 | | | 10.0 s |
| physical `fs` | X3003 | | 5.0 s | 11.4 s |
| terrain `fs` | X3003 | | 4.9 s | 11.1 s |
| terrain (layer arrays) `fs` | X3003 | | 5.0 s | 8.1 s |
| pbr lean `fs` | X3003 | | | 10.8 s |
| pbr array maps `fs`, `fs_standard`, `fs_lean`; pbr lean `fs_lean` | X3003 | | | 9.4–9.9 s |
| lightmap variants (5) | X3003 | | | 6.7–8.3 s |
| fluid 3d `fs_composite` | X3003 | | | 4.8 s |
| scatter `fs` | X3003 | | | 4.2 s |
| custom full lit `fs` | X3003 | | | 3.9 s |
| volumetric `fs_raymarch` | X3003 | | | 1.9 s |

All 256 entry points link with no error and no lost context; every other
one links in under 1 s. The 25 draws that report `INVALID_OPERATION` are
the page's, not the shaders': skinned and sprite vertex shaders with
integer attributes it does not bind, and stylise object's `fs_object`,
which writes a `uvec4` to its float target. All of them render in the
scenes below.

The constant-buffer layout costs about 5 s per lit shader over the bare
arrays. Offline FXC on ANGLE's HLSL of pbr `fs`: 9.6 s, 9.0 s without
ANGLE's static copies of the blocks, 3.9 s with StructuredBuffers;
removing the area-light call alone gives 3.5 s, so the time is FXC on the
LTC and light reads through constant buffers. Leaving only the LTC table
a StructuredBuffer (16 SRVs) would save 1 to 2 s per lit shader, not
taken: it depends on the unproven slot limit above.

fx-showcase's startup on WebGL2 builds its 37 pipelines behind the gate in
25.4 s (Chrome) and 25.1 s (Edge), cold; on WebGPU 43 in 1.3 s.

## Results

- `cargo test -p og-render`: 89 unit, 13 integration (the GLSL ES, MSL,
  SPIR-V and HLSL translations and the ratchet), 1 doc: pass. Clippy
  `-D warnings`, fmt: clean.
- og-tests' lighting subset on DX12 (`light::`, `lights::`,
  `cluster_compute::`, `probes::`, `physical::`, `fluids3d::`, point/spot,
  local lights, lights beyond, showcase, volumetric, fog, rect, area,
  cookie, terrain lit, scatter): 67 of 67 pass in 128 s.
- `reference.ts` (fx-showcase, release web build of this tree):

| Lane | Result |
|---|---|
| Chrome WebGL2 | PASS 5/5 in 58.7 s: post 0.00675, lens 0.00675, fx 0.00469, 2d 0.00290, stylised 0.00864 (RMSE vs metal) |
| Edge WebGL2 | PASS 5/5 in 59.2 s, the same numbers |
| Chrome WebGPU | PASS 5/5: 0.00455, 0.00676, 0.00392, 0.00288, 0.00863, as at 95104f3e |
| Edge WebGPU | PASS 5/5, the same numbers |

- `reference-gfx.ts` (og-tests' 132 scenes and og-ui's):

| Lane | Rendered | Pass | Fail |
|---|---|---|---|
| Chrome WebGL2 | 128 (4 do not apply) in 305 s | 121 | 7 |
| Edge WebGL2 | 128 in 265 s | 121 | 7 |
| Chrome WebGPU | 131 in 102 s | 125 | 6 |
| Edge WebGPU | 131 in 108 s | 125 | 6 |

  Six failures in every lane are the known DX12-vs-metal six, with the
  numbers of `../windows-x64-95104f3e/NOTES.md` to five places:
  atmosphere-night 0.01650/0.091 %, atmosphere-moon 0.01613/0.090 %,
  sprites-array 0.02048/0.136 %, stylise-pixel 0.01484/1.649 %,
  tilemap-lit 0.03037/2.745 %, water-underwater 0.02975/3.883 %. The
  seventh on WebGL2 is og-ui/glass (Owed). The six frames with a
  `references/webgl2` frame all match it: msaa 0.00116, showcase 0.00969,
  overlay 0.00961, water-3d 0.00146, volumetric-fog 0.00097, water-ripples
  0.00100 (showcase is as close on WebGPU, 0.0099 against metal). There
  are no `references/webgpu` frames; WebGPU is judged against metal.

lights-256 on WebGL2 (Chrome): 0.000 % outliers, RMSE 0.00135 against
metal. I read the frame: every one of the coloured pools on the
checkerboard, the pillars with their shadows and the dark spheres are where
metal has them, no pool missing, no cluster seam. point-spot-lights 0.00295,
rect-lights 0.00113, light-cookies 0.00124, volumetric-fog against
`references/webgl2` 0.00097.

The other dynamic-index sites, each reached by a passing WebGL2 scene:
terrain (terrain-near, -mid, -far, -grass, -scatter-unshadowed), stylise
hatching and halftone (stylise-surface, stylise-halftone-surface), object
ids (stylise-object, stylise-outline-voxels, -ground), pack (materials-
physical, whose physical maps are packed on the GPU), crt (post-crt).

No reference frame changed, no tolerance moved, no tier added.

## Owed

- macrunner: the metal lane and Safari/Chrome WebGL2 on ANGLE Metal after
  the merge (the GL tier's WGSL changed; the full tier's storage
  declarations did too, same bytes).
- og-ui/glass on WebGL2 over D3D11 (RMSE 0.01728, 0.000 % outliers,
  against metal; no webgl2 reference): the blur of the frosted panel
  differs along the orange bar. og-ui and og-render's glass share nothing
  with this change, and this is the first Windows WebGL2 run that reached
  it. Not investigated.
- The full lane (AGENTS.md) was not run, by the brief: the coordinator's
  batch run covers it.
- Lit shaders link in 8 to 11 s on ANGLE D3D11, and fx-showcase's
  WebGL2 startup is 25 s cold. Within the bar set (not tens of seconds per
  shader), but the slowest part of a WebGL2 start on Windows.
