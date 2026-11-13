// What any shader may call, after the hashes (hash.wgsl): one copy each of
// the octahedral map of unit vectors, the triangle that covers the screen,
// the view-space point under a pixel and interleaved gradient noise. Pure
// functions: a module that reads its own uniform passes it in. The two
// files are LIBRARY_WGSL, which the frame group's text starts with
// (FRAME_WGSL) and a module without the frame group puts first.

// Octahedral encoding of a unit vector into -1..1 (two channels): the
// G-buffer's normals, the probes' maps, a compact vertex's normal
// (compact.rs `oct_encode` is the same map).
fn oct_encode(n: vec3<f32>) -> vec2<f32> {
  let p = n.xy / (abs(n.x) + abs(n.y) + abs(n.z));
  if (n.z >= 0.0) { return p; }
  return (1.0 - abs(p.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), p >= vec2<f32>(0.0));
}

// The unit vector `oct_encode` mapped to `e`.
fn oct_decode(e: vec2<f32>) -> vec3<f32> {
  var n = vec3<f32>(e, 1.0 - abs(e.x) - abs(e.y));
  if (n.z < 0.0) {
    n = vec3<f32>((1.0 - abs(n.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), n.xy >= vec2<f32>(0.0)), n.z);
  }
  return normalize(n);
}

// Vertex `i` (0 to 2) of the one triangle that covers the screen, in clip
// space at depth 0: (-1, -1), (3, -1), (-1, 3).
fn fullscreen_clip(i: u32) -> vec4<f32> {
  return vec4<f32>(f32(i32(i & 1u) * 4 - 1), f32(i32(i & 2u) * 2 - 1), 0.0, 1.0);
}

// The view-space point at `uv` whose view depth is `d`, through the
// projection's inverse (perspective or orthographic).
fn view_point(inv_proj: mat4x4<f32>, uv: vec2<f32>, d: f32) -> vec3<f32> {
  let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
  let a = inv_proj * vec4<f32>(ndc, 0.0, 1.0);
  let b = inv_proj * vec4<f32>(ndc, 1.0, 1.0);
  let p0 = a.xyz / a.w;
  let p1 = b.xyz / b.w;
  let t = (d + p0.z) / (p0.z - p1.z);
  return mix(p0, p1, t);
}

// Interleaved gradient noise (Jimenez 2014), 0..1: a fixed pattern over
// the pixels, blue-noise-like without a texture.
fn ign(px: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(dot(px, vec2<f32>(0.06711056, 0.00583715))));
}
