// The occlusion test against the depth pyramid (cull_gpu.rs), shared by
// the instances' cull (cull.wgsl) and the crowds' (crowd_cull.wgsl). The
// including module defines `params` with `sizes` (y: the pyramid's levels,
// zw: the screen's size) and `view_proj`, and `pyramid`.

// An instance's box (its local bounds through its transform) on screen:
// its pixel rectangle and nearest depth; `ok` is false when a corner is
// behind the camera.
struct ScreenBox {
  lo: vec2<f32>,
  hi: vec2<f32>,
  near: f32,
  ok: bool,
};

fn screen_box(m: mat4x4<f32>, center: vec3<f32>, extent: vec3<f32>) -> ScreenBox {
  let size = vec2<f32>(params.sizes.zw);
  var r = ScreenBox(vec2<f32>(1e30), vec2<f32>(-1e30), 1e30, true);
  for (var i = 0u; i < 8u; i++) {
    let corner = center + extent * vec3<f32>(
      select(-1.0, 1.0, (i & 1u) != 0u),
      select(-1.0, 1.0, (i & 2u) != 0u),
      select(-1.0, 1.0, (i & 4u) != 0u));
    let clip = mul_point(params.view_proj, m * vec4<f32>(corner, 1.0));
    if (clip.w <= 1e-6) { r.ok = false; return r; }
    let ndc = clip.xyz / clip.w;
    let px = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * size;
    r.lo = min(r.lo, px);
    r.hi = max(r.hi, px);
    r.near = min(r.near, ndc.z);
  }
  return r;
}

// Whether the instance's box lies behind the occluders' depth everywhere
// it covers (conservative: anything uncertain is visible).
fn occluded(m: mat4x4<f32>, center: vec3<f32>, extent: vec3<f32>) -> bool {
  let size = vec2<f32>(params.sizes.zw);
  let r = screen_box(m, center, extent);
  if (!r.ok) { return false; }
  let lo = r.lo;
  let hi = r.hi;
  let near = r.near;
  if (near <= 0.0) { return false; }
  // One pixel of margin, clamped to the screen.
  let x0 = u32(clamp(floor(lo.x) - 1.0, 0.0, size.x - 1.0));
  let y0 = u32(clamp(floor(lo.y) - 1.0, 0.0, size.y - 1.0));
  let x1 = u32(clamp(floor(hi.x) + 1.0, 0.0, size.x - 1.0));
  let y1 = u32(clamp(floor(hi.y) + 1.0, 0.0, size.y - 1.0));
  let texels = max(x1 - x0, y1 - y0) + 1u;
  // The level whose texels are at least the rectangle's size: at most
  // 2 x 2 of them cover it.
  var level = 0u;
  while ((1u << level) < texels) { level++; }
  if (level >= params.sizes.y) { return false; }
  // The level's size from the screen's (as wgpu sizes mips), not
  // textureDimensions(pyramid, level): llvmpipe answers level 0's size for
  // every level, so a box at the right edge read a texel past the last and
  // got zero (nearest) depth.
  let dims = max(vec2<u32>(1u), params.sizes.zw >> vec2<u32>(level));
  let t0 = min(vec2<u32>(x0, y0) >> vec2<u32>(level), dims - 1u);
  let t1 = min(vec2<u32>(x1, y1) >> vec2<u32>(level), dims - 1u);
  var farthest = 0.0;
  for (var y = t0.y; y <= t1.y; y++) {
    for (var x = t0.x; x <= t1.x; x++) {
      farthest = max(farthest, textureLoad(pyramid, vec2<u32>(x, y), level).r);
    }
  }
  // A margin over the rasterized depth's rounding (16 ulps near 1).
  return near > farthest + 1e-6;
}
