// The view depth's MIP chain (depth_mips.rs, the `depth-mips` target):
// level 0 the view depth itself, each level after it one value for each
// 2 x 2 texels of the one below. XeGTAO's depth prefilter (Intel, MIT,
// third_party/xegtao: XeGTAO_PrefilterDepths16x16 and
// XeGTAO_DepthMIPFilter): one 8 x 8 workgroup per 16 x 16 pixels, each
// thread reading its 2 x 2 pixels and writing them to level 0 and their
// filtered value to level 1, the levels after it reduced through workgroup
// memory. A level holds the 2 x 2's mean weighted towards the farthest of
// them: a texel within the AO's falloff of the farthest counts fully, one
// past the radius not at all, so a far sample of the GTAO's horizon search
// (fxpost.wgsl `gtao_bent`) finds the surface behind a thin near occluder
// rather than a depth floating between the two. Four levels where XeGTAO
// has five: WebGPU's default limits allow four storage textures in a
// stage. Prepended by fxpost_uniform.wgsl.

@group(0) @binding(0) var<uniform> fxp: FxPost;
// The depth the AO searches (r: view depth; the prepass's, or the one with
// the water's surfaces in front).
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var level0: texture_storage_2d<r32float, write>;
@group(0) @binding(3) var level1: texture_storage_2d<r32float, write>;
@group(0) @binding(4) var level2: texture_storage_2d<r32float, write>;
@group(0) @binding(5) var level3: texture_storage_2d<r32float, write>;

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

// `d` at texel `at` of each level, where the level has it (a storage
// texture is no function argument).
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
  let b = vec2<u32>(base);
  store0(b, d0);
  store0(b + vec2<u32>(1u, 0u), d1);
  store0(b + vec2<u32>(0u, 1u), d2);
  store0(b + vec2<u32>(1u, 1u), d3);
  let m1 = depth_mip_filter(d0, d1, d2, d3);
  store1(texel, m1);
  scratch[local.x][local.y] = m1;
  workgroupBarrier();
  // Every second thread in x and y: the 2 x 2 of level 1 at its corner.
  let l = local.xy;
  var m2 = 0.0;
  if (all(l % 2u == vec2<u32>(0u))) {
    m2 = depth_mip_filter(
      scratch[l.x][l.y], scratch[l.x + 1u][l.y],
      scratch[l.x][l.y + 1u], scratch[l.x + 1u][l.y + 1u]);
    store2(group.xy * 4u + l / 2u, m2);
  }
  workgroupBarrier();
  if (all(l % 2u == vec2<u32>(0u))) {
    scratch[l.x][l.y] = m2;
  }
  workgroupBarrier();
  // Every fourth: the 2 x 2 of level 2, two threads apart.
  if (all(l % 4u == vec2<u32>(0u))) {
    let m3 = depth_mip_filter(
      scratch[l.x][l.y], scratch[l.x + 2u][l.y],
      scratch[l.x][l.y + 2u], scratch[l.x + 2u][l.y + 2u]);
    store3(group.xy * 2u + l / 4u, m3);
  }
}
