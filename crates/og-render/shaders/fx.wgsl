// The lit passes' effect group: what a frame's screen-space effects give
// every lit surface (the screen AO, the decals and their clusters, the
// planar reflection's plane), bound at the group the module's layout puts
// it (`fx_wgsl(group)` in og-render-data's shader_source.rs; decals.wgsl
// declares `DecalGpu`).
// `FxUniform` in fx.rs: per-frame effect inputs of the lit passes.
struct Fx {
  view_proj_unjittered: mat4x4<f32>,
  prev_view_proj: mat4x4<f32>,
  screen: vec4<f32>,      // w, h, 1/w, 1/h
  ao: vec4<f32>,          // x: 1 when the AO texture is valid, y: intensity
  decal_info: vec4<f32>,  // x: decals (negative: no decal clusters, walk them all), y: atlas (1 linear, 2 sRGB), z: cos fade width, w: the previous joints' section (0: none; vs_prepass_skinned)
  planar: vec4<f32>,      // the planar reflection's plane (n, n·at); n 0: none, then w -1 while drawing it
  motion: vec4<f32>,      // x: 1 when the frame's motion vectors are read (TAA, motion blur, SSR, SSGI, a custom post), 0 when nothing reads them
};
@group(FX_GROUP) @binding(0) var<uniform> fx: Fx;
@group(FX_GROUP) @binding(1) var ssao_tex: texture_2d<f32>;
@group(FX_GROUP) @binding(2) var decal_atlas: texture_2d<f32>;
@group(FX_GROUP) @binding(3) var fx_sampler: sampler;
// The decal records and their clusters (fx.rs, clusters.rs `DecalBins`):
// one word per cluster (first reference, 24 bits; count, 8 bits), then the
// references, one record index a word.
@group(FX_GROUP) @binding(5) var<storage, read> decal_list: array<DecalGpu>;
@group(FX_GROUP) @binding(6) var<storage, read> decal_cells: array<u32>;
