// A view's resolve (views.rs): its HDR image into a render target or a
// rect of the frame, exposed and tone mapped (`fs_color`), as linear
// radiance (`fs_linear`), or its depth as the normalised linear view depth
// (`fs_depth`): this part is shared, views_color.wgsl or views_depth.wgsl
// follows it.
struct Resolve {
  params: vec4<f32>,  // x: exposure multiplier, y: tone curve (0 ACES, 1 none, 2 AgX, 3 Khronos PBR Neutral), z: 1 mirrored across x, w: 1 encode sRGB here
  rect: vec4<f32>,    // xy: the rect's origin in destination pixels
  depth: vec4<f32>,   // x: near, y: far, z: 1 orthographic, w: source width
};
@group(0) @binding(1) var<uniform> r: Resolve;

struct VsOut { @builtin(position) pos: vec4<f32> };

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
  var o: VsOut;
  let x = f32(i32(i & 1u) * 4 - 1);
  let y = f32(i32(i & 2u) * 2 - 1);
  o.pos = vec4<f32>(x, y, 0.0, 1.0);
  return o;
}

// The source texel of a destination pixel (mirrored across x for a mirror).
fn texel(pos: vec4<f32>) -> vec2<i32> {
  var p = vec2<i32>(pos.xy - r.rect.xy);
  if (r.params.z > 0.5) { p.x = i32(r.depth.w) - 1 - p.x; }
  return p;
}

fn aces(x: vec3<f32>) -> vec3<f32> {
  let a = 2.51; let b = 0.03; let c = 2.43; let d = 0.59; let e = 0.14;
  return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn agx_contrast(x: vec3<f32>) -> vec3<f32> {
  let x2 = x * x;
  let x4 = x2 * x2;
  return 15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4 - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232;
}

// As post.wgsl's AgX.
fn agx(x: vec3<f32>) -> vec3<f32> {
  let srgb_to_2020 = mat3x3<f32>(vec3<f32>(0.6274, 0.0691, 0.0164), vec3<f32>(0.3293, 0.9195, 0.0880), vec3<f32>(0.0433, 0.0113, 0.8956));
  let rec2020_to_srgb = mat3x3<f32>(vec3<f32>(1.6605, -0.1246, -0.0182), vec3<f32>(-0.5876, 1.1329, -0.1006), vec3<f32>(-0.0728, -0.0083, 1.1187));
  let inset = mat3x3<f32>(vec3<f32>(0.856627153315983, 0.137318972929847, 0.11189821299995), vec3<f32>(0.0951212405381588, 0.761241990602591, 0.0767994186031903), vec3<f32>(0.0482516061458583, 0.101439036467562, 0.811302368396859));
  let outset = mat3x3<f32>(vec3<f32>(1.1271005818144368, -0.1413297634984383, -0.14132976349843826), vec3<f32>(-0.11060664309660323, 1.157823702216272, -0.11060664309660294), vec3<f32>(-0.016493938717834573, -0.016493938717834257, 1.2519364065950405));
  var c = inset * (srgb_to_2020 * x);
  c = clamp((log2(max(c, vec3<f32>(1e-10))) + 12.47393) / 16.49999, vec3<f32>(0.0), vec3<f32>(1.0));
  c = outset * agx_contrast(c);
  c = pow(max(c, vec3<f32>(0.0)), vec3<f32>(2.2));
  return clamp(rec2020_to_srgb * c, vec3<f32>(0.0), vec3<f32>(1.0));
}

// As post.wgsl's Khronos PBR Neutral.
fn neutral(color: vec3<f32>) -> vec3<f32> {
  let x = min(color.r, min(color.g, color.b));
  var c = color - select(0.04, x - 6.25 * x * x, x < 0.08);
  let peak = max(c.r, max(c.g, c.b));
  if (peak < 0.76) { return clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)); }
  let new_peak = 1.0 - 0.0576 / (peak - 0.52);
  c *= new_peak / peak;
  let g = 1.0 - 1.0 / (0.15 * (peak - new_peak) + 1.0);
  return clamp(mix(c, vec3<f32>(new_peak), g), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn srgb_encode(c: vec3<f32>) -> vec3<f32> {
  let lo = c * 12.92;
  let hi = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
  return select(hi, lo, c <= vec3<f32>(0.0031308));
}

