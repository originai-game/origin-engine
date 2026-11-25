// The view depth's MIP chain (depth_mips.rs, the `depth-mips` target):
// stores half, quarter, eighth and sixteenth resolution filtered view depths
// (levels 1..4 of a 5-level XeGTAO pyramid; level 0 is read directly from the
// prepass's `lindepth` without copying it). One 8 x 8 workgroup covers 16 x 16
// full-res pixels, reducing all four mip levels in workgroup memory in one
// dispatch while staying within WebGPU's 4-storage-texture limit. Prepended by
// fxpost_uniform.wgsl.

@group(0) @binding(0) var<uniform> fxp: FxPost;
// The full-resolution view depth (r: view depth in metres).
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var level0: texture_storage_2d<r32float, write>; // 1/2
@group(0) @binding(3) var level1: texture_storage_2d<r32float, write>; // 1/4
@group(0) @binding(4) var level2: texture_storage_2d<r32float, write>; // 1/8
@group(0) @binding(5) var level3: texture_storage_2d<r32float, write>; // 1/16

var<workgroup> scratch: array<array<f32, 8>, 8>;

// XeGTAO_DepthMIPFilter over the GTAO's own falloff: its weight falls from
// half the radius to the radius (fxpost.wgsl `gtao_bent`, `2 - 2 l /
// radius`), the radius XeGTAO's 0.75 of the effect radius, which grows
// with depth under PostEffects.aoRadiusScale as the search's does.
fn depth_mip_filter(d0: f32, d1: f32, d2: f32, d3: f32) -> f32 {
  let far = max(max(d0, d1), max(d2, d3));
  let radius = select(fxp.ao.x, min(fxp.ao.x, fxp.ao2.x * far), fxp.ao2.x > 0.0);
  // XeGTAO's falloff over 0.75 of the radius: full weight within its
  // first half, none past it.
  let range = max(0.75 * radius * 0.5, 1e-4);
  let w = saturate(2.0 - (vec4<f32>(far) - vec4<f32>(d0, d1, d2, d3)) / range);
  return dot(w, vec4<f32>(d0, d1, d2, d3)) / (w.x + w.y + w.z + w.w);
}

fn store0(at: vec2<u32>, d: f32) {
  if (all(at < textureDimensions(level0))) { textureStore(level0, at, vec4<f32>(d, 0.0, 0.0, 1.0)); }
}
fn store1(at: vec2<u32>, d: f32) {
  if (all(at < textureDimensions(level1))) { textureStore(level1, at, vec4<f32>(d, 0.0, 0.0, 1.0)); }
}
fn store2(at: vec2<u32>, d: f32) {
  if (all(at < textureDimensions(level2))) { textureStore(level2, at, vec4<f32>(d, 0.0, 0.0, 1.0)); }
}
fn store3(at: vec2<u32>, d: f32) {
  if (all(at < textureDimensions(level3))) { textureStore(level3, at, vec4<f32>(d, 0.0, 0.0, 1.0)); }
}

@compute @workgroup_size(8, 8)
fn cs_depth_mips(
  @builtin(workgroup_id) group: vec3<u32>,
  @builtin(local_invocation_id) local: vec3<u32>,
) {
  let size = vec2<i32>(textureDimensions(source));
  let texel = group.xy * 8u + local.xy;
  let base = vec2<i32>(texel * 2u);
  let last = size - 1;
  let d0 = textureLoad(source, min(base, last), 0).r;
  let d1 = textureLoad(source, min(base + vec2<i32>(1, 0), last), 0).r;
  let d2 = textureLoad(source, min(base + vec2<i32>(0, 1), last), 0).r;
  let d3 = textureLoad(source, min(base + vec2<i32>(1, 1), last), 0).r;
  let m1 = depth_mip_filter(d0, d1, d2, d3);
  store0(texel, m1);
  scratch[local.x][local.y] = m1;
  workgroupBarrier();
  let l = local.xy;
  var m2 = 0.0;
  if (all(l % 2u == vec2<u32>(0u))) {
    m2 = depth_mip_filter(
      scratch[l.x][l.y], scratch[l.x + 1u][l.y],
      scratch[l.x][l.y + 1u], scratch[l.x + 1u][l.y + 1u]);
    store1(group.xy * 4u + l / 2u, m2);
  }
  workgroupBarrier();
  if (all(l % 2u == vec2<u32>(0u))) {
    scratch[l.x][l.y] = m2;
  }
  workgroupBarrier();
  var m3 = 0.0;
  if (all(l % 4u == vec2<u32>(0u))) {
    m3 = depth_mip_filter(
      scratch[l.x][l.y], scratch[l.x + 2u][l.y],
      scratch[l.x][l.y + 2u], scratch[l.x + 2u][l.y + 2u]);
    store2(group.xy * 2u + l / 4u, m3);
  }
  workgroupBarrier();
  if (all(l % 4u == vec2<u32>(0u))) {
    scratch[l.x][l.y] = m3;
  }
  workgroupBarrier();
  if (l.x == 0u && l.y == 0u) {
    let m4 = depth_mip_filter(
      scratch[0][0], scratch[4][0],
      scratch[0][4], scratch[4][4]);
    store3(group.xy, m4);
  }
}
