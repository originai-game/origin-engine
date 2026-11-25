// Prefiltered colour MIP pyramid for screen-space GI (depth_mips.rs):
// one 8 x 8 workgroup per 16 x 16 full-resolution pixels reads the previous
// frame's lit image once, softly clamps extreme single-pixel fireflies, and
// writes four prefiltered levels (half, quarter, eighth, sixteenth) through
// workgroup memory in a single dispatch. Prepended by common.wgsl.

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var level0: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var level1: texture_storage_2d<rgba16float, write>;
@group(0) @binding(3) var level2: texture_storage_2d<rgba16float, write>;
@group(0) @binding(4) var level3: texture_storage_2d<rgba16float, write>;

var<workgroup> scratch_c: array<array<vec3<f32>, 8>, 8>;

fn fetch_clamped(at: vec2<i32>, last: vec2<i32>) -> vec3<f32> {
  let c = max(textureLoad(source, min(at, last), 0).rgb, vec3<f32>(0.0));
  return c * min(1.0, 64.0 / max(luma(c), 1e-4));
}

fn store_c0(at: vec2<u32>, c: vec3<f32>) {
  if (all(at < textureDimensions(level0))) { textureStore(level0, at, vec4<f32>(c, 1.0)); }
}
fn store_c1(at: vec2<u32>, c: vec3<f32>) {
  if (all(at < textureDimensions(level1))) { textureStore(level1, at, vec4<f32>(c, 1.0)); }
}
fn store_c2(at: vec2<u32>, c: vec3<f32>) {
  if (all(at < textureDimensions(level2))) { textureStore(level2, at, vec4<f32>(c, 1.0)); }
}
fn store_c3(at: vec2<u32>, c: vec3<f32>) {
  if (all(at < textureDimensions(level3))) { textureStore(level3, at, vec4<f32>(c, 1.0)); }
}

@compute @workgroup_size(8, 8)
fn cs_color_mips(
  @builtin(workgroup_id) group: vec3<u32>,
  @builtin(local_invocation_id) local: vec3<u32>,
) {
  let last = vec2<i32>(textureDimensions(source)) - 1;
  let texel = group.xy * 8u + local.xy;
  let base = vec2<i32>(texel * 2u);
  let m0 = 0.25 * (
    fetch_clamped(base, last)
    + fetch_clamped(base + vec2<i32>(1, 0), last)
    + fetch_clamped(base + vec2<i32>(0, 1), last)
    + fetch_clamped(base + vec2<i32>(1, 1), last)
  );
  store_c0(texel, m0);
  scratch_c[local.x][local.y] = m0;
  workgroupBarrier();
  let l = local.xy;
  var m1 = vec3<f32>(0.0);
  if (all(l % 2u == vec2<u32>(0u))) {
    m1 = 0.25 * (
      scratch_c[l.x][l.y] + scratch_c[l.x + 1u][l.y]
      + scratch_c[l.x][l.y + 1u] + scratch_c[l.x + 1u][l.y + 1u]
    );
    store_c1(group.xy * 4u + l / 2u, m1);
  }
  workgroupBarrier();
  if (all(l % 2u == vec2<u32>(0u))) {
    scratch_c[l.x][l.y] = m1;
  }
  workgroupBarrier();
  var m2 = vec3<f32>(0.0);
  if (all(l % 4u == vec2<u32>(0u))) {
    m2 = 0.25 * (
      scratch_c[l.x][l.y] + scratch_c[l.x + 2u][l.y]
      + scratch_c[l.x][l.y + 2u] + scratch_c[l.x + 2u][l.y + 2u]
    );
    store_c2(group.xy * 2u + l / 4u, m2);
  }
  workgroupBarrier();
  if (all(l % 4u == vec2<u32>(0u))) {
    scratch_c[l.x][l.y] = m2;
  }
  workgroupBarrier();
  if (l.x == 0u && l.y == 0u) {
    let m3 = 0.25 * (
      scratch_c[0][0] + scratch_c[4][0]
      + scratch_c[0][4] + scratch_c[4][4]
    );
    store_c3(group.xy, m3);
  }
}
