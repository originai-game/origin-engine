// The occlusion test against the depth pyramid (cull_gpu.rs), written once
// for every cull: the instances' (cull.wgsl) and the crowds'
// (crowd_cull.wgsl) through their boxes (cull_box.wgsl), the clusters'
// (meshlet_cull.wgsl) and the scattered copies' (scatter_cull.wgsl)
// through their spheres. The including module declares `pyramid`.

// What a bound covers on screen: its pixel rectangle and nearest depth;
// `ok` is false when a corner is behind the camera.
struct ScreenBox {
  lo: vec2<f32>,
  hi: vec2<f32>,
  near: f32,
  ok: bool,
};

fn hiz_nothing() -> ScreenBox {
  return ScreenBox(vec2<f32>(1e30), vec2<f32>(-1e30), 1e30, true);
}

// Corner `i` (0 to 7) of the box of half extent 1.
fn hiz_corner(i: u32) -> vec3<f32> {
  return vec3<f32>(
    select(-1.0, 1.0, (i & 1u) != 0u),
    select(-1.0, 1.0, (i & 2u) != 0u),
    select(-1.0, 1.0, (i & 4u) != 0u));
}

// `r` grown to hold a corner at `clip` on a screen of `size` pixels.
fn hiz_extend(r: ptr<function, ScreenBox>, clip: vec4<f32>, size: vec2<f32>) {
  if (clip.w <= 1e-6) { (*r).ok = false; return; }
  let ndc = clip.xyz / clip.w;
  let px = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * size;
  (*r).lo = min((*r).lo, px);
  (*r).hi = max((*r).hi, px);
  (*r).near = min((*r).near, ndc.z);
}

// The box about the sphere `r` about `c` (world), seen through
// `view_proj` (meshlet.rs `ScreenRect::of_sphere`).
fn hiz_sphere(view_proj: mat4x4<f32>, c: vec3<f32>, r: f32, size: vec2<f32>) -> ScreenBox {
  var box = hiz_nothing();
  for (var i = 0u; i < 8u && box.ok; i++) {
    hiz_extend(&box, view_proj * vec4<f32>(c + r * hiz_corner(i), 1.0), size);
  }
  return box;
}

// Whether `r` lies behind the occluders' depth everywhere it covers, on a
// screen of `screen` pixels whose pyramid has `levels` levels
// (conservative: anything uncertain is visible).
fn hiz_hidden(r: ScreenBox, screen: vec2<u32>, levels: u32) -> bool {
  if (!r.ok || r.near <= 0.0) { return false; }
  let size = vec2<f32>(screen);
  // One pixel of margin, clamped to the screen.
  let x0 = u32(clamp(floor(r.lo.x) - 1.0, 0.0, size.x - 1.0));
  let y0 = u32(clamp(floor(r.lo.y) - 1.0, 0.0, size.y - 1.0));
  let x1 = u32(clamp(floor(r.hi.x) + 1.0, 0.0, size.x - 1.0));
  let y1 = u32(clamp(floor(r.hi.y) + 1.0, 0.0, size.y - 1.0));
  let texels = max(x1 - x0, y1 - y0) + 1u;
  // The level whose texels are at least the rectangle's size: at most
  // 2 x 2 of them cover it.
  var level = 0u;
  while ((1u << level) < texels) { level++; }
  if (level >= levels) { return false; }
  // The level's size from the screen's (as wgpu sizes mips), not
  // textureDimensions(pyramid, level): llvmpipe answers level 0's size for
  // every level, so a box at the right edge read a texel past the last and
  // got zero (nearest) depth.
  let dims = max(vec2<u32>(1u), screen >> vec2<u32>(level));
  let t0 = min(vec2<u32>(x0, y0) >> vec2<u32>(level), dims - 1u);
  let t1 = min(vec2<u32>(x1, y1) >> vec2<u32>(level), dims - 1u);
  var farthest = 0.0;
  for (var y = t0.y; y <= t1.y; y++) {
    for (var x = t0.x; x <= t1.x; x++) {
      farthest = max(farthest, textureLoad(pyramid, vec2<u32>(x, y), level).r);
    }
  }
  // A margin over the rasterized depth's rounding (16 ulps near 1).
  return r.near > farthest + 1e-6;
}
